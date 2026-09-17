pub mod agent;
/// Agent 循环；目录名 `loop` 为 Rust 关键字，故模块名取 `agent_loop`。
#[path = "loop/mod.rs"]
pub mod agent_loop;
pub mod anki;
pub mod asset;
pub mod event;
pub mod llm;
pub mod permission;
pub mod scheduler;
pub mod session;
pub mod statistics;
pub mod tool;

use std::sync::Mutex;

use rusqlite::Connection;

use crate::interface::error::ApiError;

/// 短作用域数据库访问：加锁 → 读取 → 释放，保证锁不跨 `.await` 存活。
pub(crate) fn with_conn<T>(
    db: &Mutex<Connection>,
    f: impl FnOnce(&Connection) -> Result<T, ApiError>,
) -> Result<T, ApiError> {
    let conn = db
        .lock()
        .map_err(|_| ApiError::internal("数据库连接不可用"))?;
    f(&conn)
}
