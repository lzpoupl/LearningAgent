pub mod anki;
pub mod db;
pub mod deck;

use crate::interface::anki::AnkiError;

/// 把 rusqlite 错误统一映射为接口错误。
pub(crate) fn map_rusqlite(e: rusqlite::Error) -> AnkiError {
    AnkiError {
        code: "db".into(),
        message: e.to_string(),
    }
}
