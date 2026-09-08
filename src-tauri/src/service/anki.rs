//! Anki 业务服务：编排卡片的调度逻辑与持久化。

use std::sync::Arc;

use chrono::Utc;

use crate::interface::anki::{AnkiError, CardGrade, CardState, ReviewOutcome};
use crate::repository::anki as repo;

use super::scheduler::{CardMemory, SchedulingAlgorithm, SchedulerRegistry, SM2};

/// Anki 服务：持有数据库连接与算法注册表，对外提供卡片调度等用例。
pub struct AnkiService<'a> {
    conn: &'a rusqlite::Connection,
    schedulers: SchedulerRegistry,
}

impl<'a> AnkiService<'a> {
    pub fn new(conn: &'a rusqlite::Connection) -> Self {
        Self {
            conn,
            schedulers: SchedulerRegistry::new(),
        }
    }

    fn parse_id(card_id: &str) -> Result<i64, AnkiError> {
        card_id.parse::<i64>().map_err(|_| AnkiError {
            code: "invalid_id".into(),
            message: format!("无效的卡片 id: {card_id}"),
        })
    }

    fn parse_algorithm_state(
        raw: Option<String>,
        card_id: &str,
    ) -> Result<Option<serde_json::Value>, AnkiError> {
        match raw {
            Some(json) if !json.trim().is_empty() => {
                serde_json::from_str(&json).map(Some).map_err(|e| AnkiError {
                    code: "invalid_state".into(),
                    message: format!("卡片 {card_id} 的调度状态无法解析: {e}"),
                })
            }
            _ => Ok(None),
        }
    }

    fn algorithm_for(&self, name: &str) -> Arc<dyn SchedulingAlgorithm> {
        self.schedulers.get(name).unwrap_or_else(|| {
            self.schedulers
                .get(SM2)
                .expect("默认算法 sm2 必须已注册")
        })
    }

    fn not_found(card_id: &str) -> AnkiError {
        AnkiError {
            code: "not_found".into(),
            message: format!("卡片不存在: {card_id}"),
        }
    }

    /// 对卡片作答（重来/困难/良好/简单），计算并持久化下一次复习安排。
    pub fn grade_card(&self, card_id: &str, grade: CardGrade) -> Result<ReviewOutcome, AnkiError> {
        let id = Self::parse_id(card_id)?;
        let schedule =
            repo::find_schedule(self.conn, id)?.ok_or_else(|| Self::not_found(card_id))?;
        let repo::ScheduleRecord {
            state,
            algorithm,
            scheduler_state,
            ..
        } = schedule;

        let algorithm_impl = self.algorithm_for(&algorithm);
        let memory = CardMemory {
            state,
            algorithm_state: Self::parse_algorithm_state(scheduler_state, card_id)?,
        };

        let decision = algorithm_impl.review(memory, grade, Utc::now())?;

        let due_at = decision.due_at.to_rfc3339();
        let next = repo::ScheduleRecord {
            state: decision.state,
            algorithm,
            scheduler_state: Some(
                serde_json::to_string(&decision.algorithm_state).map_err(|e| AnkiError {
                    code: "internal".into(),
                    message: format!("序列化调度状态失败: {e}"),
                })?,
            ),
            due_at: Some(due_at.clone()),
        };
        repo::save_schedule(self.conn, id, &next)?;

        Ok(ReviewOutcome {
            card_id: card_id.to_string(),
            state: decision.state,
            due_at: Some(due_at),
        })
    }

    /// 彻底忘记某张卡片：恢复到「刚新增」的初始记忆状态。
    pub fn reset_card(&self, card_id: &str) -> Result<ReviewOutcome, AnkiError> {
        let id = Self::parse_id(card_id)?;
        let schedule =
            repo::find_schedule(self.conn, id)?.ok_or_else(|| Self::not_found(card_id))?;
        let repo::ScheduleRecord { algorithm, .. } = schedule;

        let algorithm_impl = self.algorithm_for(&algorithm);
        let next = repo::ScheduleRecord {
            state: CardState::New,
            algorithm,
            scheduler_state: Some(
                serde_json::to_string(&algorithm_impl.initial_state()).map_err(|e| AnkiError {
                    code: "internal".into(),
                    message: format!("序列化初始状态失败: {e}"),
                })?,
            ),
            due_at: None,
        };
        repo::save_schedule(self.conn, id, &next)?;

        Ok(ReviewOutcome {
            card_id: card_id.to_string(),
            state: CardState::New,
            due_at: None,
        })
    }
}
