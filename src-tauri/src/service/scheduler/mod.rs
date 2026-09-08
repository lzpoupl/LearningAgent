//! 调度算法抽象：定义统一的算法接口与注册表，便于接入多种算法（SM-2、FSRS 等）。

pub mod fsrs;
pub mod sm2;

use std::collections::HashMap;
use std::sync::Arc;

use chrono::{DateTime, Utc};

use crate::interface::anki::{AnkiError, CardGrade, CardState};

/// SM-2 算法标识。
pub const SM2: &str = "sm2";
/// FSRS 算法标识。
pub const FSRS: &str = "fsrs";

/// 卡片当前的记忆状态：算法无关的阶段 + 算法私有状态。
pub struct CardMemory {
    /// 卡片当前所处的复习阶段。
    pub state: CardState,
    /// 算法私有状态（JSON）。首次复习前为 None。
    pub algorithm_state: Option<serde_json::Value>,
}

/// 一次作答后，算法给出的调度决策。
pub struct SchedulingDecision {
    /// 作答后的复习阶段。
    pub state: CardState,
    /// 下次复习时间。
    pub due_at: DateTime<Utc>,
    /// 需要持久化的算法私有状态（JSON）。
    pub algorithm_state: serde_json::Value,
}

/// 调度算法抽象。新增一种算法只需实现该 trait，并注册到 SchedulerRegistry。
pub trait SchedulingAlgorithm: Send + Sync {
    /// 算法唯一标识，与卡片上存储的 algorithm 字段一致。
    fn name(&self) -> &'static str;

    /// 该算法的初始私有状态，用于新建或重置卡片。
    fn initial_state(&self) -> serde_json::Value;

    /// 根据当前记忆状态与作答等级，计算下一次复习安排。
    fn review(
        &self,
        memory: CardMemory,
        grade: CardGrade,
        now: DateTime<Utc>,
    ) -> Result<SchedulingDecision, AnkiError>;
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

/// 作答失败（Again）后卡片所处的复习阶段。
pub fn after_failure_state(current: CardState) -> CardState {
    match current {
        CardState::New | CardState::Learning => CardState::Learning,
        CardState::Review | CardState::Relearning => CardState::Relearning,
    }
}
