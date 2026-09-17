use std::path::Path;

use rusqlite::Connection;

include!(concat!(env!("OUT_DIR"), "\\migrations.rs"));

/// 打开数据库连接；文件不存在时由 SQLite 新建并执行迁移。
///
/// 路径的父目录不存在时先创建，避免落盘时因目录缺失而失败。
pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("无法创建数据库目录 {}: {e}", parent.display())),
                )
            })?;
        }
    }

    let mut conn = Connection::open(path)?;
    prepare(&mut conn)?;
    Ok(conn)
}

/// 打开仅存在于当前进程内的数据库，并执行最新迁移。
///
/// 前端测试通过 Vite 开发模式运行时，后端也以 debug 构建启动，因此使用
/// 该连接可以避免测试数据写入用户的正式数据库。
pub fn open_in_memory() -> rusqlite::Result<Connection> {
    let mut conn = Connection::open_in_memory()?;
    prepare(&mut conn)?;
    Ok(conn)
}

fn prepare(conn: &mut Connection) -> rusqlite::Result<()> {
    conn.pragma_update(None, "foreign_keys", true)?;
    migrate(conn)?;
    Ok(())
}

/// 按 `PRAGMA user_version` 记录的编号增量执行迁移。
///
/// 只执行编号大于当前版本的文件，且版本号推进与 SQL 执行在同一事务内提交，
/// 迁移失败时整体回滚，下次启动会重试同一个文件。
pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {
    migrate_with(conn, MIGRATIONS)
}

/// 迁移执行核心；迁移清单可注入，便于测试失败回滚等边界行为。
fn migrate_with(conn: &mut Connection, migrations: &[(u32, &str)]) -> rusqlite::Result<()> {
    let current: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for (number, sql) in migrations.iter() {
        if *number <= current {
            continue;
        }
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", *number as i64)?;
        tx.commit()?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user_version(conn: &Connection) -> u32 {
        conn.query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap()
    }

    fn latest_version() -> u32 {
        MIGRATIONS.iter().map(|(number, _)| *number).max().unwrap_or(0)
    }

    #[test]
    fn open_creates_database_file_and_parent_directory() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("LearningAgent.db");

        let conn = open(&path).unwrap();

        assert!(path.exists());
        assert_eq!(user_version(&conn), latest_version());
    }

    #[test]
    fn fresh_database_applies_all_migrations() {
        let conn = open_in_memory().unwrap();
        assert_eq!(user_version(&conn), latest_version());

        // 000001 与 000002 建立的表都存在，且内置数据已播种。
        let decks: i64 = conn
            .query_row("SELECT COUNT(*) FROM deck", [], |row| row.get(0))
            .unwrap();
        assert_eq!(decks, 0);
        let agents: i64 = conn
            .query_row("SELECT COUNT(*) FROM agent", [], |row| row.get(0))
            .unwrap();
        assert_eq!(agents, 2);
        let tools: i64 = conn
            .query_row("SELECT COUNT(*) FROM tool", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tools, 14);
    }

    #[test]
    fn migrate_is_idempotent_for_applied_versions() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "foreign_keys", true).unwrap();
        migrate(&mut conn).unwrap();
        // 二次执行不会重放已应用的迁移，否则 UNIQUE / 主键会冲突。
        migrate(&mut conn).unwrap();
        assert_eq!(user_version(&conn), latest_version());

        let agents: i64 = conn
            .query_row("SELECT COUNT(*) FROM agent", [], |row| row.get(0))
            .unwrap();
        assert_eq!(agents, 2);
    }

    #[test]
    fn failed_migration_rolls_back_and_keeps_version() {
        let mut conn = Connection::open_in_memory().unwrap();
        let migrations: &[(u32, &str)] = &[
            (1, "CREATE TABLE demo (id INTEGER PRIMARY KEY);"),
            (
                2,
                "INSERT INTO demo VALUES (1); INSERT INTO missing_table VALUES (1);",
            ),
        ];

        assert!(migrate_with(&mut conn, migrations).is_err());

        // 版本号停在已成功提交的 1，失败文件未推进；文件内的插入也一并回滚。
        assert_eq!(user_version(&conn), 1);
        let demo: i64 = conn
            .query_row("SELECT COUNT(*) FROM demo", [], |row| row.get(0))
            .unwrap();
        assert_eq!(demo, 0);
    }
}
