//! 界面外观配置命令。

use crate::interface::error::ApiError;
use crate::AppState;

/// 读取 `config.toml` 中保存的皮肤模式。
#[tauri::command]
pub fn ui_get_theme(state: tauri::State<'_, AppState>) -> String {
    state.config.ui().theme
}

/// 保存皮肤模式到 `config.toml`，使其在下次启动时恢复。
#[tauri::command]
pub fn ui_set_theme(theme: String, state: tauri::State<'_, AppState>) -> Result<(), ApiError> {
    let mut ui = state.config.ui();
    ui.theme = theme;
    state.config.set_ui(ui)
}
