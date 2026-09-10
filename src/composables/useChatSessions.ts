import { computed, ref } from 'vue'

import { sendMessage as requestMessage } from '../services/api'
import type {
  AgentType,
  ChatMessage,
  ChatSession,
} from '../types/chat'

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

export function useChatSessions() {
  const sessions = ref<ChatSession[]>([])
  const currentSessionId = ref('')
  const currentAgent = ref<AgentType>('math')
  const loading = ref(false)

  const currentSession = computed(() =>
    sessions.value.find(session => session.id === currentSessionId.value) ?? null,
  )

  const currentMessages = computed(() => currentSession.value?.messages ?? [])

  async function requestAI(session: ChatSession, question: string) {
    loading.value = true

    try {
      const response = await requestMessage({
        agent: session.agent,
        message: question,
        sessionId: session.id,
      })

      session.messages.push(response.message)
    } catch (error) {
      console.error(error)
      session.messages.push(
        createTextMessage('assistant', '抱歉，请求失败，请稍后重试。'),
      )
    } finally {
      loading.value = false
    }
  }

  async function startSession(agent: AgentType, question: string) {
    const session: ChatSession = {
      id: createId('session'),
      title: question.length > 20 ? `${question.substring(0, 20)}...` : question,
      agent,
      createdAt: new Date().toISOString(),
      messages: [createTextMessage('user', question)],
    }

    sessions.value.unshift(session)
    currentSessionId.value = session.id
    currentAgent.value = agent

    await requestAI(session, question)
  }

  async function sendMessage(content: string) {
    const question = content.trim()
    const session = currentSession.value

    if (!question || !session || loading.value) {
      return
    }

    session.messages.push(createTextMessage('user', question))
    await requestAI(session, question)
  }

  function selectSession(sessionId: string) {
    const session = sessions.value.find(item => item.id === sessionId)

    if (!session) {
      return
    }

    currentSessionId.value = session.id
    currentAgent.value = session.agent
  }

  function renameSession(sessionId: string, title: string) {
    const session = sessions.value.find(item => item.id === sessionId)

    if (session && title.trim()) {
      session.title = title.trim()
    }
  }

  function deleteSession(sessionId: string) {
    const index = sessions.value.findIndex(item => item.id === sessionId)

    if (index === -1) {
      return
    }

    sessions.value.splice(index, 1)

    if (currentSessionId.value !== sessionId) {
      return
    }

    const nextSession = sessions.value[0]
    currentSessionId.value = nextSession?.id ?? ''
    currentAgent.value = nextSession?.agent ?? 'math'
  }

  return {
    sessions,
    currentSessionId,
    currentSession,
    currentMessages,
    currentAgent,
    loading,
    startSession,
    sendMessage,
    selectSession,
    renameSession,
    deleteSession,
  }
}
