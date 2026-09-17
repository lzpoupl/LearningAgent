import { invoke, type Channel } from '@tauri-apps/api/core'

import type {
  AgentEvent,
  ApprovalDecision,
  SendMessageInput,
  SessionDetail,
  SessionInfo,
  StartSessionInput,
  TurnHandle,
} from '../types/chat'

/** 会话列表；缺省返回全部 Agent 的会话，按更新时间倒序。 */
export function listSessions(agentId?: number): Promise<SessionInfo[]> {
  return invoke<SessionInfo[]>('session_list', { agentId: agentId ?? null })
}

/** 会话 + 全部消息。 */
export function getSession(sessionId: number): Promise<SessionDetail> {
  return invoke<SessionDetail>('session_get', { sessionId })
}

export function renameSession(sessionId: number, title: string): Promise<SessionInfo> {
  return invoke<SessionInfo>('session_rename', { sessionId, title })
}

export function deleteSession(sessionId: number): Promise<void> {
  return invoke<void>('session_delete', { sessionId })
}

/** 新建会话并开始第一轮；进度经 `onEvent` 推送。 */
export function startSession(
  input: StartSessionInput,
  onEvent: Channel<AgentEvent>,
): Promise<TurnHandle> {
  return invoke<TurnHandle>('agent_start_session', { input, onEvent })
}

/** 在既有会话上开始新一轮；进度经 `onEvent` 推送。 */
export function sendMessage(
  input: SendMessageInput,
  onEvent: Channel<AgentEvent>,
): Promise<TurnHandle> {
  return invoke<TurnHandle>('agent_send_message', { input, onEvent })
}

export function cancelTurn(sessionId: number): Promise<void> {
  return invoke<void>('agent_cancel_turn', { sessionId })
}

export function approveToolCall(
  sessionId: number,
  turnId: string,
  callId: string,
  decision: ApprovalDecision,
): Promise<void> {
  return invoke<void>('agent_approve_tool_call', { sessionId, turnId, callId, decision })
}
