use std::path::Path;

use rusqlite::Connection;

include!(concat!(env!("OUT_DIR"), "\\migrations.rs"));

pub fn open<P: AsRef<Path>>(path: P) -> rusqlite::Result<Connection> {
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
    let current: u32 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;

    for (number, sql) in MIGRATIONS.iter() {
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
