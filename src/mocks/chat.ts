import type {
  AgentEvent,
  AgentType,
  MessageInfo,
  SessionDetail,
  SessionInfo,
  ToolCall,
} from '../types/chat'

interface StoredSession {
  session: SessionInfo
  messages: MessageInfo[]
}

function now(): string {
  return new Date().toISOString()
}

function cloneMessage(message: MessageInfo): MessageInfo {
  return {
    ...message,
    toolCalls: message.toolCalls.map(call => ({ ...call, arguments: { ...call.arguments } })),
  }
}

function cloneSession(session: SessionInfo): SessionInfo {
  return { ...session }
}

function autoTitle(content: string): string {
  return content.length > 20 ? `${content.slice(0, 20)}...` : content
}

/** 把事件按序回放到前端 Channel；Tauri 的 mockIPC 提供了 runCallback。 */
function replay(onEvent: unknown, events: AgentEvent[]): void {
  const raw = String(onEvent ?? '')
  const prefix = '__CHANNEL__:'
  if (!raw.startsWith(prefix)) {
    return
  }
  const id = Number(raw.slice(prefix.length))
  const internals = (
    window as unknown as {
      __TAURI_INTERNALS__?: {
        runCallback?: (id: number, data: { message: unknown; index: number }) => void
      }
    }
  ).__TAURI_INTERNALS__
  if (!internals?.runCallback) {
    return
  }

  // 延后到微任务之后，保证命令的返回值先被前端拿到。
  setTimeout(() => {
    events.forEach((message, index) => internals.runCallback?.(id, { message, index }))
  }, 0)
}

/** 内存版 Agent 会话后端：会话生命周期与一轮回复都只存在于 mock 层。 */
export class ChatMock {
  private sessions = new Map<number, StoredSession>()
  private nextSessionId = 1
  private nextMessageId = 1
  private turnSeq = 0

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'session_list':
        return this.listSessions(payload.agentId)
      case 'session_get':
        return this.getSession(Number(payload.sessionId ?? 0))
      case 'session_rename':
        return this.renameSession(Number(payload.sessionId ?? 0), String(payload.title ?? ''))
      case 'session_delete':
        return this.deleteSession(Number(payload.sessionId ?? 0))
      case 'agent_start_session':
        return this.startSession(payload)
      case 'agent_send_message':
        return this.sendMessage(payload)
      case 'agent_cancel_turn':
      case 'agent_approve_tool_call':
      case 'agent_answer_question':
      case 'agent_skip_question':
        return undefined
      default:
        return undefined
    }
  }

  private nextTurnId(): string {
    this.turnSeq += 1
    return `t-${Date.now()}-${this.turnSeq}`
  }

  private createMessage(
    sessionId: number,
    turnId: string,
    role: MessageInfo['role'],
    content: string,
    extra: Partial<MessageInfo> = {},
  ): MessageInfo {
    return {
      id: this.nextMessageId++,
      sessionId,
      turnId,
      role,
      content,
      toolCalls: [] as ToolCall[],
      toolCallId: null,
      toolName: null,
      status: 'complete',
      promptTokens: null,
      completionTokens: null,
      createdAt: now(),
      ...extra,
    }
  }

  private createSession(agentId: AgentType, content: string): StoredSession {
    const createdAt = now()
    const session: SessionInfo = {
      id: this.nextSessionId++,
      agentId,
      title: autoTitle(content),
      lastProvider: 'edgee',
      lastModel: 'mock-model',
      messageCount: 0,
      lastMessageAt: null,
      createdAt,
      updatedAt: createdAt,
    }
    const stored: StoredSession = { session, messages: [] }
    this.sessions.set(session.id, stored)
    return stored
  }

  private touch(stored: StoredSession): void {
    stored.session.updatedAt = now()
  }

  private replyTo(agentId: AgentType, question: string): string {
    return [
      `学习助手 ${agentId} 已收到你的问题：${question}`,
      '',
      '示例公式 $\\int_0^1 x^2\\,dx = \\frac{1}{3}$，可在历史会话中回看整轮过程。',
    ].join('\n')
  }

  private listSessions(agentId: unknown): SessionInfo[] {
    const filter = typeof agentId === 'number' ? agentId : null
    return [...this.sessions.values()]
      .map(stored => stored.session)
      .filter(session => filter === null || session.agentId === filter)
      .sort((left, right) => right.updatedAt.localeCompare(left.updatedAt))
      .map(cloneSession)
  }

  private getSession(sessionId: number): SessionDetail {
    const stored = this.requireSession(sessionId)
    return {
      session: cloneSession(stored.session),
      messages: stored.messages.map(cloneMessage),
    }
  }

  private requireSession(sessionId: number): StoredSession {
    const stored = this.sessions.get(sessionId)
    if (!stored) {
      throw new Error(`会话不存在: ${sessionId}`)
    }
    return stored
  }

  private startSession(payload: Record<string, unknown>): { sessionId: number; turnId: string } {
    const input = (payload.input ?? {}) as { agentId?: number; content?: string }
    const agentId = Number(input.agentId ?? 0)
    const content = String(input.content ?? '').trim()
    if (!agentId || !content) {
      throw new Error('会话参数不完整')
    }

    const stored = this.createSession(agentId, content)
    const turnId = this.nextTurnId()

    const system = this.createMessage(stored.session.id, turnId, 'system', '你是学习助手。')
    const user = this.createMessage(stored.session.id, turnId, 'user', content)
    const assistant = this.createMessage(
      stored.session.id,
      turnId,
      'assistant',
      this.replyTo(agentId, content),
    )
    stored.messages.push(system, user, assistant)
    stored.session.messageCount = 2
    stored.session.lastMessageAt = assistant.createdAt
    this.touch(stored)

    replay(payload.onEvent, [
      { type: 'turn-started', sessionId: stored.session.id, turnId },
      { type: 'message-completed', sessionId: stored.session.id, turnId, message: cloneMessage(user) },
      {
        type: 'message-completed',
        sessionId: stored.session.id,
        turnId,
        message: cloneMessage(assistant),
      },
      { type: 'turn-ended', sessionId: stored.session.id, turnId, status: 'completed' },
    ])

    return { sessionId: stored.session.id, turnId }
  }

  private sendMessage(payload: Record<string, unknown>): { sessionId: number; turnId: string } {
    const input = (payload.input ?? {}) as { sessionId?: number; content?: string }
    const sessionId = Number(input.sessionId ?? 0)
    const content = String(input.content ?? '').trim()
    if (!content) {
      throw new Error('消息不能为空')
    }

    const stored = this.requireSession(sessionId)
    const turnId = this.nextTurnId()
    const user = this.createMessage(sessionId, turnId, 'user', content)
    const assistant = this.createMessage(
      sessionId,
      turnId,
      'assistant',
      this.replyTo(stored.session.agentId, content),
    )
    stored.messages.push(user, assistant)
    stored.session.messageCount += 2
    stored.session.lastMessageAt = assistant.createdAt
    this.touch(stored)

    replay(payload.onEvent, [
      { type: 'turn-started', sessionId, turnId },
      { type: 'message-completed', sessionId, turnId, message: cloneMessage(user) },
      { type: 'message-completed', sessionId, turnId, message: cloneMessage(assistant) },
      { type: 'turn-ended', sessionId, turnId, status: 'completed' },
    ])

    return { sessionId, turnId }
  }

  private renameSession(sessionId: number, title: string): SessionInfo {
    const stored = this.requireSession(sessionId)
    const normalized = title.trim()
    if (!normalized) {
      throw new Error('会话名称不能为空')
    }
    stored.session.title = normalized
    this.touch(stored)
    return cloneSession(stored.session)
  }

  private deleteSession(sessionId: number): void {
    if (!this.sessions.delete(sessionId)) {
      throw new Error(`会话不存在: ${sessionId}`)
    }
  }
}
