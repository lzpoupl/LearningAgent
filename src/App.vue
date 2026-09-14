<template>
  <AppLayout :active-view="activeView" @navigate="navigate">
    <Transition name="page" mode="out-in">
      <component
        :is="currentComponent"
        :key="componentKey"
        v-bind="currentComponentProps"
        @start="startNewSession"
        @start-chat="openChatWith"
        @new-session="resetChatSession"
        @select-session="openSession"
        @rename-session="renameChatSession"
        @delete-session="deleteChatSession"
        @send="sendMessage"
        @open-agents="navigate('agents')"
        @open-assets="navigate('assets')"
        @open-anki="navigate('review')"
        @create-card="openCardCreator(undefined, $event)"
        @edit-card="openCardCreator"
        @browse="navigate('anki')"
        @close="closeTransientPage"
      />
    </Transition>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { ElMessage } from 'element-plus'

import AppLayout from './components/layout/AppLayout.vue'
import AgentSession from './pages/AgentSession.vue'
import AgentManager from './pages/AgentManager.vue'
import AnkiCreator from './pages/AnkiCreator.vue'
import AnkiManager from './pages/AnkiManager.vue'
import AnkiReview from './pages/AnkiReview.vue'
import AssetManager from './pages/AssetManager.vue'
import Home from './pages/Home.vue'
import Settings from './pages/Settings.vue'
import Statistics from './pages/Statistics.vue'

import { useChatSessions } from './composables/useChatSessions'
import type { AgentType } from './types/chat'
import type { Card } from './types/anki'
import type { AppView } from './types/navigation'

const pageComponents: Record<AppView, Component> = {
  home: Home,
  agents: AgentManager,
  chat: AgentSession,
  anki: AnkiManager,
  review: AnkiReview,
  assets: AssetManager,
  stats: Statistics,
  settings: Settings,
}

const activeView = ref<AppView>('home')
const transientPage = ref<'anki-creator' | null>(null)
const pendingAgent = ref<AgentType>()
const editingCard = ref<Card | null>(null)
const newCardDeckPath = ref<string>()

const {
  sessions,
  currentSessionId,
  currentSession,
  currentAgent,
  loading: chatLoading,
  startSession,
  sendMessage,
  selectSession,
  renameSession,
  deleteSession,
} = useChatSessions()

const currentComponent = computed(() => {
  if (transientPage.value === 'anki-creator') {
    return AnkiCreator
  }

  return pageComponents[activeView.value]
})

const componentKey = computed(() =>
  transientPage.value ? transientPage.value : activeView.value,
)

const currentComponentProps = computed<Record<string, unknown>>(() => {
  if (transientPage.value === 'anki-creator') {
    return { editingCard: editingCard.value, initialDeckPath: newCardDeckPath.value }
  }

  if (activeView.value === 'chat') {
    return {
      currentAgent: currentAgent.value,
      currentSession: currentSession.value,
      currentSessionId: currentSessionId.value,
      loading: chatLoading.value,
      initialAgent: pendingAgent.value,
      sessions: sessions.value,
    }
  }

  return {}
})

function navigate(view: AppView) {
  transientPage.value = null
  editingCard.value = null
  newCardDeckPath.value = undefined

  if (view === 'chat') {
    clearActiveSession()
    pendingAgent.value = undefined
  }

  activeView.value = view
}

// 回到新会话状态：不挂载任何会话，由左上的助手选择器决定下一步
function clearActiveSession() {
  currentSessionId.value = ''
  currentAgent.value = ''
}

function openChatWith(agent: AgentType) {
  editingCard.value = null
  newCardDeckPath.value = undefined
  transientPage.value = null
  clearActiveSession()
  pendingAgent.value = agent
  activeView.value = 'chat'
}

async function startNewSession(agent: AgentType, question: string) {
  transientPage.value = null
  pendingAgent.value = undefined
  activeView.value = 'chat'
  await startSession(agent, question)
}

function resetChatSession() {
  clearActiveSession()
  pendingAgent.value = undefined
}

async function openSession(sessionId: string) {
  transientPage.value = null
  editingCard.value = null
  newCardDeckPath.value = undefined
  pendingAgent.value = undefined
  activeView.value = 'chat'
  await selectSession(sessionId)
}

async function renameChatSession(sessionId: string, title: string) {
  try {
    await renameSession(sessionId, title)
    ElMessage.success('会话已重命名')
  } catch (error) {
    console.error(error)
    ElMessage.error('会话重命名失败，请重试。')
  }
}

async function deleteChatSession(sessionId: string) {
  try {
    await deleteSession(sessionId)
    ElMessage.success('会话已删除')
  } catch (error) {
    console.error(error)
    ElMessage.error('会话删除失败，请重试。')
  }
}

function openCardCreator(card?: Card, deckPath?: string) {
  activeView.value = 'anki'
  editingCard.value = card ?? null
  newCardDeckPath.value = deckPath
  transientPage.value = 'anki-creator'
}

function closeTransientPage() {
  transientPage.value = null
  editingCard.value = null
  newCardDeckPath.value = undefined
  activeView.value = 'anki'
}
</script>

<style scoped>
.page-enter-active,
.page-leave-active {
  transition: opacity 140ms ease, transform 140ms ease;
}

.page-enter-from {
  opacity: 0;
  transform: translateY(6px);
}

.page-leave-to {
  opacity: 0;
  transform: translateY(-6px);
}
</style>
