//! 卡片调度相关的数据访问。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::anki::{AnkiError, CardState};

/// 卡片调度相关的持久化记录。
pub struct ScheduleRecord {
    pub state: CardState,
    pub algorithm: String,
    pub scheduler_state: Option<String>,
    pub due_at: Option<String>,
}

/// 查询用到的原始行（字符串形式，解析放在查询之后）。
struct RawSchedule {
    state: String,
    algorithm: String,
    scheduler_state: Option<String>,
    due_at: Option<String>,
}

fn map_rusqlite(e: rusqlite::Error) -> AnkiError {
    AnkiError {
        code: "db".into(),
        message: e.to_string(),
    }
}

fn parse_state(raw: &str) -> Result<CardState, AnkiError> {
    match raw {
        "new" => Ok(CardState::New),
        "learning" => Ok(CardState::Learning),
        "review" => Ok(CardState::Review),
        "relearning" => Ok(CardState::Relearning),
        other => Err(AnkiError {
            code: "invalid_state".into(),
            message: format!("未知的卡片状态: {other}"),
        }),
    }
}

fn state_to_str(state: CardState) -> &'static str {
    match state {
        CardState::New => "new",
        CardState::Learning => "learning",
        CardState::Review => "review",
        CardState::Relearning => "relearning",
    }
}

/// 读取一张卡片的调度记录。
pub fn find_schedule(conn: &Connection, card_id: i64) -> Result<Option<ScheduleRecord>, AnkiError> {
    let raw = conn
        .query_row(
            "SELECT state, algorithm, scheduler_state, due_at FROM card WHERE id = ?1",
            [card_id],
            |row| {
                Ok(RawSchedule {
                    state: row.get(0)?,
                    algorithm: row.get(1)?,
                    scheduler_state: row.get(2)?,
                    due_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(map_rusqlite)?;

    match raw {
        Some(raw) => Ok(Some(ScheduleRecord {
            state: parse_state(&raw.state)?,
            algorithm: raw.algorithm,
            scheduler_state: raw.scheduler_state,
            due_at: raw.due_at,
        })),
        None => Ok(None),
    }
}

/// 持久化一张卡片的调度记录，并刷新 updated_at。
pub fn save_schedule(conn: &Connection, card_id: i64, record: &ScheduleRecord) -> Result<(), AnkiError> {
    let state = state_to_str(record.state);
    let updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE card
            SET state = ?1, due_at = ?2, algorithm = ?3, scheduler_state = ?4, updated_at = ?5
          WHERE id = ?6",
        rusqlite::params![
            state,
            record.due_at,
            record.algorithm,
            record.scheduler_state,
            updated_at,
            card_id
        ],
    )
    .map_err(map_rusqlite)?;

    if conn.changes() == 0 {
        return Err(AnkiError {
            code: "not_found".into(),
            message: format!("卡片不存在: {card_id}"),
        });
    }
    Ok(())
}
