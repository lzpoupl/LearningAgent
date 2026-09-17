//! 新模块共享的接口错误，结构与既有 `AnkiError` 一致，便于后续平滑迁移。

use serde::{Deserialize, Serialize};

use crate::interface::anki::AnkiError;

/// 统一的接口错误：`code` 为稳定错误码，`message` 面向用户。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("not_found", message)
    }

    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::new("invalid_input", message)
    }

    pub fn invalid_path(message: impl Into<String>) -> Self {
        Self::new("invalid_path", message)
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new("conflict", message)
    }

    pub fn builtin_protected(message: impl Into<String>) -> Self {
        Self::new("builtin_protected", message)
    }

    pub fn tool_denied(message: impl Into<String>) -> Self {
        Self::new("tool_denied", message)
    }

    pub fn tool_unavailable(message: impl Into<String>) -> Self {
        Self::new("tool_unavailable", message)
    }

    pub fn llm_unconfigured(message: impl Into<String>) -> Self {
        Self::new("llm_unconfigured", message)
    }

    pub fn llm_error(message: impl Into<String>) -> Self {
        Self::new("llm_error", message)
    }

    pub fn turn_conflict(message: impl Into<String>) -> Self {
        Self::new("turn_conflict", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("internal", message)
    }
}

impl From<rusqlite::Error> for ApiError {
    fn from(e: rusqlite::Error) -> Self {
        Self::new("db", e.to_string())
    }
}

impl From<AnkiError> for ApiError {
    fn from(e: AnkiError) -> Self {
        Self {
            code: e.code,
            message: e.message,
        }
    }
}
