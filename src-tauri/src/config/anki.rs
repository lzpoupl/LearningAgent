//! Anki 模块配置。
//!
//! 字段名与 `config.toml` 中的键保持一致，采用 snake_case，避免不同配置源
//! 对键名大小写的处理差异导致配置被静默忽略。

use serde::{Deserialize, Serialize};

use crate::service::scheduler::SM2;

/// Anki 模块配置。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AnkiConfig {
    /// 复习策略调度器配置。
    pub scheduler: SchedulerConfig,
}

/// 复习策略调度器配置：学习阶段步长与当前启用的算法。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct SchedulerConfig {
    /// 当前启用的调度算法（算法切换），决定新卡使用哪种策略，如 `sm2` / `fsrs`。
    pub algorithm: String,
    /// 学习阶段答「重来」（Again）后的复习延时（分钟）。
    pub learning_again_minutes: i64,
    /// 学习阶段答「困难」（Hard）后的复习延时（分钟）。
    pub learning_hard_minutes: i64,
    /// 学习阶段答「良好」（Good）后的复习延时（分钟）。
    pub learning_good_minutes: i64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            algorithm: SM2.to_string(),
            learning_again_minutes: 1,
            learning_hard_minutes: 6,
            learning_good_minutes: 10,
        }
    }
}
