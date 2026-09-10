//! SM-2（SuperMemo-2）调度算法实现。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::{after_failure_state, CardMemory, SchedulingAlgorithm, SchedulingDecision, SM2};
use crate::interface::anki::{AnkiError, CardGrade, CardState};

/// SM-2 算法的私有状态，序列化为 JSON 后存入卡片。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Sm2State {
    /// 连续答对的次数。
    pub repetitions: u32,
    /// 难度系数（E-Factor），初始 2.5，下限 1.3。
    pub ease_factor: f64,
    /// 当前复习间隔（天）。
    pub interval_days: u32,
}

impl Default for Sm2State {
    fn default() -> Self {
        Self {
            repetitions: 0,
            ease_factor: 2.5,
            interval_days: 0,
        }
    }
}

/// 经典 SM-2 算法。
pub struct Sm2;

/// 将作答等级映射为 SM-2 的 0..5 质量分。
fn quality(grade: CardGrade) -> u32 {
    match grade {
        CardGrade::Again => 0,
        CardGrade::Hard => 3,
        CardGrade::Good => 4,
        CardGrade::Easy => 5,
    }
}

/// EF 增量：EF' = EF + (0.1 - (5-q)*(0.08 + (5-q)*0.02))。
fn ease_delta(q: u32) -> f64 {
    0.1 - (5.0 - q as f64) * (0.08 + (5.0 - q as f64) * 0.02)
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
        memory: CardMemory,
        grade: CardGrade,
        now: DateTime<Utc>,
    ) -> Result<SchedulingDecision, AnkiError> {
        let mut state: Sm2State = match memory.algorithm_state {
            Some(value) => serde_json::from_value(value).map_err(|e| AnkiError {
                code: "invalid_state".into(),
                message: format!("无法解析 SM-2 状态: {e}"),
            })?,
            None => Sm2State::default(),
        };

        let q = quality(grade);

        let (next_state, interval_days) = if q < 3 {
            // 遗忘：重置连续答对次数，回到 1 天间隔。
            state.repetitions = 0;
            state.interval_days = 1;
            (after_failure_state(memory.state), 1)
        } else {
            state.ease_factor = (state.ease_factor + ease_delta(q)).max(1.3);
            state.repetitions += 1;
            let interval = match state.repetitions {
                1 => 1,
                2 => 6,
                _ => (state.interval_days as f64 * state.ease_factor).round() as u32,
            };
            state.interval_days = interval;
            (CardState::Review, interval)
        };

        Ok(SchedulingDecision {
            state: next_state,
            due_at: now + Duration::days(interval_days as i64),
            algorithm_state: serde_json::to_value(state).expect("Sm2State 序列化不会失败"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::scheduler::SchedulingAlgorithm;
    use chrono::{TimeZone, Utc};

    #[test]
    fn print_sm2_grading_results() {
        let algorithm = Sm2;
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let mut state = None;

        for grade in [
            CardGrade::Again,
            CardGrade::Hard,
            CardGrade::Good,
            CardGrade::Easy,
        ] {
            let result = algorithm
                .review(
                    CardMemory {
                        state: CardState::New,
                        algorithm_state: state,
                    },
                    grade,
                    now,
                )
                .unwrap();
            println!(
                "SM-2 grade={grade:?} state={:?} due_at={} algorithm_state={}",
                result.state,
                result.due_at.to_rfc3339(),
                serde_json::to_string_pretty(&result.algorithm_state).unwrap()
            );
            state = Some(result.algorithm_state);
        }
    }

    #[test]
    fn print_sm2_card_initial_review() {
        let algorithm = Sm2;
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let state = serde_json::to_value(Sm2State {
            repetitions: 0,
            ease_factor: 2.5,
            interval_days: 0,
        })
        .ok();

        for grade in [
            CardGrade::Again,
            CardGrade::Hard,
            CardGrade::Good,
            CardGrade::Easy,
        ] {
            let result = algorithm
                .review(
                    CardMemory {
                        state: CardState::New,
                        algorithm_state: state.clone(),
                    },
                    grade,
                    now,
                )
                .unwrap();
            println!(
                "SM-2 initial review grade={:?} due_at={}",
                grade,
                result.due_at.to_rfc3339()
            );
        }
    }

    #[test]
    fn print_sm2_again_from_review_state() {
        let algorithm = Sm2;
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let state = serde_json::to_value(Sm2State {
            repetitions: 3,
            ease_factor: 2.5,
            interval_days: 20,
        })
        .unwrap();

        let result = algorithm
            .review(
                CardMemory {
                    state: CardState::Review,
                    algorithm_state: Some(state),
                },
                CardGrade::Again,
                now,
            )
            .unwrap();
        println!(
            "SM-2 review again state={:?} due_at={} algorithm_state={}",
            result.state,
            result.due_at.to_rfc3339(),
            serde_json::to_string_pretty(&result.algorithm_state).unwrap()
        );
    }
}
