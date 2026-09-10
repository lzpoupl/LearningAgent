import { invoke } from '@tauri-apps/api/core'

import type { AgentType, ChatMessage, ChatSession } from '../types/chat'

export interface ChatRequest {
  agent: AgentType
  message: string
  sessionId: string
}

export interface CreateSessionRequest {
  agent: AgentType
  message: string
}

export interface ChatResponse {
  message: ChatMessage
}

/** 与 Agent 会话后端保持一一对应的请求封装。 */
export function listSessions(): Promise<ChatSession[]> {
  return invoke<ChatSession[]>('chat_list_sessions')
}

export function getSession(sessionId: string): Promise<ChatSession> {
  return invoke<ChatSession>('chat_get_session', { sessionId })
}

export function createSession(request: CreateSessionRequest): Promise<ChatSession> {
  return invoke<ChatSession>('chat_create_session', { ...request })
}

export function sendMessage(request: ChatRequest): Promise<ChatResponse> {
  return invoke<ChatResponse>('chat_send_message', { ...request })
}

export function renameSession(sessionId: string, title: string): Promise<ChatSession> {
  return invoke<ChatSession>('chat_rename_session', { sessionId, title })
}

export function deleteSession(sessionId: string): Promise<void> {
  return invoke<void>('chat_delete_session', { sessionId })
}
