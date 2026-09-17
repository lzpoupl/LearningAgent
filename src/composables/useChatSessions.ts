import { computed, onMounted, ref } from 'vue'
import { Channel } from '@tauri-apps/api/core'

import {
  approveToolCall as requestApproveToolCall,
  cancelTurn as requestCancelTurn,
  deleteSession as requestDeleteSession,
  getSession as requestGetSession,
  listSessions,
  renameSession as requestRenameSession,
  sendMessage as requestSendMessage,
  startSession as requestStartSession,
} from '../services/session'
import type {
  AgentEvent,
  AgentType,
  ApprovalDecision,
  MessageInfo,
  PendingApproval,
  SessionInfo,
  ToolCallResult,
} from '../types/chat'

/**
 * Agent 会话与轮次编排：会话列表来自 `session_*`，一轮对话在途的内容由
 * 命令传入的 `Channel<AgentEvent>` 驱动，轮次结束后以数据库内容为准刷新。
 */
export function useChatSessions() {
  const sessions = ref<SessionInfo[]>([])
  const currentSessionId = ref(0)
  const currentAgent = ref<AgentType>(0)
  const currentMessages = ref<MessageInfo[]>([])
  const loading = ref(false)
  const streamingMessageId = ref<number | null>(null)
  const pendingApproval = ref<PendingApproval | null>(null)
  const toolResults = ref<Record<string, ToolCallResult>>({})
  const turnError = ref('')

  const sessionChannels = new Map<number, Channel<AgentEvent>>()
  let pendingChannel: Channel<AgentEvent> | null = null
  let currentTurnId = ''
  let tempId = -1

  const currentSession = computed(
    () => sessions.value.find(session => session.id === currentSessionId.value) ?? null,
  )

  function refreshSessions() {
    return listSessions()
      .then(loaded => {
        sessions.value = loaded
      })
      .catch(error => console.error(error))
  }

  async function refreshMessages(sessionId: number) {
    try {
      const detail = await requestGetSession(sessionId)
      if (currentSessionId.value !== sessionId) {
        return
      }
      // system 快照不进入前端消息列表。
      currentMessages.value = detail.messages.filter(message => message.role !== 'system')
      const index = sessions.value.findIndex(session => session.id === detail.session.id)
      if (index !== -1) {
        sessions.value[index] = detail.session
      }
    } catch (error) {
      console.error(error)
    }
  }

  function upsertMessage(message: MessageInfo) {
    const list = currentMessages.value
    const index = list.findIndex(item => item.id === message.id)
    if (index !== -1) {
      list[index] = message
      return
    }
    if (message.role === 'user') {
      const optimistic = list.findIndex(
        item => item.id < 0 && item.role === 'user' && item.content === message.content,
      )
      if (optimistic !== -1) {
        list[optimistic] = message
        return
      }
    }
    list.push(message)
  }

  function handleEvent(event: AgentEvent) {
    if (!currentSessionId.value) {
      currentSessionId.value = event.sessionId
    }
    if (event.sessionId !== currentSessionId.value) {
      return
    }

    switch (event.type) {
      case 'turn-started':
        loading.value = true
        turnError.value = ''
        currentTurnId = event.turnId
        if (pendingChannel) {
          sessionChannels.set(event.sessionId, pendingChannel)
          pendingChannel = null
        }
        break
      case 'message-delta': {
        streamingMessageId.value = event.messageId
        const existing = currentMessages.value.find(item => item.id === event.messageId)
        if (existing) {
          existing.content += event.delta
          existing.status = 'streaming'
        } else {
          currentMessages.value.push({
            id: event.messageId,
            sessionId: event.sessionId,
            turnId: event.turnId,
            role: 'assistant',
            content: event.delta,
            toolCalls: [],
            toolCallId: null,
            toolName: null,
            status: 'streaming',
            promptTokens: null,
            completionTokens: null,
            createdAt: new Date().toISOString(),
          })
        }
        break
      }
      case 'message-completed':
        upsertMessage(event.message)
        break
      case 'tool-result':
        toolResults.value = {
          ...toolResults.value,
          [event.callId]: { ok: event.ok, result: event.result, error: event.error },
        }
        break
      case 'tool-approval-required':
        pendingApproval.value = {
          callId: event.callId,
          toolId: event.toolId,
          arguments: event.arguments,
          turnId: event.turnId,
        }
        break
      case 'turn-ended':
        loading.value = false
        streamingMessageId.value = null
        pendingApproval.value = null
        currentTurnId = ''
        if (event.status === 'failed') {
          turnError.value = event.error?.message ?? '本轮请求失败，请稍后重试。'
        }
        sessionChannels.delete(event.sessionId)
        void refreshMessages(event.sessionId).then(refreshSessions)
        break
    }
  }

  function createChannel(): Channel<AgentEvent> {
    const channel = new Channel<AgentEvent>()
    channel.onmessage = handleEvent
    pendingChannel = channel
    return channel
  }

  async function startSession(agent: AgentType, content: string) {
    const question = content.trim()
    if (!agent || !question) {
      return
    }

    currentMessages.value = []
    toolResults.value = {}
    turnError.value = ''
    currentSessionId.value = 0
    currentAgent.value = agent
    loading.value = true

    const channel = createChannel()
    try {
      const handle = await requestStartSession(
        { agentId: agent, content: question },
        channel,
      )
      if (!currentSessionId.value) {
        currentSessionId.value = handle.sessionId
      }
      currentTurnId = handle.turnId
      if (pendingChannel === channel) {
        sessionChannels.set(handle.sessionId, channel)
        pendingChannel = null
      }
      await refreshSessions()
    } catch (error) {
      channel.onmessage = () => {}
      pendingChannel = null
      loading.value = false
      turnError.value = error instanceof Error ? error.message : String(error)
      throw error
    }
  }

  async function sendMessage(content: string) {
    const question = content.trim()
    const session = currentSession.value
    if (!question || !session || loading.value) {
      return
    }

    loading.value = true
    turnError.value = ''
    currentMessages.value.push({
      id: tempId--,
      sessionId: session.id,
      turnId: null,
      role: 'user',
      content: question,
      toolCalls: [],
      toolCallId: null,
      toolName: null,
      status: 'complete',
      promptTokens: null,
      completionTokens: null,
      createdAt: new Date().toISOString(),
    })

    const channel = createChannel()
    try {
      const handle = await requestSendMessage({ sessionId: session.id, content: question }, channel)
      currentTurnId = handle.turnId
      if (pendingChannel === channel) {
        sessionChannels.set(handle.sessionId, channel)
        pendingChannel = null
      }
    } catch (error) {
      channel.onmessage = () => {}
      pendingChannel = null
      loading.value = false
      turnError.value = error instanceof Error ? error.message : String(error)
      throw error
    }
  }

  async function cancelTurn() {
    if (!currentSessionId.value || !currentTurnId) {
      return
    }
    try {
      await requestCancelTurn(currentSessionId.value)
    } catch (error) {
      console.error(error)
    }
  }

  async function approveToolCall(decision: ApprovalDecision) {
    const approval = pendingApproval.value
    if (!approval) {
      return
    }
    const sessionId = currentSessionId.value
    pendingApproval.value = null
    try {
      await requestApproveToolCall(sessionId, approval.turnId, approval.callId, decision)
    } catch (error) {
      console.error(error)
    }
  }

  async function selectSession(sessionId: number) {
    const session = sessions.value.find(item => item.id === sessionId)
    if (!session) {
      return
    }
    currentSessionId.value = sessionId
    currentAgent.value = session.agentId
    toolResults.value = {}
    turnError.value = ''
    await refreshMessages(sessionId)
  }

  async function renameSession(sessionId: number, title: string) {
    const normalizedTitle = title.trim()
    if (!normalizedTitle) {
      return
    }
    const updated = await requestRenameSession(sessionId, normalizedTitle)
    const index = sessions.value.findIndex(item => item.id === sessionId)
    if (index !== -1) {
      sessions.value[index] = updated
    }
  }

  async function deleteSession(sessionId: number) {
    const index = sessions.value.findIndex(item => item.id === sessionId)
    if (index === -1) {
      return
    }
    await requestDeleteSession(sessionId)
    sessions.value.splice(index, 1)

    if (currentSessionId.value !== sessionId) {
      return
    }

    const next = sessions.value[0]
    currentMessages.value = []
    currentAgent.value = next?.agentId ?? 0
    if (next) {
      await selectSession(next.id)
    } else {
      currentSessionId.value = 0
    }
  }

  function resetSession() {
    currentSessionId.value = 0
    currentAgent.value = 0
    currentMessages.value = []
    toolResults.value = {}
    pendingApproval.value = null
    turnError.value = ''
  }

  async function loadSessions() {
    try {
      const loaded = await listSessions()
      sessions.value = loaded
      const first = loaded[0]
      if (first) {
        await selectSession(first.id)
      }
    } catch (error) {
      console.error(error)
    }
  }

  onMounted(loadSessions)

  return {
    sessions,
    currentSessionId,
    currentSession,
    currentMessages,
    currentAgent,
    loading,
    streamingMessageId,
    pendingApproval,
    toolResults,
    turnError,
    startSession,
    sendMessage,
    selectSession,
    renameSession,
    deleteSession,
    cancelTurn,
    approveToolCall,
    resetSession,
  }
}
