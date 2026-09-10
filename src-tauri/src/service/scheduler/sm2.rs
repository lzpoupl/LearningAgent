//! SM-2（SuperMemo-2）调度算法实现。
//!
//! 依据 Piotr Wozniak 的《Algorithm SM-2》
//! （<https://www.super-memory.com/english/ol/sm2.htm>）实现：
//!
//! - 每张卡片维护一个 E-Factor（难度系数），初始 2.5，下限 1.3；
//! - 复习间隔 I(1)=1、I(2)=6，n>2 时 I(n)=I(n-1)*EF，出现小数向上取整；
//! - 每次作答按质量分 q 更新 EF：EF' = EF + (0.1 - (5-q)*(0.08 + (5-q)*0.02))，
//!   间隔使用更新前的 EF 计算；
//! - 作答质量低于 3 时从 I(1) 重新开始，且保持 EF 不变。
//!
//! 本模块只负责 Review 卡片的间隔与自身状态，卡片阶段的流转见
//! [`super::schedule`]。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::{AlgorithmOutcome, SchedulingAlgorithm, SM2};
use crate::interface::anki::{AnkiError, CardGrade};

/// 卡片插入时的初始 E-Factor。
const INITIAL_EASE_FACTOR: f64 = 2.5;
/// E-Factor 下限；低于该值的卡片通常需要重新表述。
const MIN_EASE_FACTOR: f64 = 1.3;
/// 低于该质量分即视为遗忘。
const FAILURE_QUALITY: u32 = 3;

/// SM-2 算法的私有状态，序列化为 JSON 后存入卡片。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sm2State {
    /// 连续成功复习的次数 n。
    pub repetitions: u32,
    /// 难度系数（E-Factor），初始 2.5，下限 1.3。
    pub ease_factor: f64,
    /// 最近一次安排的复习间隔 I(n)，单位为天。
    pub interval_days: u32,
}

impl Default for Sm2State {
    fn default() -> Self {
        Self {
            repetitions: 0,
            ease_factor: INITIAL_EASE_FACTOR,
            interval_days: 0,
        }
    }
}

/// 经典 SM-2 算法。
pub struct Sm2;

/// 作答等级映射为 SM-2 的 0..5 质量分。
fn quality(grade: CardGrade) -> u32 {
    match grade {
        CardGrade::Again => 0,
        CardGrade::Hard => 3,
        CardGrade::Good => 4,
        CardGrade::Easy => 5,
    }
}

/// 根据质量分更新 E-Factor：EF' = EF + (0.1 - (5-q)*(0.08 + (5-q)*0.02))，
/// 并保证结果不低于下限 1.3。
fn updated_ease_factor(ease_factor: f64, quality: u32) -> f64 {
    let penalty = 5.0 - quality as f64;
    (ease_factor + (0.1 - penalty * (0.08 + penalty * 0.02))).max(MIN_EASE_FACTOR)
}

/// 计算第 `repetition` 次重复的间隔：I(1)=1，I(2)=6，
/// n>2 时 I(n)=I(n-1)*EF（向上取整）。
fn repetition_interval(repetition: u32, previous_interval: u32, ease_factor: f64) -> u32 {
    match repetition {
        0 | 1 => 1,
        2 => 6,
        _ => (previous_interval as f64 * ease_factor).ceil() as u32,
    }
}

impl SchedulingAlgorithm for Sm2 {
    fn name(&self) -> &'static str {
        SM2
    }

    fn initial_state(&self) -> serde_json::Value {
        serde_json::to_value(Sm2State::default()).expect("Sm2State 序列化不会失败")
    }

    fn review(
        &self,
        algorithm_state: Option<serde_json::Value>,
        grade: CardGrade,
        now: DateTime<Utc>,
    ) -> Result<AlgorithmOutcome, AnkiError> {
        let state: Sm2State = match algorithm_state {
            Some(value) => serde_json::from_value(value).map_err(|e| AnkiError {
                code: "invalid_state".into(),
                message: format!("无法解析 SM-2 状态: {e}"),
            })?,
            None => Sm2State::default(),
        };

        let quality = quality(grade);

        let next_state = if quality < FAILURE_QUALITY {
            // 遗忘：从第 1 次重复重新开始，E-Factor 保持不变。
            Sm2State {
                repetitions: 0,
                ease_factor: state.ease_factor,
                interval_days: repetition_interval(1, 0, state.ease_factor),
            }
        } else {
            // 成功：先用旧的 EF 计算本次间隔，再更新 EF。
            let repetition = state.repetitions + 1;
            let interval_days =
                repetition_interval(repetition, state.interval_days, state.ease_factor);
            Sm2State {
                repetitions: repetition,
                ease_factor: updated_ease_factor(state.ease_factor, quality),
                interval_days,
            }
        };

        Ok(AlgorithmOutcome {
            due_at: now + Duration::days(next_state.interval_days as i64),
            algorithm_state: serde_json::to_value(next_state).expect("Sm2State 序列化不会失败"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::scheduler::SchedulingAlgorithm;
    use chrono::{TimeZone, Utc};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
    }

    fn sm2_state(outcome: &AlgorithmOutcome) -> Sm2State {
        serde_json::from_value(outcome.algorithm_state.clone()).unwrap()
    }

    fn review(state: Option<serde_json::Value>, grade: CardGrade) -> AlgorithmOutcome {
        Sm2.review(state, grade, now()).unwrap()
    }

    #[test]
    fn good_reviews_grow_interval_by_ease_factor() {
        // I(1)=1，I(2)=6，其后 I(n)=I(n-1)*2.5 向上取整。
        let expected = [1, 6, 15, 38];
        let mut state = None;

        for (index, expected_interval) in expected.iter().enumerate() {
            let outcome = review(state, CardGrade::Good);
            let next = sm2_state(&outcome);
            assert_eq!(
                next.interval_days,
                *expected_interval,
                "第 {} 次复习的间隔",
                index + 1
            );
            assert!((next.ease_factor - INITIAL_EASE_FACTOR).abs() < 1e-9);
            assert_eq!(next.repetitions, index as u32 + 1);
            state = Some(outcome.algorithm_state);
        }
    }

    #[test]
    fn interval_uses_ease_factor_before_update() {
        // 连续三次 Good 后 EF=2.5、间隔=15；第四次 Easy 应仍用旧 EF：
        // 15*2.5=37.5 -> 38，而不是用更新后的 2.6 得到 39。
        let mut state = None;
        for _ in 0..3 {
            state = Some(review(state, CardGrade::Good).algorithm_state);
        }

        let outcome = review(state, CardGrade::Easy);
        let next = sm2_state(&outcome);
        assert_eq!(next.interval_days, 38);
        assert!((next.ease_factor - 2.6).abs() < 1e-9);
    }

    #[test]
    fn grades_adjust_ease_factor() {
        assert!((updated_ease_factor(2.5, 5) - 2.6).abs() < 1e-9);
        assert!((updated_ease_factor(2.5, 4) - 2.5).abs() < 1e-9);
        assert!((updated_ease_factor(2.5, 3) - 2.36).abs() < 1e-9);
        assert!((updated_ease_factor(1.35, 3) - MIN_EASE_FACTOR).abs() < 1e-9);
    }

    #[test]
    fn failure_restarts_without_changing_ease_factor() {
        let state = serde_json::to_value(Sm2State {
            repetitions: 3,
            ease_factor: 2.1,
            interval_days: 20,
        })
        .ok();

        let outcome = review(state, CardGrade::Again);
        let next = sm2_state(&outcome);
        assert_eq!(next.repetitions, 0);
        assert_eq!(next.interval_days, 1);
        assert!((next.ease_factor - 2.1).abs() < 1e-9);
    }
}
