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

#[cfg(test)]
mod tests {
    use crate::repository::{anki, db};

    use super::*;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    fn create_card(
        conn: &Connection,
        deck_id: i64,
        front: &str,
        back: &str,
    ) -> Result<i64, AnkiError> {
        anki::create_card_with_algorithm(conn, deck_id, front, back, "sm2")
    }

    /// 创建牌组并返回 (id, 规范化路径)。
    fn make_deck(conn: &Connection, path: &str) -> (i64, String) {
        let normalized = create_deck(conn, path).unwrap();
        let id = resolve_deck(conn, &normalized).unwrap().unwrap();
        (id, normalized)
    }

    fn deck_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM deck", [], |row| row.get(0))
            .unwrap()
    }

    fn card_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM card", [], |row| row.get(0))
            .unwrap()
    }

    #[test]
    fn split_path_ignores_empty_and_redundant_slashes() {
        assert!(split_path("").is_empty());
        assert!(split_path("/").is_empty());
        assert!(split_path("///").is_empty());
        assert_eq!(split_path("/a/b/c"), vec!["a", "b", "c"]);
        assert_eq!(split_path("a//b//"), vec!["a", "b"]);
        assert_eq!(split_path("/中文/牌组"), vec!["中文", "牌组"]);
    }

    #[test]
    fn resolve_deck_maps_root_and_missing_paths() {
        let conn = setup();
        assert_eq!(resolve_deck(&conn, "/").unwrap(), Some(0));
        assert_eq!(resolve_deck(&conn, "").unwrap(), Some(0));
        assert_eq!(resolve_deck(&conn, "///").unwrap(), Some(0));

        let (_, path) = make_deck(&conn, "/a/b");
        assert!(resolve_deck(&conn, &path).unwrap().is_some());
        assert!(resolve_deck(&conn, "/a//b/").unwrap().is_some());

        // 部分路径存在时不应命中更深的路径。
        assert_eq!(resolve_deck(&conn, "/a/b/c").unwrap(), None);
        assert_eq!(resolve_deck(&conn, "/missing").unwrap(), None);
        assert_eq!(resolve_deck(&conn, "/a/x/b").unwrap(), None);
    }

    #[test]
    fn create_deck_normalizes_and_is_idempotent() {
        let conn = setup();
        assert_eq!(create_deck(&conn, "/a//b/").unwrap(), "/a/b");
        assert_eq!(create_deck(&conn, "a/b").unwrap(), "/a/b");
        assert_eq!(create_deck(&conn, "/a/b").unwrap(), "/a/b");
        assert_eq!(deck_count(&conn), 2);

        let (_, parent_path) = make_deck(&conn, "/a");
        assert_eq!(parent_path, "/a");
        assert_eq!(deck_count(&conn), 2);
    }

    #[test]
    fn create_deck_rejects_empty_paths() {
        let conn = setup();
        assert_eq!(create_deck(&conn, "").unwrap_err().code, "invalid_path");
        assert_eq!(create_deck(&conn, "/").unwrap_err().code, "invalid_path");
        assert_eq!(create_deck(&conn, "///").unwrap_err().code, "invalid_path");
        assert_eq!(deck_count(&conn), 0);
    }

    #[test]
    fn deck_path_round_trips_and_reports_errors() {
        let conn = setup();
        assert_eq!(deck_path(&conn, 0).unwrap(), "/");

        let (id, _) = make_deck(&conn, "/a/b/c");
        assert_eq!(deck_path(&conn, id).unwrap(), "/a/b/c");

        assert_eq!(deck_path(&conn, 999_999).unwrap_err().code, "not_found");
    }

    #[test]
    fn deck_path_detects_cycles() {
        let conn = setup();
        let (a, _) = make_deck(&conn, "/a");
        let (b, _) = make_deck(&conn, "/a/b");

        // parent_id 没有外键约束，可直接制造环来验证防护逻辑。
        conn.execute(
            "UPDATE deck SET parent_id = ?1 WHERE id = ?2",
            rusqlite::params![b, a],
        )
        .unwrap();

        assert_eq!(deck_path(&conn, a).unwrap_err().code, "internal");
    }

    #[test]
    fn list_subdecks_orders_by_name_and_counts() {
        let conn = setup();
        let parent = make_deck(&conn, "/parent").0;
        let child = make_deck(&conn, "/parent/child").0;
        make_deck(&conn, "/parent/child/grand");
        let other = make_deck(&conn, "/parent/other").0;
        create_card(&conn, child, "f1", "b1").unwrap();
        create_card(&conn, child, "f2", "b2").unwrap();

        let subs = list_subdecks(&conn, parent).unwrap();
        assert_eq!(subs.len(), 2);
        assert_eq!(subs[0].name, "child");
        assert_eq!(subs[0].path, "/parent/child");
        assert_eq!(subs[0].card_count, 2);
        assert_eq!(subs[0].subdeck_count, 1);
        assert!(!subs[0].created_at.is_empty());
        assert_eq!(subs[1].name, "other");
        assert_eq!(subs[1].path, "/parent/other");
        assert_eq!(subs[1].card_count, 0);
        assert_eq!(subs[1].subdeck_count, 0);

        // 叶子牌组没有子牌组，根下能列出 /parent。
        assert!(list_subdecks(&conn, other).unwrap().is_empty());
        assert!(list_subdecks(&conn, 999_999).unwrap().is_empty());
        let roots = list_subdecks(&conn, 0).unwrap();
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].path, "/parent");
    }

    #[test]
    fn move_deck_rejects_invalid_targets() {
        let conn = setup();
        let (a, _) = make_deck(&conn, "/a");
        make_deck(&conn, "/a/b");
        let (c, _) = make_deck(&conn, "/a/b/c");

        assert_eq!(move_deck(&conn, 0, c).unwrap_err().code, "invalid_path");
        assert_eq!(move_deck(&conn, a, a).unwrap_err().code, "invalid_path");
        // 移动到直接子牌组与更深的后代都应被拒绝。
        assert_eq!(move_deck(&conn, a, c).unwrap_err().code, "invalid_path");

        assert_eq!(move_deck(&conn, a, 999_999).unwrap_err().code, "not_found");
        assert_eq!(move_deck(&conn, 888_888, 0).unwrap_err().code, "not_found");
    }

    #[test]
    fn move_deck_reports_name_conflict() {
        let conn = setup();
        let one = make_deck(&conn, "/one").0;
        let source = make_deck(&conn, "/two/a").0;
        make_deck(&conn, "/one/a");

        assert_eq!(move_deck(&conn, source, one).unwrap_err().code, "conflict");
        assert_eq!(deck_path(&conn, source).unwrap(), "/two/a");
    }

    #[test]
    fn move_deck_rejects_card_as_target() {
        let conn = setup();
        let source = make_deck(&conn, "/source").0;
        let host = make_deck(&conn, "/host").0;

        // 卡片 id 与牌组 id 属于不同的表且各自自增。多建几张卡片，取一个
        // 不对应任何牌组的卡片 id，确保测试的是“目标是卡片”而非 id 恰好撞车。
        let mut card_id = 0;
        for _ in 0..3 {
            card_id = create_card(&conn, host, "front", "back").unwrap();
        }
        let deck_exists: bool = conn
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM deck WHERE id = ?1)",
                [card_id],
                |row| row.get(0),
            )
            .unwrap();
        assert!(!deck_exists);

        // 卡片不能作为父级，因此把卡片 id 当作目标时应视为目标牌组不存在。
        assert_eq!(
            move_deck(&conn, source, card_id).unwrap_err().code,
            "not_found"
        );
        assert_eq!(deck_path(&conn, source).unwrap(), "/source");
    }

    #[test]
    fn move_deck_updates_parent_and_path() {
        let conn = setup();
        let b = make_deck(&conn, "/b").0;
        let child = make_deck(&conn, "/a/child").0;

        move_deck(&conn, child, b).unwrap();
        assert_eq!(deck_path(&conn, child).unwrap(), "/b/child");

        // target_id = 0 表示移动到根下。
        move_deck(&conn, child, 0).unwrap();
        assert_eq!(deck_path(&conn, child).unwrap(), "/child");
    }

    #[test]
    fn delete_deck_removes_subtree_and_cards() {
        let conn = setup();
        let root = make_deck(&conn, "/root").0;
        let child = make_deck(&conn, "/root/child").0;
        make_deck(&conn, "/root/child/grand");
        let sibling = make_deck(&conn, "/sibling").0;
        create_card(&conn, root, "rf", "rb").unwrap();
        create_card(&conn, child, "cf", "cb").unwrap();
        create_card(&conn, sibling, "sf", "sb").unwrap();

        delete_deck(&conn, root).unwrap();

        // 整个子树都没了，卡片通过外键级联删除。
        assert_eq!(deck_count(&conn), 1);
        assert_eq!(card_count(&conn), 1);
        assert!(deck_path(&conn, root).is_err());
        assert!(deck_path(&conn, child).is_err());
        // 兄弟牌组与其卡片保留。
        assert_eq!(deck_path(&conn, sibling).unwrap(), "/sibling");
    }

    #[test]
    fn delete_deck_rejects_root_and_missing() {
        let conn = setup();
        make_deck(&conn, "/keep");

        assert_eq!(delete_deck(&conn, 0).unwrap_err().code, "invalid_path");
        assert_eq!(delete_deck(&conn, 999_999).unwrap_err().code, "not_found");
        assert_eq!(deck_count(&conn), 1);
    }
}
