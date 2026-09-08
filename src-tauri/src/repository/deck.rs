//! 牌组相关的数据访问与路径解析。

use std::collections::HashSet;

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::anki::{AnkiError, Deck};

use super::map_rusqlite;

/// 把牌组路径拆成各层名字（忽略空段与多余斜杠）。
pub fn split_path(path: &str) -> Vec<String> {
    path.split('/')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect()
}

/// 根据牌组路径解析出牌组 id；路径 "/" 对应根（id = 0）。
pub fn resolve_deck(conn: &Connection, path: &str) -> Result<Option<i64>, AnkiError> {
    let mut parent = 0i64;
    for name in split_path(path) {
        let row: Option<i64> = conn
            .query_row(
                "SELECT id FROM deck WHERE parent_id = ?1 AND name = ?2",
                rusqlite::params![parent, name],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(map_rusqlite)?;
        match row {
            Some(id) => parent = id,
            None => return Ok(None),
        }
    }
    Ok(Some(parent))
}

/// 根据牌组 id 反推完整路径（从根到该牌组）。
pub fn deck_path(conn: &Connection, deck_id: i64) -> Result<String, AnkiError> {
    let mut names: Vec<String> = Vec::new();
    let mut cur = deck_id;
    let mut visited = HashSet::new();
    while cur != 0 {
        if !visited.insert(cur) {
            return Err(AnkiError {
                code: "internal".into(),
                message: format!("牌组层级出现环: {deck_id}"),
            });
        }
        let row = conn
            .query_row(
                "SELECT parent_id, name FROM deck WHERE id = ?1",
                [cur],
                |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
            )
            .optional()
            .map_err(map_rusqlite)?;
        match row {
            Some((parent, name)) => {
                names.push(name);
                cur = parent;
            }
            None => {
                return Err(AnkiError {
                    code: "not_found".into(),
                    message: format!("牌组不存在: {deck_id}"),
                })
            }
        }
    }
    names.reverse();
    Ok(format!("/{}", names.join("/")))
}

fn count_cards(conn: &Connection, deck_id: i64) -> Result<u32, AnkiError> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM card WHERE deck_id = ?1",
            [deck_id],
            |row| row.get(0),
        )
        .map_err(map_rusqlite)?;
    Ok(n as u32)
}

fn count_subdecks(conn: &Connection, deck_id: i64) -> Result<u32, AnkiError> {
    let n: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deck WHERE parent_id = ?1",
            [deck_id],
            |row| row.get(0),
        )
        .map_err(map_rusqlite)?;
    Ok(n as u32)
}

/// 列出某牌组的直接子牌组。
pub fn list_subdecks(conn: &Connection, deck_id: i64) -> Result<Vec<Deck>, AnkiError> {
    let mut stmt = conn
        .prepare("SELECT id, name, created_at FROM deck WHERE parent_id = ?1 ORDER BY name")
        .map_err(map_rusqlite)?;
    let rows = stmt
        .query_map([deck_id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })
        .map_err(map_rusqlite)?;

    let mut decks = Vec::new();
    for row in rows {
        let (id, name, created_at) = row.map_err(map_rusqlite)?;
        decks.push(Deck {
            path: deck_path(conn, id)?,
            name,
            card_count: count_cards(conn, id)?,
            subdeck_count: count_subdecks(conn, id)?,
            created_at,
        });
    }
    Ok(decks)
}

/// 创建（或确保存在）指定路径的牌组，返回规范化后的路径。
pub fn create_deck(conn: &Connection, path: &str) -> Result<String, AnkiError> {
    let segments = split_path(path);
    if segments.is_empty() {
        return Err(AnkiError {
            code: "invalid_path".into(),
            message: "牌组路径不能为空".into(),
        });
    }

    let mut parent = 0i64;
    for name in &segments {
        let existing = conn
            .query_row(
                "SELECT id FROM deck WHERE parent_id = ?1 AND name = ?2",
                rusqlite::params![parent, name],
                |row| row.get::<_, i64>(0),
            )
            .optional()
            .map_err(map_rusqlite)?;
        parent = match existing {
            Some(id) => id,
            None => {
                let created_at = Utc::now().to_rfc3339();
                conn.execute(
                    "INSERT INTO deck (parent_id, name, created_at) VALUES (?1, ?2, ?3)",
                    rusqlite::params![parent, name, created_at],
                )
                .map_err(map_rusqlite)?;
                conn.last_insert_rowid()
            }
        };
    }

    Ok(format!("/{}", segments.join("/")))
}

fn is_descendant(conn: &Connection, ancestor: i64, node: i64) -> Result<bool, AnkiError> {
    let mut cur = node;
    let mut visited = HashSet::new();
    while cur != 0 {
        if cur == ancestor {
            return Ok(true);
        }
        if !visited.insert(cur) {
            break;
        }
        let parent = conn
            .query_row("SELECT parent_id FROM deck WHERE id = ?1", [cur], |row| {
                row.get::<_, i64>(0)
            })
            .optional()
            .map_err(map_rusqlite)?
            .unwrap_or(0);
        cur = parent;
    }
    Ok(false)
}

/// 把牌组移动到新的父牌组下（target_id 为 0 表示移到根下）。
pub fn move_deck(conn: &Connection, source_id: i64, target_id: i64) -> Result<(), AnkiError> {
    if source_id == 0 {
        return Err(AnkiError {
            code: "invalid_path".into(),
            message: "根牌组不可移动".into(),
        });
    }
    if source_id == target_id {
        return Err(AnkiError {
            code: "invalid_path".into(),
            message: "不能把牌组移动到自身".into(),
        });
    }
    if is_descendant(conn, source_id, target_id)? {
        return Err(AnkiError {
            code: "invalid_path".into(),
            message: "不能把牌组移动到它的子牌组下".into(),
        });
    }

    if target_id != 0 {
        let target_exists = conn
            .query_row("SELECT 1 FROM deck WHERE id = ?1", [target_id], |_| Ok(()))
            .optional()
            .map_err(map_rusqlite)?;
        if target_exists.is_none() {
            return Err(AnkiError {
                code: "not_found".into(),
                message: format!("目标牌组不存在: {target_id}"),
            });
        }
    }

    let source_name: String = conn
        .query_row("SELECT name FROM deck WHERE id = ?1", [source_id], |row| {
            row.get(0)
        })
        .optional()
        .map_err(map_rusqlite)?
        .ok_or_else(|| AnkiError {
            code: "not_found".into(),
            message: format!("牌组不存在: {source_id}"),
        })?;

    let clash: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM deck WHERE parent_id = ?1 AND name = ?2 AND id != ?3",
            rusqlite::params![target_id, source_name, source_id],
            |row| row.get(0),
        )
        .map_err(map_rusqlite)?;
    if clash > 0 {
        return Err(AnkiError {
            code: "conflict".into(),
            message: "目标牌组下已存在同名子牌组".into(),
        });
    }

    conn.execute(
        "UPDATE deck SET parent_id = ?1 WHERE id = ?2",
        rusqlite::params![target_id, source_id],
    )
    .map_err(map_rusqlite)?;
    Ok(())
}

/// 删除牌组及其所有子牌组（卡片通过外键级联删除）。
pub fn delete_deck(conn: &Connection, deck_id: i64) -> Result<(), AnkiError> {
    if deck_id == 0 {
        return Err(AnkiError {
            code: "invalid_path".into(),
            message: "根牌组不可删除".into(),
        });
    }

    let child_ids: Vec<i64> = {
        let mut stmt = conn
            .prepare("SELECT id FROM deck WHERE parent_id = ?1")
            .map_err(map_rusqlite)?;
        let rows = stmt
            .query_map([deck_id], |row| row.get::<_, i64>(0))
            .map_err(map_rusqlite)?;
        let mut ids = Vec::new();
        for row in rows {
            ids.push(row.map_err(map_rusqlite)?);
        }
        ids
    };
    for child_id in child_ids {
        delete_deck(conn, child_id)?;
    }

    conn.execute("DELETE FROM deck WHERE id = ?1", [deck_id])
        .map_err(map_rusqlite)?;
    if conn.changes() == 0 {
        return Err(AnkiError {
            code: "not_found".into(),
            message: format!("牌组不存在: {deck_id}"),
        });
    }
    Ok(())
}
