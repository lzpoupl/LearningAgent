//! bucket 表的 SQL CRUD。
//!
//! 资产本体与元数据都来自文件系统，这里只维护「名称 -> 目录」映射；
//! `asset_count` 由 service 层用 `fs::count` 填充。

use chrono::Utc;
use rusqlite::{Connection, OptionalExtension};

use crate::interface::asset::Bucket;
use crate::interface::error::ApiError;

const BUCKET_COLUMNS: &str = "id, name, root_path, created_at, updated_at";

/// 列出全部 bucket，按 id 升序。
pub fn list_buckets(conn: &Connection) -> Result<Vec<Bucket>, ApiError> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {BUCKET_COLUMNS} FROM bucket ORDER BY id"
    ))?;
    let rows = stmt.query_map([], bucket_from_row)?;
    let mut buckets = Vec::new();
    for row in rows {
        buckets.push(row?);
    }
    Ok(buckets)
}

/// 按 id 读取 bucket。
pub fn get_bucket(conn: &Connection, bucket_id: i64) -> Result<Option<Bucket>, ApiError> {
    conn.query_row(
        &format!("SELECT {BUCKET_COLUMNS} FROM bucket WHERE id = ?1"),
        [bucket_id],
        bucket_from_row,
    )
    .optional()
    .map_err(ApiError::from)
}

/// 按名称读取 bucket。
pub fn find_bucket_by_name(conn: &Connection, name: &str) -> Result<Option<Bucket>, ApiError> {
    conn.query_row(
        &format!("SELECT {BUCKET_COLUMNS} FROM bucket WHERE name = ?1"),
        [name],
        bucket_from_row,
    )
    .optional()
    .map_err(ApiError::from)
}

/// 创建 bucket；名称重复返回 `conflict`。
pub fn create_bucket(conn: &Connection, name: &str, root_path: &str) -> Result<Bucket, ApiError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(ApiError::invalid_input("bucket 名称不能为空"));
    }
    if root_path.trim().is_empty() {
        return Err(ApiError::invalid_input("bucket 目录不能为空"));
    }
    if find_bucket_by_name(conn, name)?.is_some() {
        return Err(ApiError::conflict(format!("bucket 名称已存在: {name}")));
    }

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO bucket (name, root_path, created_at, updated_at) VALUES (?1, ?2, ?3, ?3)",
        rusqlite::params![name, root_path, now],
    )?;
    get_bucket(conn, conn.last_insert_rowid())?
        .ok_or_else(|| ApiError::internal("新建 bucket 后无法读取"))
}

/// 局部更新 bucket；`None` 表示沿用原值。
pub fn update_bucket(
    conn: &Connection,
    bucket_id: i64,
    name: Option<String>,
    root_path: Option<String>,
) -> Result<Bucket, ApiError> {
    let current = get_bucket(conn, bucket_id)?
        .ok_or_else(|| ApiError::not_found(format!("bucket 不存在: {bucket_id}")))?;

    let next_name = match name {
        Some(raw) => {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() {
                return Err(ApiError::invalid_input("bucket 名称不能为空"));
            }
            if let Some(other) = find_bucket_by_name(conn, &trimmed)? {
                if other.id != bucket_id {
                    return Err(ApiError::conflict(format!("bucket 名称已存在: {trimmed}")));
                }
            }
            trimmed
        }
        None => current.name,
    };

    let next_root = match root_path {
        Some(raw) => {
            if raw.trim().is_empty() {
                return Err(ApiError::invalid_input("bucket 目录不能为空"));
            }
            raw
        }
        None => current.root_path,
    };

    let now = Utc::now().to_rfc3339();
    conn.execute(
        "UPDATE bucket SET name = ?1, root_path = ?2, updated_at = ?3 WHERE id = ?4",
        rusqlite::params![next_name, next_root, now, bucket_id],
    )?;

    get_bucket(conn, bucket_id)?.ok_or_else(|| ApiError::internal("更新 bucket 后无法读取"))
}

/// 删除 bucket 映射；不触碰文件系统上的真实目录。
pub fn delete_bucket(conn: &Connection, bucket_id: i64) -> Result<(), ApiError> {
    conn.execute("DELETE FROM bucket WHERE id = ?1", [bucket_id])?;
    if conn.changes() == 0 {
        return Err(ApiError::not_found(format!("bucket 不存在: {bucket_id}")));
    }
    Ok(())
}

fn bucket_from_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Bucket> {
    Ok(Bucket {
        id: row.get(0)?,
        name: row.get(1)?,
        root_path: row.get(2)?,
        asset_count: 0,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::db;

    fn setup() -> Connection {
        db::open_in_memory().unwrap()
    }

    #[test]
    fn bucket_crud_round_trip() {
        let conn = setup();
        assert!(list_buckets(&conn).unwrap().is_empty());

        let bucket = create_bucket(&conn, "资料", "C:\\Users\\me\\资料").unwrap();
        assert_eq!(bucket.asset_count, 0);
        assert_eq!(bucket.root_path, "C:\\Users\\me\\资料");
        assert!(find_bucket_by_name(&conn, "资料").unwrap().is_some());
        assert_eq!(get_bucket(&conn, bucket.id).unwrap().unwrap().id, bucket.id);

        let renamed = update_bucket(&conn, bucket.id, Some("课程资料".into()), None).unwrap();
        assert_eq!(renamed.name, "课程资料");
        assert_eq!(renamed.root_path, "C:\\Users\\me\\资料");

        create_bucket(&conn, "笔记", "/tmp/notes").unwrap();
        let moved = update_bucket(&conn, bucket.id, None, Some("/tmp/course".into())).unwrap();
        assert_eq!(moved.root_path, "/tmp/course");

        delete_bucket(&conn, bucket.id).unwrap();
        assert!(get_bucket(&conn, bucket.id).unwrap().is_none());
    }

    #[test]
    fn bucket_reports_conflicts_and_missing() {
        let conn = setup();
        create_bucket(&conn, "资料", "/tmp/a").unwrap();

        assert_eq!(
            create_bucket(&conn, "资料", "/tmp/b").unwrap_err().code,
            "conflict"
        );
        assert_eq!(
            create_bucket(&conn, "  ", "/tmp/b").unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            create_bucket(&conn, "空的", "  ").unwrap_err().code,
            "invalid_input"
        );

        assert_eq!(
            update_bucket(&conn, 999, Some("x".into()), None)
                .unwrap_err()
                .code,
            "not_found"
        );
        assert_eq!(
            delete_bucket(&conn, 999).unwrap_err().code,
            "not_found"
        );
    }

    #[test]
    fn update_bucket_rejects_duplicate_name() {
        let conn = setup();
        let one = create_bucket(&conn, "一", "/tmp/a").unwrap();
        create_bucket(&conn, "二", "/tmp/b").unwrap();

        assert_eq!(
            update_bucket(&conn, one.id, Some("二".into()), None)
                .unwrap_err()
                .code,
            "conflict"
        );
        // 名称不变时不应误判冲突。
        assert!(update_bucket(&conn, one.id, Some("一".into()), None).is_ok());
    }
}
