pub mod agent;
pub mod anki;
pub mod bucket;
pub mod db;
pub mod deck;
pub mod fs;
pub mod statistics;

use rusqlite::Connection;

use crate::interface::anki::AnkiError;
use crate::interface::error::ApiError;

/// 把 rusqlite 错误统一映射为接口错误。
pub(crate) fn map_rusqlite(e: rusqlite::Error) -> AnkiError {
    AnkiError {
        code: "db".into(),
        message: e.to_string(),
    }
}

/// 需要原子性时开启事务；调用方已处于事务中则并入外层事务。
pub(crate) fn with_tx<T>(
    conn: &Connection,
    f: impl FnOnce(&Connection) -> Result<T, ApiError>,
) -> Result<T, ApiError> {
    if conn.is_autocommit() {
        let tx = conn.unchecked_transaction()?;
        let out = f(&tx)?;
        tx.commit()?;
        Ok(out)
    } else {
        f(conn)
    }
}
