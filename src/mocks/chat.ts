import type { AgentType, ChatMessage, ChatSession } from '../types/chat'

function createId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

function createTextMessage(role: ChatMessage['role'], content: string): ChatMessage {
  return {
    id: createId(role),
    role,
    time: new Date().toISOString(),
    content: [{ type: 'text', content }],
  }
}

function cloneMessage(message: ChatMessage): ChatMessage {
  return {
    ...message,
    content: message.content.map(block => ({
      ...block,
      data: block.data ? { ...block.data } : undefined,
    })),
  }
}

function cloneSession(session: ChatSession): ChatSession {
  return {
    ...session,
    messages: session.messages.map(cloneMessage),
  }
}

function createAssistantMessage(agent: AgentType, question: string): ChatMessage {
  return {
    id: createId('assistant'),
    role: 'assistant',
    time: new Date().toISOString(),
    content: [
      {
        type: 'text',
        content: `学习助手 ${agent} 已收到你的问题：${question}`,
      },
      {
        type: 'latex',
        content: '\\int_0^1 x^2 dx = \\frac{1}{3}',
      },
      {
        type: 'result',
        title: '计算结果',
        data: {
          answer: '1/3',
          status: 'success',
        },
      },
    ],
  }
}

/** 内存版 Agent 会话后端：会话生命周期和示例回复都只存在于 mock 层。 */
export class ChatMock {
  private sessions = new Map<string, ChatSession>()

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'chat_list_sessions':
        return [...this.sessions.values()].map(cloneSession)
      case 'chat_get_session':
        return cloneSession(this.requireSession(String(payload.sessionId ?? '')))
      case 'chat_create_session':
        return this.createSession(
          Number(payload.agent ?? 0),
          String(payload.message ?? ''),
        )
      case 'chat_send_message':
        return this.sendMessage(
          String(payload.sessionId ?? ''),
          Number(payload.agent ?? 0),
          String(payload.message ?? ''),
        )
      case 'chat_rename_session':
        return this.renameSession(
          String(payload.sessionId ?? ''),
          String(payload.title ?? ''),
        )
      case 'chat_delete_session':
        return this.deleteSession(String(payload.sessionId ?? ''))
      default:
        return undefined
    }
  }

  private requireSession(sessionId: string): ChatSession {
    const session = this.sessions.get(sessionId)
    if (!session) {
      throw new Error(`会话不存在: ${sessionId}`)
    }
    return session
  }

  private createSession(agent: AgentType, question: string): ChatSession {
    if (!agent || !question.trim()) {
      throw new Error('会话参数不完整')
    }

    const id = createId('session')
    const normalizedQuestion = question.trim()
    const session: ChatSession = {
      id,
      title: normalizedQuestion.length > 20
        ? `${normalizedQuestion.substring(0, 20)}...`
        : normalizedQuestion,
      agent,
      createdAt: new Date().toISOString(),
      messages: [
        createTextMessage('user', normalizedQuestion),
        createAssistantMessage(agent, normalizedQuestion),
      ],
    }

    this.sessions.set(id, session)
    return cloneSession(session)
  }

  private sendMessage(sessionId: string, agent: AgentType, question: string): { message: ChatMessage } {
    const session = this.requireSession(sessionId)
    const normalizedQuestion = question.trim()
    if (!normalizedQuestion) {
      throw new Error('消息不能为空')
    }

    session.messages.push(createTextMessage('user', normalizedQuestion))
    const message = createAssistantMessage(agent || session.agent, normalizedQuestion)
    session.messages.push(message)
    return { message: cloneMessage(message) }
  }

  private renameSession(sessionId: string, title: string): ChatSession {
    const session = this.requireSession(sessionId)
    const normalizedTitle = title.trim()
    if (!normalizedTitle) {
      throw new Error('会话名称不能为空')
    }
    session.title = normalizedTitle
    return cloneSession(session)
  }

  private deleteSession(sessionId: string): void {
    if (!this.sessions.delete(sessionId)) {
      throw new Error(`会话不存在: ${sessionId}`)
    }
  }
}
