//! 会话与轮次命令；仅做参数透传，把 `Channel<AgentEvent>` 包装为 `ChannelEmitter`。

use std::sync::Arc;

use tauri::ipc::Channel;

use crate::interface::error::ApiError;
use crate::interface::event::AgentEvent;
use crate::interface::session::{
    ApprovalDecision, SendMessageInput, SessionDetail, SessionInfo, StartSessionInput, TurnHandle,
};
use crate::service::event::{ChannelEmitter, EventEmitter};
use crate::AppState;

#[tauri::command]
pub fn session_list(
    state: tauri::State<'_, AppState>,
    agent_id: Option<i64>,
) -> Result<Vec<SessionInfo>, ApiError> {
    state.session.list_sessions(agent_id)
}

#[tauri::command]
pub fn session_get(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<SessionDetail, ApiError> {
    state.session.get_session(session_id)
}

#[tauri::command]
pub fn session_rename(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    title: String,
) -> Result<SessionInfo, ApiError> {
    state.session.rename_session(session_id, &title)
}

#[tauri::command]
pub fn session_delete(state: tauri::State<'_, AppState>, session_id: i64) -> Result<(), ApiError> {
    state.session.delete_session(session_id)
}

#[tauri::command]
pub fn agent_start_session(
    state: tauri::State<'_, AppState>,
    input: StartSessionInput,
    on_event: Channel<AgentEvent>,
) -> Result<TurnHandle, ApiError> {
    let emitter: Arc<dyn EventEmitter> = Arc::new(ChannelEmitter::new(on_event));
    state.session.start_session(input, emitter)
}

#[tauri::command]
pub fn agent_send_message(
    state: tauri::State<'_, AppState>,
    input: SendMessageInput,
    on_event: Channel<AgentEvent>,
) -> Result<TurnHandle, ApiError> {
    let emitter: Arc<dyn EventEmitter> = Arc::new(ChannelEmitter::new(on_event));
    state.session.send_message(input, emitter)
}

#[tauri::command]
pub fn agent_cancel_turn(
    state: tauri::State<'_, AppState>,
    session_id: i64,
) -> Result<(), ApiError> {
    state.session.cancel_turn(session_id)
}

#[tauri::command]
pub fn agent_approve_tool_call(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    turn_id: String,
    call_id: String,
    decision: ApprovalDecision,
) -> Result<(), ApiError> {
    state
        .session
        .approve_tool_call(session_id, &turn_id, &call_id, decision)
}

#[tauri::command]
pub fn agent_answer_question(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    turn_id: String,
    call_id: String,
    answer: String,
) -> Result<(), ApiError> {
    state
        .session
        .answer_question(session_id, &turn_id, &call_id, &answer)
}

#[tauri::command]
pub fn agent_skip_question(
    state: tauri::State<'_, AppState>,
    session_id: i64,
    turn_id: String,
    call_id: String,
) -> Result<(), ApiError> {
    state.session.skip_question(session_id, &turn_id, &call_id)
}
