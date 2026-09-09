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

pub fn migrate(conn: &mut Connection) -> rusqlite::Result<()> {

    for sql in MIGRATIONS.iter() {
        conn.execute_batch(sql)?;
    }

    Ok(())
}
