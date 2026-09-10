<template>
  <AppLayout
    v-model:collapsed="sidebarCollapsed"
    :active-view="activeView"
    :current-session="currentSessionId"
    :sessions="sessions"
    :show-history="showHistory"
    @navigate="navigate"
    @new-session="openNewSession()"
    @select-session="selectSession"
    @rename="renameSession"
    @delete="deleteSession"
  >
    <Transition name="page" mode="out-in">
      <component
        :is="currentComponent"
        :key="componentKey"
        v-bind="currentComponentProps"
        @start="startNewSession"
        @start-chat="startChat"
        @new-session="openNewSession()"
        @send="sendMessage"
        @open-agents="navigate('agents')"
        @open-assets="navigate('assets')"
        @open-anki="navigate('review')"
        @create-card="openCardCreator()"
        @edit-card="openCardCreator"
        @browse="navigate('anki')"
        @close="closeTransientPage"
      />
    </Transition>
  </AppLayout>
</template>

<script setup lang="ts">
import { computed, ref, type Component } from 'vue'

import AppLayout from './components/layout/AppLayout.vue'
import AgentSession from './pages/AgentSession.vue'
import AgentManager from './pages/AgentManager.vue'
import AnkiCreator from './pages/AnkiCreator.vue'
import AnkiManager from './pages/AnkiManager.vue'
import AnkiReview from './pages/AnkiReview.vue'
import AssetManager from './pages/AssetManager.vue'
import Home from './pages/Home.vue'
import NewSession from './pages/NewSession.vue'
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
const transientPage = ref<'new-session' | 'anki-creator' | null>(null)
const initialAgent = ref<AgentType>('math')
const editingCard = ref<Card | null>(null)
const sidebarCollapsed = ref(false)

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
  if (transientPage.value === 'new-session') {
    return NewSession
  }

  if (transientPage.value === 'anki-creator') {
    return AnkiCreator
  }

  return pageComponents[activeView.value]
})

const componentKey = computed(() =>
  transientPage.value ? transientPage.value : activeView.value,
)

const currentComponentProps = computed<Record<string, unknown>>(() => {
  if (transientPage.value === 'new-session') {
    return { initialAgent: initialAgent.value }
  }

  if (transientPage.value === 'anki-creator') {
    return { editingCard: editingCard.value }
  }

  if (activeView.value === 'chat') {
    return {
      currentAgent: currentAgent.value,
      currentSession: currentSession.value,
      loading: chatLoading.value,
    }
  }

  return {}
})

const showHistory = computed(() => activeView.value === 'chat')

function navigate(view: AppView) {
  transientPage.value = null
  editingCard.value = null
  activeView.value = view
}

function openNewSession(agent: AgentType = 'math') {
  activeView.value = 'chat'
  initialAgent.value = agent
  editingCard.value = null
  transientPage.value = 'new-session'
}

function startChat(agent: AgentType) {
  openNewSession(agent)
}

async function startNewSession(agent: AgentType, question: string) {
  transientPage.value = null
  activeView.value = 'chat'
  await startSession(agent, question)
}

function openCardCreator(card?: Card) {
  activeView.value = 'anki'
  editingCard.value = card ?? null
  transientPage.value = 'anki-creator'
}

function closeTransientPage() {
  transientPage.value = null
  editingCard.value = null
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
