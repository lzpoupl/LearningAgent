//! Agent 与工具命令，仅做参数透传。

use crate::interface::agent::{
    AgentConfigInput, AgentInfo, AgentToolPermission, AgentToolPermissionInput, ToolGroup, ToolInfo,
    ToolPermission,
};
use crate::interface::error::ApiError;
use crate::AppState;

#[tauri::command]
pub fn agent_list(state: tauri::State<'_, AppState>) -> Result<Vec<AgentInfo>, ApiError> {
    state.agent.list_agents()
}

#[tauri::command]
pub fn agent_get(state: tauri::State<'_, AppState>, agent_id: i64) -> Result<AgentInfo, ApiError> {
    state.agent.get_agent(agent_id)
}

#[tauri::command]
pub fn agent_create(
    state: tauri::State<'_, AppState>,
    input: AgentConfigInput,
) -> Result<AgentInfo, ApiError> {
    state.agent.create_agent(input)
}

#[tauri::command]
pub fn agent_update(
    state: tauri::State<'_, AppState>,
    agent_id: i64,
    input: AgentConfigInput,
) -> Result<AgentInfo, ApiError> {
    state.agent.update_agent(agent_id, input)
}

#[tauri::command]
pub fn agent_delete(state: tauri::State<'_, AppState>, agent_id: i64) -> Result<(), ApiError> {
    state.agent.delete_agent(agent_id)
}

#[tauri::command]
pub fn tool_list(state: tauri::State<'_, AppState>) -> Result<Vec<ToolInfo>, ApiError> {
    state.agent.list_tools()
}

#[tauri::command]
pub fn get_all_tool_groups(state: tauri::State<'_, AppState>) -> Result<Vec<ToolGroup>, ApiError> {
    state.agent.list_tool_groups()
}

#[tauri::command]
pub fn agent_get_tool_permissions(
    state: tauri::State<'_, AppState>,
    agent_id: i64,
) -> Result<Vec<AgentToolPermission>, ApiError> {
    state.agent.get_tool_permissions(agent_id)
}

#[tauri::command]
pub fn agent_set_tool_permission(
    state: tauri::State<'_, AppState>,
    agent_id: i64,
    tool_id: String,
    permission: ToolPermission,
) -> Result<AgentToolPermission, ApiError> {
    state.agent.set_tool_permission(agent_id, &tool_id, permission)
}

#[tauri::command]
pub fn agent_set_tool_permissions(
    state: tauri::State<'_, AppState>,
    agent_id: i64,
    items: Vec<AgentToolPermissionInput>,
) -> Result<Vec<AgentToolPermission>, ApiError> {
    state.agent.set_tool_permissions(agent_id, items)
}

#[tauri::command]
pub fn agent_resolve_tool_permission(
    state: tauri::State<'_, AppState>,
    agent_id: i64,
    tool_id: String,
) -> Result<ToolPermission, ApiError> {
    state.agent.resolve_tool_permission(agent_id, &tool_id)
}
