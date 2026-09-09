//! FSRS（Free Spaced Repetition Scheduler）调度算法实现。
//!
//! 采用 DSR（Difficulty / Stability / Retrievability）记忆模型，默认参数取自
//! 开源 FSRS 项目的 FSRS-4 权重，调度粒度按天。

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use super::{after_failure_state, CardMemory, SchedulingAlgorithm, SchedulingDecision, FSRS};
use crate::interface::anki::{AnkiError, CardGrade, CardState};

/// 可提取性公式中的常量：R(t,S) = (1 + FACTOR * t / S)^DECAY。
const FACTOR: f64 = 19.0 / 81.0;
const DECAY: f64 = -0.5;

/// FSRS-4 模型参数（17 个权重）。
#[derive(Clone, Debug)]
pub struct FsrsParameters {
    /// 首次复习的初始稳定性 S0，按 Again/Hard/Good/Easy 顺序。
    pub initial_stability: [f64; 4],
    /// 初始难度 D0。
    pub initial_difficulty: f64,
    /// 难度随作答等级的线性变化系数。
    pub difficulty_delta: f64,
    /// 难度向初始值回归的权重（均值回归）。
    pub difficulty_reversion: f64,
    /// 稳定性缩放系数（以 e 的幂形式参与计算）。
    pub stability_scale: f64,
    /// 稳定性指数。
    pub stability_exp: f64,
    /// 可提取性（Retrievability）系数。
    pub retrievability_factor: f64,
    /// 遗忘时的稳定性缩放系数。
    pub forget_scale: f64,
    /// 遗忘时的难度指数。
    pub forget_difficulty_exp: f64,
    /// 遗忘时的稳定性指数。
    pub forget_stability_exp: f64,
    /// 遗忘时的可提取性系数。
    pub forget_retrievability: f64,
    /// Hard 等级的稳定性惩罚系数。
    pub hard_penalty: f64,
    /// Easy 等级的稳定性奖励系数。
    pub easy_bonus: f64
}

impl Default for FsrsParameters {
    fn default() -> Self {
        Self {
            initial_stability: [0.4872, 1.4003, 3.7145, 13.8206],
            initial_difficulty: 5.1618,
            difficulty_delta: 0.8975,
            difficulty_reversion: 0.031,
            stability_scale: 1.6474,
            stability_exp: 0.1367,
            retrievability_factor: 1.0461,
            forget_scale: 2.1072,
            forget_difficulty_exp: 0.0793,
            forget_stability_exp: 0.3246,
            forget_retrievability: 1.587,
            hard_penalty: 0.2272,
            easy_bonus: 2.8755
        }
    }
}

/// FSRS 算法的私有状态。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FsrsState {
    /// 难度 D，范围 [1, 10]。
    pub difficulty: f64,
    /// 稳定性 S（天）。
    pub stability: f64,
    /// 上次复习的 Unix 秒时间戳；None 表示尚未复习过。
    pub last_review_at: Option<i64>,
}

impl Default for FsrsState {
    fn default() -> Self {
        Self {
            difficulty: 0.0,
            stability: 0.0,
            last_review_at: None,
        }
    }
}

/// FSRS 调度器。
pub struct Fsrs {
    parameters: FsrsParameters,
}

impl Fsrs {
    pub fn new(parameters: FsrsParameters) -> Self {
        Self { parameters }
    }

    fn grade_index(grade: CardGrade) -> usize {
        match grade {
            CardGrade::Again => 0,
            CardGrade::Hard => 1,
            CardGrade::Good => 2,
            CardGrade::Easy => 3,
        }
    }

    fn retrievability(&self, elapsed_days: f64, stability: f64) -> f64 {
        (1.0 + FACTOR * elapsed_days / stability).powf(DECAY)
    }

    fn next_difficulty(&self, difficulty: f64, grade: CardGrade) -> f64 {
        // FSRS 中作答等级取 1..=4（Again=1 ... Easy=4）。
        let fsrs_grade = Self::grade_index(grade) as f64 + 1.0;
        let next = difficulty - self.parameters.difficulty_delta * (fsrs_grade - 3.0);
        let reverted = self.parameters.difficulty_reversion * self.parameters.initial_difficulty
            + (1.0 - self.parameters.difficulty_reversion) * next;
        reverted.clamp(1.0, 10.0)
    }

    fn next_stability(
        &self,
        difficulty: f64,
        stability: f64,
        retrievability: f64,
        grade: CardGrade,
    ) -> f64 {
        if grade == CardGrade::Again {
            let forget = self.parameters.forget_scale
                * difficulty.powf(-self.parameters.forget_difficulty_exp)
                * ((stability + 1.0).powf(self.parameters.forget_stability_exp) - 1.0)
                * (self.parameters.forget_retrievability * (1.0 - retrievability)).exp();
            return forget.min(stability);
        }

        let hard_penalty = if grade == CardGrade::Hard {
            self.parameters.hard_penalty
        } else {
            1.0
        };
        let easy_bonus = if grade == CardGrade::Easy {
            self.parameters.easy_bonus
        } else {
            1.0
        };

        stability
            * (1.0
                + self.parameters.stability_scale.exp()
                    * (11.0 - difficulty)
                    * stability.powf(-self.parameters.stability_exp)
                    * ((self.parameters.retrievability_factor * (1.0 - retrievability)).exp()
                        - 1.0)
                    * hard_penalty
                    * easy_bonus)
    }
}

impl Default for Fsrs {
    fn default() -> Self {
        Self::new(FsrsParameters::default())
    }
}

impl SchedulingAlgorithm for Fsrs {
    fn name(&self) -> &'static str {
        FSRS
    }

    fn initial_state(&self) -> serde_json::Value {
        serde_json::to_value(FsrsState::default()).expect("FsrsState 序列化不会失败")
    }

    fn review(
        &self,
        memory: CardMemory,
        grade: CardGrade,
        now: DateTime<Utc>,
    ) -> Result<SchedulingDecision, AnkiError> {
        let state: FsrsState = match memory.algorithm_state {
            Some(value) => serde_json::from_value(value).map_err(|e| AnkiError {
                code: "invalid_state".into(),
                message: format!("无法解析 FSRS 状态: {e}"),
            })?,
            None => FsrsState::default(),
        };

        let (difficulty, stability) = match state.last_review_at {
            Some(seconds) if state.stability > 0.0 => {
                let elapsed_days = ((now.timestamp() - seconds) as f64 / 86_400.0).max(0.0);
                let retrievability = self.retrievability(elapsed_days, state.stability);
                let difficulty = self.next_difficulty(state.difficulty, grade);
                let stability =
                    self.next_stability(state.difficulty, state.stability, retrievability, grade);
                (difficulty, stability)
            }
            _ => {
                // 首次复习：直接使用初始稳定性与初始难度。
                let g = Self::grade_index(grade);
                (self.parameters.initial_difficulty, self.parameters.initial_stability[g])
            }
        };

        let interval_days = stability.max(1.0).ceil() as i64;
        let next_state = if grade == CardGrade::Again {
            after_failure_state(memory.state)
        } else {
            CardState::Review
        };

        Ok(SchedulingDecision {
            state: next_state,
            due_at: now + Duration::days(interval_days),
            algorithm_state: serde_json::to_value(FsrsState {
                difficulty,
                stability,
                last_review_at: Some(now.timestamp()),
            })
            .expect("FsrsState 序列化不会失败"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use crate::service::scheduler::SchedulingAlgorithm;

    #[test]
    fn print_fsrs_first_review_results() {
        let algorithm = Fsrs::default();
        let now = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();

        for grade in [CardGrade::Again, CardGrade::Hard, CardGrade::Good, CardGrade::Easy] {
            let result = algorithm
                .review(
                    CardMemory {
                        state: CardState::New,
                        algorithm_state: None,
                    },
                    grade,
                    now,
                )
                .unwrap();
            println!(
                "FSRS first grade={grade:?} state={:?} due_at={} algorithm_state={}",
                result.state,
                result.due_at.to_rfc3339(),
                serde_json::to_string_pretty(&result.algorithm_state).unwrap()
            );
        }
    }

    #[test]
    fn print_fsrs_review_sequence() {
        let algorithm = Fsrs::default();
        let first_at = Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap();
        let second_at = Utc.with_ymd_and_hms(2026, 1, 8, 12, 0, 0).unwrap();
        let mut state = None;
        let mut card_state = CardState::New;

        for (grade, now) in [
            (CardGrade::Good, first_at),
            (CardGrade::Easy, second_at),
            (CardGrade::Again, second_at),
        ] {
            let result = algorithm
                .review(
                    CardMemory {
                        state: card_state,
                        algorithm_state: state,
                    },
                    grade,
                    now,
                )
                .unwrap();
            println!(
                "FSRS sequence grade={grade:?} state={:?} due_at={} algorithm_state={}",
                result.state,
                result.due_at.to_rfc3339(),
                serde_json::to_string_pretty(&result.algorithm_state).unwrap()
            );
            card_state = result.state;
            state = Some(result.algorithm_state);
        }
    }
}
