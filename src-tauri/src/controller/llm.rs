//! LLM provider 配置与探测命令。

use crate::interface::error::ApiError;
use crate::interface::llm::{LlmConfigView, LlmProviderInput, ProbeResult};
use crate::AppState;

#[tauri::command]
pub fn llm_get_config(state: tauri::State<'_, AppState>) -> Result<LlmConfigView, ApiError> {
    Ok(state.llm.config_view())
}

#[tauri::command]
pub fn llm_upsert_provider(
    state: tauri::State<'_, AppState>,
    input: LlmProviderInput,
) -> Result<LlmConfigView, ApiError> {
    state.llm.upsert_provider(input)
}

#[tauri::command]
pub fn llm_set_default_provider(
    state: tauri::State<'_, AppState>,
    name: String,
) -> Result<LlmConfigView, ApiError> {
    state.llm.set_default_provider(&name)
}

#[tauri::command]
pub async fn llm_test_provider(
    state: tauri::State<'_, AppState>,
    name: Option<String>,
) -> Result<ProbeResult, ApiError> {
    let llm = state.llm.clone();
    llm.probe(name.as_deref()).await
}
