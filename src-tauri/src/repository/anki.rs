//! 卡片相关的数据访问与调度持久化。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::anki::{AnkiError, Card, CardQuery, CardSearch, CardState};

use super::deck;
use super::map_rusqlite;

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
pub fn save_schedule(
    conn: &Connection,
    card_id: i64,
    record: &ScheduleRecord,
) -> Result<(), AnkiError> {
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

/// 卡片查询统一使用的列顺序。
const CARD_COLUMNS: &str = "id, deck_id, front, back, state, due_at, created_at, updated_at";

/// 卡片原始行（state 保持字符串，稍后解析）。
struct CardRow {
    id: i64,
    deck_id: i64,
    front: String,
    back: String,
    state: String,
    due_at: Option<String>,
    created_at: String,
    updated_at: String,
}

fn card_from_row(conn: &Connection, row: CardRow) -> Result<Card, AnkiError> {
    let deck_path = deck::deck_path(conn, row.deck_id)?;
    card_from_row_with_path(row, deck_path)
}

fn card_from_row_with_path(row: CardRow, deck_path: String) -> Result<Card, AnkiError> {
    Ok(Card {
        id: row.id.to_string(),
        deck_path,
        front: row.front,
        back: row.back,
        state: parse_state(&row.state)?,
        due_at: row.due_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

fn read_card_rows(
    conn: &Connection,
    sql: &str,
    params: Vec<Box<dyn rusqlite::ToSql>>,
) -> Result<Vec<CardRow>, AnkiError> {
    let mut stmt = conn.prepare(sql).map_err(map_rusqlite)?;
    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();
    let rows = stmt
        .query_map(param_refs.as_slice(), |row| {
            Ok(CardRow {
                id: row.get(0)?,
                deck_id: row.get(1)?,
                front: row.get(2)?,
                back: row.get(3)?,
                state: row.get(4)?,
                due_at: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(map_rusqlite)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_rusqlite)?);
    }
    Ok(out)
}

/// 按 id 读取单张卡片。
pub fn get_card(conn: &Connection, card_id: i64) -> Result<Option<Card>, AnkiError> {
    let row = conn
        .query_row(
            &format!("SELECT {CARD_COLUMNS} FROM card WHERE id = ?1"),
            [card_id],
            |row| {
                Ok(CardRow {
                    id: row.get(0)?,
                    deck_id: row.get(1)?,
                    front: row.get(2)?,
                    back: row.get(3)?,
                    state: row.get(4)?,
                    due_at: row.get(5)?,
                    created_at: row.get(6)?,
                    updated_at: row.get(7)?,
                })
            },
        )
        .optional()
        .map_err(map_rusqlite)?;
    match row {
        Some(row) => Ok(Some(card_from_row(conn, row)?)),
        None => Ok(None),
    }
}

/// 列出某牌组下的卡片，支持状态、到期时间、关键字与分页过滤。
pub fn list_cards(
    conn: &Connection,
    deck_id: i64,
    query: &CardQuery,
) -> Result<Vec<Card>, AnkiError> {
    let deck_path = deck::deck_path(conn, deck_id)?;

    let mut sql = format!("SELECT {CARD_COLUMNS} FROM card WHERE deck_id = ?");
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = vec![Box::new(deck_id)];

    if let Some(state) = query.state {
        sql.push_str(" AND state = ?");
        params.push(Box::new(state_to_str(state)));
    }
    if let Some(ref due_before) = query.due_before {
        sql.push_str(" AND due_at <= ?");
        params.push(Box::new(due_before.clone()));
    }
    if let Some(ref due_after) = query.due_after {
        sql.push_str(" AND due_at >= ?");
        params.push(Box::new(due_after.clone()));
    }
    if let Some(ref keyword) = query.keyword {
        sql.push_str(" AND (front LIKE ? OR back LIKE ?)");
        let pattern = format!("%{keyword}%");
        params.push(Box::new(pattern.clone()));
        params.push(Box::new(pattern));
    }
    sql.push_str(" ORDER BY id");
    if let Some(limit) = query.limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }
    if let Some(offset) = query.offset {
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset as i64));
    }

    let rows = read_card_rows(conn, &sql, params)?;
    rows.into_iter()
        .map(|row| card_from_row_with_path(row, deck_path.clone()))
        .collect()
}

/// 新建一张卡片（初始为 new 状态），返回新卡片 id。
pub fn create_card(
    conn: &Connection,
    deck_id: i64,
    front: &str,
    back: &str,
) -> Result<i64, AnkiError> {
    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO card (deck_id, front, back, state, due_at, created_at, updated_at)
         VALUES (?1, ?2, ?3, 'new', NULL, ?4, ?5)",
        rusqlite::params![deck_id, front, back, now, now],
    )
    .map_err(map_rusqlite)?;
    Ok(conn.last_insert_rowid())
}

/// 把卡片移动到目标牌组。
pub fn move_card(
    conn: &Connection,
    card_id: i64,
    target_deck_id: i64,
) -> Result<(), AnkiError> {
    let updated_at = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE card SET deck_id = ?1, updated_at = ?2 WHERE id = ?3",
        rusqlite::params![target_deck_id, updated_at, card_id],
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

/// 更新卡片正面或背面内容。
pub fn update_card_content(
    conn: &Connection,
    card_id: i64,
    front: Option<String>,
    back: Option<String>,
) -> Result<(), AnkiError> {
    if front.is_none() && back.is_none() {
        return Err(AnkiError {
            code: "invalid_input".into(),
            message: "没有需要更新的字段".into(),
        });
    }

    let mut sets: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();
    if let Some(front) = front {
        sets.push("front = ?".into());
        params.push(Box::new(front));
    }
    if let Some(back) = back {
        sets.push("back = ?".into());
        params.push(Box::new(back));
    }
    sets.push("updated_at = ?".into());
    params.push(Box::new(Utc::now().to_rfc3339()));

    let sql = format!("UPDATE card SET {} WHERE id = ?", sets.join(", "));
    params.push(Box::new(card_id));

    let param_refs: Vec<&dyn rusqlite::ToSql> =
        params.iter().map(|b| &**b as &dyn rusqlite::ToSql).collect();
    conn.execute(&sql, param_refs.as_slice()).map_err(map_rusqlite)?;
    if conn.changes() == 0 {
        return Err(AnkiError {
            code: "not_found".into(),
            message: format!("卡片不存在: {card_id}"),
        });
    }
    Ok(())
}

/// 删除单张卡片。
pub fn delete_card(conn: &Connection, card_id: i64) -> Result<(), AnkiError> {
    conn.execute("DELETE FROM card WHERE id = ?1", [card_id])
        .map_err(map_rusqlite)?;
    if conn.changes() == 0 {
        return Err(AnkiError {
            code: "not_found".into(),
            message: format!("卡片不存在: {card_id}"),
        });
    }
    Ok(())
}

/// 按关键字搜索卡片正面或背面，可按牌组限定范围。
pub fn search_cards(
    conn: &Connection,
    keyword: &str,
    search: &CardSearch,
) -> Result<Vec<Card>, AnkiError> {
    let mut clauses: Vec<String> = Vec::new();
    let mut params: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

    if let Some(ref deck_path) = search.deck_path {
        let deck_id = deck::resolve_deck(conn, deck_path)?.ok_or_else(|| AnkiError {
            code: "not_found".into(),
            message: format!("牌组不存在: {deck_path}"),
        })?;
        clauses.push("deck_id = ?".into());
        params.push(Box::new(deck_id));
    }

    let pattern = format!("%{keyword}%");
    let mut field_clauses: Vec<String> = Vec::new();
    if search.front {
        field_clauses.push("front LIKE ?".into());
        params.push(Box::new(pattern.clone()));
    }
    if search.back {
        field_clauses.push("back LIKE ?".into());
        params.push(Box::new(pattern.clone()));
    }
    if field_clauses.is_empty() {
        field_clauses.push("front LIKE ?".into());
        params.push(Box::new(pattern.clone()));
        field_clauses.push("back LIKE ?".into());
        params.push(Box::new(pattern));
    }
    clauses.push(format!("({})", field_clauses.join(" OR ")));

    let mut sql = format!(
        "SELECT {CARD_COLUMNS} FROM card WHERE {}",
        clauses.join(" AND ")
    );
    sql.push_str(" ORDER BY id");
    if let Some(limit) = search.limit {
        sql.push_str(" LIMIT ?");
        params.push(Box::new(limit as i64));
    }
    if let Some(offset) = search.offset {
        sql.push_str(" OFFSET ?");
        params.push(Box::new(offset as i64));
    }

    let rows = read_card_rows(conn, &sql, params)?;
    rows.into_iter().map(|row| card_from_row(conn, row)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::{db, deck};

    fn setup_card() -> (Connection, i64) {
        let conn = db::open_in_memory().unwrap();
        let deck_path = deck::create_deck(&conn, "/测试").unwrap();
        let deck_id = deck::resolve_deck(&conn, &deck_path).unwrap().unwrap();
        let card_id = create_card(&conn, deck_id, "front", "back").unwrap();
        (conn, card_id)
    }

    #[test]
    fn migration_contains_scheduling_columns() {
        let conn = db::open_in_memory().unwrap();
        let mut stmt = conn.prepare("PRAGMA table_info(card)").unwrap();
        let names: Vec<String> = stmt
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|row| row.unwrap())
            .collect();

        assert!(names.iter().any(|name| name == "algorithm"));
        assert!(names.iter().any(|name| name == "scheduler_state"));
    }

    #[test]
    fn schedule_round_trip_preserves_algorithm_state() {
        let (conn, card_id) = setup_card();
        let before = find_schedule(&conn, card_id).unwrap().unwrap();
        assert_eq!(before.state, CardState::New);
        assert_eq!(before.algorithm, "sm2");
        assert!(before.scheduler_state.is_none());

        let record = ScheduleRecord {
            state: CardState::Review,
            algorithm: "fsrs".into(),
            scheduler_state: Some(r#"{"difficulty":5.1,"stability":3.7}"#.into()),
            due_at: Some("2026-01-08T12:00:00+00:00".into()),
        };
        save_schedule(&conn, card_id, &record).unwrap();

        let after = find_schedule(&conn, card_id).unwrap().unwrap();
        assert_eq!(after.state, CardState::Review);
        assert_eq!(after.algorithm, "fsrs");
        assert_eq!(after.scheduler_state, record.scheduler_state);
        assert_eq!(after.due_at, record.due_at);
    }

    #[test]
    fn missing_card_and_state_round_trip_are_reported() {
        let (conn, card_id) = setup_card();
        assert!(find_schedule(&conn, 999999).unwrap().is_none());
        assert_eq!(save_schedule(&conn, 999999, &ScheduleRecord {
            state: CardState::New,
            algorithm: "sm2".into(),
            scheduler_state: None,
            due_at: None,
        }).unwrap_err().code, "not_found");

        let record = ScheduleRecord {
            state: CardState::Learning,
            algorithm: "sm2".into(),
            scheduler_state: Some("not-json-yet".into()),
            due_at: None,
        };
        save_schedule(&conn, card_id, &record).unwrap();
        let saved = find_schedule(&conn, card_id).unwrap().unwrap();
        assert_eq!(saved.state, CardState::Learning);
        assert_eq!(saved.scheduler_state.as_deref(), Some("not-json-yet"));
    }
}
