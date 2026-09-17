//! 界面外观配置。
//!
//! 字段名与 `config.toml` 中的键保持一致，采用 snake_case。

use serde::{Deserialize, Serialize};

use crate::interface::error::ApiError;

/// 支持的皮肤模式，与前端 `UserTheme` 保持一致。
pub const THEME_MODES: [&str; 3] = ["light", "dark", "system"];

/// 皮肤模式默认值：跟随操作系统深浅色。
pub const DEFAULT_THEME_MODE: &str = "system";

/// 界面外观配置。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct UiConfig {
    /// 皮肤模式：`light` / `dark` / `system`。
    pub theme: String,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: DEFAULT_THEME_MODE.to_string(),
        }
    }
}

impl UiConfig {
    /// 校验皮肤模式取值，非法值返回 `invalid_input`，避免写坏配置文件。
    pub fn validate_theme(theme: &str) -> Result<(), ApiError> {
        if THEME_MODES.contains(&theme) {
            return Ok(());
        }

        Err(ApiError::invalid_input(format!(
            "不支持的皮肤模式 {theme}，可选值为 {}。",
            THEME_MODES.join(" / ")
        )))
    }
}
