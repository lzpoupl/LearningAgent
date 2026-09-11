//! 调度算法抽象：定义统一的算法接口与注册表，并实现与算法无关的卡片状态机。

pub mod fsrs;
pub mod sm2;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Duration, Utc};

use crate::config::SchedulerConfig;
use crate::interface::anki::{AnkiError, CardGrade, CardState};

/// SM-2 算法标识。
pub const SM2: &str = "sm2";
/// FSRS 算法标识。
pub const FSRS: &str = "fsrs";

/// 卡片当前的记忆状态：算法无关阶段 + 算法私有状态。
pub struct CardMemory {
    /// 卡片当前所处的复习阶段。
    pub state: CardState,
    /// 算法私有状态（JSON）。首次复习前为 None。
    pub algorithm_state: Option<serde_json::Value>,
}

/// 具体算法给出的调度结果：只包含间隔与算法私有状态。
///
/// 卡片所处的复习阶段由状态机决定，算法无需关心。
pub struct AlgorithmOutcome {
    /// 下次复习时间。
    pub due_at: DateTime<Utc>,
    /// 需要持久化的算法私有状态（JSON）。
    pub algorithm_state: serde_json::Value,
}

/// 状态机综合作答等级与算法结果后给出的完整调度决策。
pub struct SchedulingDecision {
    /// 作答后的复习阶段。
    pub state: CardState,
    /// 下次复习时间。
    pub due_at: DateTime<Utc>,
    /// 需要持久化的算法私有状态（JSON）。
    pub algorithm_state: serde_json::Value,
}

/// 调度算法抽象。
///
/// 算法只负责 Review 卡片的间隔与自身状态；New / Learning 的状态流转
/// 由 [`schedule`] 统一处理，因此与具体算法无关。
pub trait SchedulingAlgorithm: Send + Sync {
    /// 算法唯一标识，与卡片上存储的 algorithm 字段一致。
    fn name(&self) -> &'static str;

    /// 该算法的初始私有状态，用于新建或重置卡片。
    fn initial_state(&self) -> serde_json::Value;

    /// 根据作答等级计算下一次复习安排，并更新算法私有状态。
    fn review(
        &self,
        algorithm_state: Option<serde_json::Value>,
        grade: CardGrade,
        now: DateTime<Utc>,
    ) -> Result<AlgorithmOutcome, AnkiError>;
}

/// 算法注册表：按名字查找算法实现，是扩展新算法的唯一入口。
pub struct SchedulerRegistry {
    algorithms: HashMap<&'static str, Arc<dyn SchedulingAlgorithm>>,
}

impl SchedulerRegistry {
    /// 构建注册表，并注册内置的 SM-2 与 FSRS 算法。
    pub fn new() -> Self {
        let mut registry = Self {
            algorithms: HashMap::new(),
        };
        registry.register(Arc::new(sm2::Sm2));
        registry.register(Arc::new(fsrs::Fsrs::default()));
        registry
    }

    /// 注册一个算法实现。
    pub fn register(&mut self, algorithm: Arc<dyn SchedulingAlgorithm>) {
        self.algorithms.insert(algorithm.name(), algorithm);
    }

    /// 按名字查找算法实现。
    pub fn get(&self, name: &str) -> Option<Arc<dyn SchedulingAlgorithm>> {
        self.algorithms.get(name).cloned()
    }
}

impl Default for SchedulerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub fn schedule(
    algorithm: &dyn SchedulingAlgorithm,
    config: &SchedulerConfig,
    memory: CardMemory,
    grade: CardGrade,
    now: DateTime<Utc>,
) -> Result<SchedulingDecision, AnkiError> {
    match memory.state {
        CardState::New | CardState::Learning | CardState::Relearning => {
            learning_step(algorithm, config, memory, grade, now)
        }
        CardState::Review => review_step(algorithm, config, memory, grade, now),
    }
}

/// 学习阶段：使用配置的固定步长，算法私有状态保持不变。
fn learning_step(
    algorithm: &dyn SchedulingAlgorithm,
    config: &SchedulerConfig,
    memory: CardMemory,
    grade: CardGrade,
    now: DateTime<Utc>,
) -> Result<SchedulingDecision, AnkiError> {
    match grade {
        CardGrade::Again => Ok(SchedulingDecision {
            state: CardState::Learning,
            due_at: now + Duration::minutes(config.learning_again_minutes),
            algorithm_state: preserved_algorithm_state(algorithm, &memory),
        }),
        CardGrade::Hard => Ok(SchedulingDecision {
            state: CardState::Learning,
            due_at: now + Duration::minutes(config.learning_hard_minutes),
            algorithm_state: preserved_algorithm_state(algorithm, &memory),
        }),
        CardGrade::Good => Ok(SchedulingDecision {
            state: CardState::Review,
            due_at: now + Duration::minutes(config.learning_good_minutes),
            algorithm_state: preserved_algorithm_state(algorithm, &memory),
        }),
        // Easy 直接毕业进入 Review，间隔交由具体算法计算。
        CardGrade::Easy => {
            let outcome = algorithm.review(memory.algorithm_state, grade, now)?;
            Ok(SchedulingDecision {
                state: CardState::Review,
                due_at: outcome.due_at,
                algorithm_state: outcome.algorithm_state,
            })
        }
    }
}

/// 复习阶段：由具体算法安排；Again 时额外回到 Learning。
fn review_step(
    algorithm: &dyn SchedulingAlgorithm,
    config: &SchedulerConfig,
    memory: CardMemory,
    grade: CardGrade,
    now: DateTime<Utc>,
) -> Result<SchedulingDecision, AnkiError> {
    let outcome = algorithm.review(memory.algorithm_state, grade, now)?;

    let (state, due_at) = match grade {
        // 遗忘：算法已记录本次遗忘，状态机让卡片立即回到学习阶段重学。
        CardGrade::Again => (
            CardState::Learning,
            now + Duration::minutes(config.learning_again_minutes),
        ),
        _ => (CardState::Review, outcome.due_at),
    };

    Ok(SchedulingDecision {
        state,
        due_at,
        algorithm_state: outcome.algorithm_state,
    })
}

/// 学习阶段不推进算法，保留原有私有状态；缺失时用初始值补齐。
fn preserved_algorithm_state(
    algorithm: &dyn SchedulingAlgorithm,
    memory: &CardMemory,
) -> serde_json::Value {
    memory
        .algorithm_state
        .clone()
        .unwrap_or_else(|| algorithm.initial_state())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SchedulerConfig;
    use chrono::{TimeZone, Utc};

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 1, 1, 12, 0, 0).unwrap()
    }

    fn scheduler() -> SchedulerConfig {
        SchedulerConfig::default()
    }

    fn schedule_from(
        state: CardState,
        algorithm_state: Option<serde_json::Value>,
        grade: CardGrade,
    ) -> SchedulingDecision {
        schedule(
            &sm2::Sm2,
            &scheduler(),
            CardMemory {
                state,
                algorithm_state,
            },
            grade,
            now(),
        )
        .unwrap()
    }

    fn minutes(decision: &SchedulingDecision) -> i64 {
        (decision.due_at - now()).num_minutes()
    }

    #[test]
    fn new_card_uses_learning_steps() {
        let config = scheduler();
        let again = schedule_from(CardState::New, None, CardGrade::Again);
        assert_eq!(again.state, CardState::Learning);
        assert_eq!(minutes(&again), config.learning_again_minutes);

        let hard = schedule_from(CardState::New, None, CardGrade::Hard);
        assert_eq!(hard.state, CardState::Learning);
        assert_eq!(minutes(&hard), config.learning_hard_minutes);

        let good = schedule_from(CardState::New, None, CardGrade::Good);
        assert_eq!(good.state, CardState::Review);
        assert_eq!(minutes(&good), config.learning_good_minutes);
    }

    #[test]
    fn learning_again_and_hard_stay_in_learning() {
        let config = scheduler();
        for (grade, expected) in [
            (CardGrade::Again, config.learning_again_minutes),
            (CardGrade::Hard, config.learning_hard_minutes),
        ] {
            let decision = schedule_from(CardState::Learning, None, grade);
            assert_eq!(decision.state, CardState::Learning);
            assert_eq!(minutes(&decision), expected);
        }
    }

    #[test]
    fn learning_steps_follow_config() {
        let custom = SchedulerConfig {
            learning_again_minutes: 3,
            learning_hard_minutes: 12,
            learning_good_minutes: 20,
            ..SchedulerConfig::default()
        };
        let decision = schedule(
            &sm2::Sm2,
            &custom,
            CardMemory {
                state: CardState::New,
                algorithm_state: None,
            },
            CardGrade::Hard,
            now(),
        )
        .unwrap();
        assert_eq!(minutes(&decision), 12);
    }

    #[test]
    fn learning_easy_graduates_with_algorithm_interval() {
        let decision = schedule_from(CardState::Learning, None, CardGrade::Easy);
        assert_eq!(decision.state, CardState::Review);
        // SM-2 首次复习间隔 I(1)=1 天。
        assert_eq!((decision.due_at - now()).num_days(), 1);
    }

    #[test]
    fn learning_steps_keep_algorithm_state_untouched() {
        let initial = sm2::Sm2.initial_state();
        let decision = schedule_from(CardState::Learning, Some(initial.clone()), CardGrade::Again);
        assert_eq!(decision.algorithm_state, initial);
    }

    #[test]
    fn review_again_returns_to_learning_and_records_lapse() {
        let state = serde_json::to_value(sm2::Sm2State {
            repetitions: 4,
            ease_factor: 2.5,
            interval_days: 30,
        })
        .unwrap();

        let decision = schedule_from(CardState::Review, Some(state), CardGrade::Again);
        assert_eq!(decision.state, CardState::Learning);
        assert_eq!(minutes(&decision), scheduler().learning_again_minutes);

        let next: sm2::Sm2State = serde_json::from_value(decision.algorithm_state.clone()).unwrap();
        assert_eq!(next.repetitions, 0, "算法应记录本次遗忘");
    }

    #[test]
    fn review_good_stays_in_review() {
        let decision = schedule_from(CardState::Review, None, CardGrade::Good);
        assert_eq!(decision.state, CardState::Review);
        assert!((decision.due_at - now()).num_days() >= 1);
    }
}
