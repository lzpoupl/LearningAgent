//! Anki 业务服务：编排卡片与牌组的业务逻辑、调度与持久化。

use std::sync::{Arc, Mutex, MutexGuard};

use chrono::Utc;

use crate::interface::anki::{
    AnkiError, Card, CardGrade, CardQuery, CardSearch, CardState, Deck, NewCard, ReviewOutcome,
    UpdateCardContent,
};
use crate::repository::anki as card_repo;
use crate::repository::deck as deck_repo;

use super::scheduler::{CardMemory, SchedulerRegistry, SchedulingAlgorithm, SM2};

/// Anki 服务：拥有数据库连接与算法注册表，在应用启动时构建一次，之后由各命令共享。
pub struct AnkiService {
    db: Arc<Mutex<rusqlite::Connection>>,
    schedulers: Arc<SchedulerRegistry>,
}

impl AnkiService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>, schedulers: Arc<SchedulerRegistry>) -> Self {
        Self { db, schedulers }
    }

    /// 获取数据库连接锁；锁在方法内部持有，命令层无需关心并发细节。
    fn conn(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, AnkiError> {
        self.db.lock().map_err(|_| AnkiError {
            code: "internal".into(),
            message: "数据库连接不可用".into(),
        })
    }

    // ---------- 公共用例 ----------

    pub fn get_subdecks(&self, deck_path: &str) -> Result<Vec<Deck>, AnkiError> {
        let conn = self.conn()?;
        let deck_id = Self::resolve_deck_id(&conn, deck_path)?;
        deck_repo::list_subdecks(&conn, deck_id)
    }

    pub fn get_cards(&self, deck_path: &str, query: CardQuery) -> Result<Vec<Card>, AnkiError> {
        let conn = self.conn()?;
        let deck_id = Self::resolve_deck_id(&conn, deck_path)?;
        card_repo::list_cards(&conn, deck_id, &query)
    }

    pub fn get_card(&self, card_id: &str) -> Result<Card, AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        card_repo::get_card(&conn, id)?.ok_or_else(|| Self::card_not_found(card_id))
    }

    pub fn search_cards(&self, keyword: &str, search: CardSearch) -> Result<Vec<Card>, AnkiError> {
        let conn = self.conn()?;
        card_repo::search_cards(&conn, keyword, &search)
    }

    pub fn create_deck(&self, deck_path: &str) -> Result<String, AnkiError> {
        let conn = self.conn()?;
        deck_repo::create_deck(&conn, deck_path)
    }

    pub fn create_card(&self, new_card: NewCard) -> Result<String, AnkiError> {
        let conn = self.conn()?;
        let deck_id = Self::resolve_deck_id(&conn, &new_card.deck_path)?;
        Self::ensure_concrete_deck(deck_id, &new_card.deck_path)?;
        let id = card_repo::create_card(&conn, deck_id, &new_card.front, &new_card.back)?;
        Ok(id.to_string())
    }

    pub fn move_deck(&self, source_path: &str, target_path: &str) -> Result<(), AnkiError> {
        let conn = self.conn()?;
        let source_id = Self::resolve_deck_id(&conn, source_path)?;
        let target_id = Self::resolve_deck_id(&conn, target_path)?;
        deck_repo::move_deck(&conn, source_id, target_id)
    }

    pub fn move_card(&self, card_id: &str, target_deck_path: &str) -> Result<(), AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        let target_deck_id = Self::resolve_deck_id(&conn, target_deck_path)?;
        Self::ensure_concrete_deck(target_deck_id, target_deck_path)?;
        card_repo::move_card(&conn, id, target_deck_id)
    }

    pub fn update_card_content(
        &self,
        card_id: &str,
        content: UpdateCardContent,
    ) -> Result<(), AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        card_repo::update_card_content(&conn, id, content.front, content.back)
    }

    pub fn delete_deck(&self, deck_path: &str) -> Result<(), AnkiError> {
        let conn = self.conn()?;
        let deck_id = Self::resolve_deck_id(&conn, deck_path)?;
        deck_repo::delete_deck(&conn, deck_id)
    }

    pub fn delete_card(&self, card_id: &str) -> Result<(), AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        card_repo::delete_card(&conn, id)
    }

    /// 对卡片作答（重来/困难/良好/简单），计算并持久化下一次复习安排。
    pub fn grade_card(&self, card_id: &str, grade: CardGrade) -> Result<ReviewOutcome, AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        let schedule =
            card_repo::find_schedule(&conn, id)?.ok_or_else(|| Self::card_not_found(card_id))?;
        let card_repo::ScheduleRecord {
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
        let next = card_repo::ScheduleRecord {
            state: decision.state,
            algorithm,
            scheduler_state: Some(serde_json::to_string(&decision.algorithm_state).map_err(
                |e| AnkiError {
                    code: "internal".into(),
                    message: format!("序列化调度状态失败: {e}"),
                },
            )?),
            due_at: Some(due_at.clone()),
        };
        card_repo::save_schedule(&conn, id, &next)?;

        Ok(ReviewOutcome {
            card_id: card_id.to_string(),
            state: decision.state,
            due_at: Some(due_at),
        })
    }

    /// 彻底忘记某张卡片：恢复到「刚新增」的初始记忆状态。
    pub fn reset_card(&self, card_id: &str) -> Result<ReviewOutcome, AnkiError> {
        let conn = self.conn()?;
        let id = Self::parse_id(card_id)?;
        let schedule =
            card_repo::find_schedule(&conn, id)?.ok_or_else(|| Self::card_not_found(card_id))?;
        let card_repo::ScheduleRecord { algorithm, .. } = schedule;

        let algorithm_impl = self.algorithm_for(&algorithm);
        let next = card_repo::ScheduleRecord {
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
        card_repo::save_schedule(&conn, id, &next)?;

        Ok(ReviewOutcome {
            card_id: card_id.to_string(),
            state: CardState::New,
            due_at: None,
        })
    }

    // ---------- 内部辅助 ----------

    fn algorithm_for(&self, name: &str) -> Arc<dyn SchedulingAlgorithm> {
        self.schedulers
            .get(name)
            .unwrap_or_else(|| self.schedulers.get(SM2).expect("默认算法 sm2 必须已注册"))
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
                serde_json::from_str(&json)
                    .map(Some)
                    .map_err(|e| AnkiError {
                        code: "invalid_state".into(),
                        message: format!("卡片 {card_id} 的调度状态无法解析: {e}"),
                    })
            }
            _ => Ok(None),
        }
    }

    fn card_not_found(card_id: &str) -> AnkiError {
        AnkiError {
            code: "not_found".into(),
            message: format!("卡片不存在: {card_id}"),
        }
    }

    fn deck_not_found(deck_path: &str) -> AnkiError {
        AnkiError {
            code: "not_found".into(),
            message: format!("牌组不存在: {deck_path}"),
        }
    }

    fn resolve_deck_id(conn: &rusqlite::Connection, deck_path: &str) -> Result<i64, AnkiError> {
        deck_repo::resolve_deck(conn, deck_path)?.ok_or_else(|| Self::deck_not_found(deck_path))
    }

    /// 卡片必须放在具体牌组下，不能放在根（id = 0）。
    fn ensure_concrete_deck(deck_id: i64, deck_path: &str) -> Result<(), AnkiError> {
        if deck_id == 0 {
            return Err(AnkiError {
                code: "invalid_path".into(),
                message: format!("卡片必须放在具体牌组下，不能放在根: {deck_path}"),
            });
        }
        Ok(())
    }
}
