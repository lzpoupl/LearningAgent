//! 卡片统计的聚合查询。
//!
//! 本模块只做条件计数与按天聚合，不承担业务口径：日期补零、百分比与
//! 平均值计算都在 `service/statistics.rs` 完成。

use rusqlite::Connection;

use crate::interface::anki::AnkiError;

use super::anki::today_local;
use super::map_rusqlite;

/// 四个卡片分类的原始计数，与 `CardState` 一一对应。
#[derive(Clone, Copy, Debug, Default)]
pub struct CategoryCounts {
    pub new: u32,
    pub learning: u32,
    pub review: u32,
    pub relearning: u32,
}

impl CategoryCounts {
    /// 全部卡片数量。
    pub fn total(&self) -> u32 {
        self.new + self.learning + self.review + self.relearning
    }
}

/// 今日学习统计所需的原始计数。
#[derive(Clone, Copy, Debug, Default)]
pub struct TodayCounts {
    pub reviewed_cards: u32,
    pub new_remaining: u32,
    pub due_remaining: u32,
    pub total_cards: u32,
}

/// 统计各分类的卡片数量；四个分类互斥且覆盖全表。
pub fn category_counts(conn: &Connection) -> Result<CategoryCounts, AnkiError> {
    conn.query_row(
        "SELECT
            COALESCE(SUM(CASE WHEN state = 'new' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN state = 'learning' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN state = 'review' THEN 1 ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN state = 'relearning' THEN 1 ELSE 0 END), 0)
         FROM card",
        [],
        |row| {
            Ok(CategoryCounts {
                new: row.get(0)?,
                learning: row.get(1)?,
                review: row.get(2)?,
                relearning: row.get(3)?,
            })
        },
    )
    .map_err(map_rusqlite)
}

/// 今日进度：已作答的卡片数、仍需作答的卡片数与卡片总数。
///
/// `now` 为 RFC3339 时间串，用于判断是否到期；「今日」按本地时区日期取。
pub fn today_counts(conn: &Connection, now: &str) -> Result<TodayCounts, AnkiError> {
    let (new_remaining, due_remaining, total_cards) = conn
        .query_row(
            "SELECT
                COALESCE(SUM(CASE WHEN state = 'new' THEN 1 ELSE 0 END), 0),
                COALESCE(SUM(CASE WHEN state <> 'new' AND due_at IS NOT NULL
                                   AND datetime(due_at) <= datetime(?1) THEN 1 ELSE 0 END), 0),
                COUNT(*)
             FROM card",
            [now],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .map_err(map_rusqlite)?;

    let reviewed_cards: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT card_id) FROM review_log WHERE review_date = ?1",
            [today_local()],
            |row| row.get(0),
        )
        .map_err(map_rusqlite)?;

    Ok(TodayCounts {
        reviewed_cards: reviewed_cards as u32,
        new_remaining,
        due_remaining,
        total_cards,
    })
}

/// 指定日期（含）之后的每日复习次数，按日期升序。
pub fn review_counts_by_day(
    conn: &Connection,
    from: &str,
) -> Result<Vec<(String, u32)>, AnkiError> {
    let mut stmt = conn
        .prepare(
            "SELECT review_date, COUNT(*) FROM review_log
              WHERE review_date >= ?1
              GROUP BY review_date
              ORDER BY review_date",
        )
        .map_err(map_rusqlite)?;
    let rows = stmt
        .query_map([from], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })
        .map_err(map_rusqlite)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_rusqlite)?);
    }
    Ok(out)
}

/// 指定日期（含）之后有作答记录的天数。
pub fn studied_day_count(conn: &Connection, from: &str) -> Result<u32, AnkiError> {
    let days: i64 = conn
        .query_row(
            "SELECT COUNT(DISTINCT review_date) FROM review_log WHERE review_date >= ?1",
            [from],
            |row| row.get(0),
        )
        .map_err(map_rusqlite)?;
    Ok(days as u32)
}

/// 指定日期（含）之后的每日新增卡片数量，按日期升序。
///
/// `created_at` 以 UTC 存储，这里折算为本地日期，与统计口径一致。
pub fn added_counts_by_day(
    conn: &Connection,
    from: &str,
) -> Result<Vec<(String, u32)>, AnkiError> {
    let mut stmt = conn
        .prepare(
            "SELECT date(created_at, 'localtime') AS day, COUNT(*) FROM card
              WHERE date(created_at, 'localtime') >= ?1
              GROUP BY day
              ORDER BY day",
        )
        .map_err(map_rusqlite)?;
    let rows = stmt
        .query_map([from], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as u32))
        })
        .map_err(map_rusqlite)?;

    let mut out = Vec::new();
    for row in rows {
        out.push(row.map_err(map_rusqlite)?);
    }
    Ok(out)
}

/// 首个有数据的日期：复习记录与卡片新增中较早的一天。
pub fn first_activity_day(conn: &Connection) -> Result<Option<String>, AnkiError> {
    conn.query_row(
        "SELECT MIN(day) FROM (
            SELECT review_date AS day FROM review_log
            UNION ALL
            SELECT date(created_at, 'localtime') AS day FROM card
         )",
        [],
        |row| row.get(0),
    )
    .map_err(map_rusqlite)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::anki::{CardGrade, CardState};
    use crate::repository::{anki as card_repo, db, deck};

    /// 建一个牌组并返回 (连接, 牌组 id)。
    fn setup() -> (Connection, i64) {
        let conn = db::open_in_memory().unwrap();
        let path = deck::create_deck(&conn, "/统计").unwrap();
        let deck_id = deck::resolve_deck(&conn, &path).unwrap().unwrap();
        (conn, deck_id)
    }

    fn add_card(conn: &Connection, deck_id: i64, state: &str) -> i64 {
        let id = card_repo::create_card_with_algorithm(conn, deck_id, "f", "b", "sm2").unwrap();
        conn.execute(
            "UPDATE card SET state = ?1 WHERE id = ?2",
            rusqlite::params![state, id],
        )
        .unwrap();
        id
    }

    #[test]
    fn category_counts_lists_four_states() {
        let (conn, deck_id) = setup();
        add_card(&conn, deck_id, "new");
        add_card(&conn, deck_id, "learning");
        add_card(&conn, deck_id, "relearning");
        add_card(&conn, deck_id, "review");
        add_card(&conn, deck_id, "review");

        let counts = category_counts(&conn).unwrap();
        assert_eq!(counts.new, 1);
        assert_eq!(counts.learning, 1);
        assert_eq!(counts.relearning, 1);
        assert_eq!(counts.review, 2);
        assert_eq!(counts.total(), 5);
    }

    #[test]
    fn today_counts_separates_new_and_due_cards() {
        let (conn, deck_id) = setup();
        add_card(&conn, deck_id, "new");
        let due = add_card(&conn, deck_id, "review");
        let future = add_card(&conn, deck_id, "review");
        conn.execute(
            "UPDATE card SET due_at = '2020-01-01T00:00:00+00:00' WHERE id = ?1",
            [due],
        )
        .unwrap();
        conn.execute(
            "UPDATE card SET due_at = '2100-01-01T00:00:00+00:00' WHERE id = ?1",
            [future],
        )
        .unwrap();

        let counts = today_counts(&conn, "2026-01-01T00:00:00+00:00").unwrap();
        assert_eq!(counts.new_remaining, 1);
        assert_eq!(counts.due_remaining, 1);
        assert_eq!(counts.total_cards, 3);
        assert_eq!(counts.reviewed_cards, 0);
    }

    #[test]
    fn review_history_aggregates_by_local_day() {
        let (conn, deck_id) = setup();
        let card_id = add_card(&conn, deck_id, "review");
        for day in ["2025-12-31", "2026-01-01", "2026-01-01"] {
            card_repo::insert_review_log(
                &conn,
                &card_repo::ReviewLogEntry {
                    card_id,
                    deck_id,
                    grade: CardGrade::Good,
                    prev_state: CardState::Review,
                    next_state: CardState::Review,
                    duration_ms: 0,
                    reviewed_at: format!("{day}T04:00:00+00:00"),
                    review_date: day.to_string(),
                },
            )
            .unwrap();
        }

        let days = review_counts_by_day(&conn, "2026-01-01").unwrap();
        assert_eq!(days, vec![("2026-01-01".to_string(), 2)]);
        assert_eq!(studied_day_count(&conn, "2025-12-31").unwrap(), 2);
        assert_eq!(studied_day_count(&conn, "2026-01-02").unwrap(), 0);
    }

    #[test]
    fn added_counts_use_local_day_and_first_activity_is_earliest() {
        let (conn, deck_id) = setup();
        let card_id = add_card(&conn, deck_id, "new");
        conn.execute(
            "UPDATE card SET created_at = '2026-01-01T02:00:00+00:00' WHERE id = ?1",
            [card_id],
        )
        .unwrap();

        let added = added_counts_by_day(&conn, "2026-01-01").unwrap();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].1, 1);
        assert_eq!(added[0].0, "2026-01-01", "UTC 时间按本地日期折算");

        assert!(added_counts_by_day(&conn, "2026-02-01").unwrap().is_empty());
        assert!(first_activity_day(&conn).unwrap().is_some());
    }

    #[test]
    fn empty_database_reports_zero_counts() {
        let conn = db::open_in_memory().unwrap();

        let counts = category_counts(&conn).unwrap();
        assert_eq!(counts.total(), 0);
        let today = today_counts(&conn, "2026-01-01T00:00:00+00:00").unwrap();
        assert_eq!(today.total_cards, 0);
        assert_eq!(today.reviewed_cards, 0);
        assert!(first_activity_day(&conn).unwrap().is_none());
    }
}