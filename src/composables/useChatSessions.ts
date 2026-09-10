import { computed, onMounted, ref } from 'vue'

import {
  createSession as requestCreateSession,
  deleteSession as requestDeleteSession,
  getSession as requestGetSession,
  listSessions,
  renameSession as requestRenameSession,
  sendMessage as requestMessage,
} from '../services/api'
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

export function useChatSessions() {
  const sessions = ref<ChatSession[]>([])
  const currentSessionId = ref('')
  const currentAgent = ref<AgentType>('')
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
      session.messages.push(createTextMessage('assistant', '抱歉，请求失败，请稍后重试。'))
    } finally {
      loading.value = false
    }
  }

  async function loadSessions() {
    try {
      const loadedSessions = await listSessions()
      sessions.value = loadedSessions

      const firstSession = loadedSessions[0]
      if (firstSession) {
        currentSessionId.value = firstSession.id
        currentAgent.value = firstSession.agent
      }
    } catch (error) {
      console.error(error)
    }
  }

  async function startSession(agent: AgentType, question: string) {
    const content = question.trim()
    if (!agent || !content) {
      return
    }

    loading.value = true
    try {
      const session = await requestCreateSession({ agent, message: content })
      sessions.value = [session, ...sessions.value.filter(item => item.id !== session.id)]
      currentSessionId.value = session.id
      currentAgent.value = session.agent
    } catch (error) {
      console.error(error)
    } finally {
      loading.value = false
    }
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

  async function selectSession(sessionId: string) {
    const session = sessions.value.find(item => item.id === sessionId)

    if (!session) {
      return
    }

    currentSessionId.value = session.id
    currentAgent.value = session.agent

    try {
      const loadedSession = await requestGetSession(sessionId)
      const index = sessions.value.findIndex(item => item.id === sessionId)
      if (index !== -1) {
        sessions.value[index] = loadedSession
      }
    } catch (error) {
      console.error(error)
    }
  }

  async function renameSession(sessionId: string, title: string) {
    const normalizedTitle = title.trim()
    const session = sessions.value.find(item => item.id === sessionId)

    if (!session || !normalizedTitle) {
      return
    }

    try {
      const updatedSession = await requestRenameSession(sessionId, normalizedTitle)
      const index = sessions.value.findIndex(item => item.id === sessionId)
      if (index !== -1) {
        sessions.value[index] = updatedSession
      }
    } catch (error) {
      console.error(error)
    }
  }

  async function deleteSession(sessionId: string) {
    const index = sessions.value.findIndex(item => item.id === sessionId)

    if (index === -1) {
      return
    }

    try {
      await requestDeleteSession(sessionId)
      sessions.value.splice(index, 1)

      if (currentSessionId.value !== sessionId) {
        return
      }

      const nextSession = sessions.value[0]
      currentSessionId.value = nextSession?.id ?? ''
      currentAgent.value = nextSession?.agent ?? ''
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
    startSession,
    sendMessage,
    selectSession,
    renameSession,
    deleteSession,
  }
}
