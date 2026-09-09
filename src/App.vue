<template>
  <div class="app">

    <!-- 左侧栏 -->
    <ChatSidebar
      :current-session="currentSession"
      :sessions="sessions"
      :active-view="activeView"
      :show-history="!showAnkiGenerator && !showAnkiManager && !showReview && !showStatistics && !showMaterials"

      @open-home="openHome"

      @open-chat="openChat"

      @new-chat="openNewSession"

      @open-anki-manager="openAnkiManager"

      @open-anki-review="openAnkiReview"

      @open-statistics="openStatistics"

      @open-materials="openMaterials"

      @select-session="selectSession"

      @rename="renameSession"

      @delete="deleteSession"
    />

    <AnkiGenerator
      v-if="showAnkiGenerator"
      :editing-card="editingCard"
      @close="openAnkiManager"
    />

    <AnkiManager
      v-else-if="showAnkiManager"
      @create-card="openNewCard"
      @edit-card="openCardEditor"
    />

    <AnkiReview
      v-else-if="showReview"
      @browse="openAnkiManager"
      @edit-card="openCardEditor"
    />

    <Statistics
      v-else-if="showStatistics"
    />

    <Materials
      v-else-if="showMaterials"
    />

    <!-- 新会话预备页面 -->
    <NewSession
      v-else-if="showNewSession"
      @start="startNewSession"
    />

    <!-- 右侧 -->
    <main
      v-else
      class="chat-area"
    >

      <!-- 顶部 -->
      <header class="chat-header">

        <div>
          <div class="agent-title">
            {{ currentAgentInfo.name }}
          </div>

          <div class="agent-description">
            {{ currentAgentInfo.description }}
          </div>
        </div>

      </header>

      <!-- 消息 -->
      <section
        ref="messageContainer"
        class="messages"
      >

        <div
          v-if="currentMessages.length === 0"
          class="welcome"
        >
          <div class="welcome-icon">
            {{ currentAgentInfo.icon }}
          </div>

          <h1>
            你好，我是{{ currentAgentInfo.name }}
          </h1>

          <p>
            有什么学习问题都可以问我
          </p>
        </div>

        <ChatMessage
          v-for="message in currentMessages"
          :key="message.id"
          :message="message"
        />

        <div
          v-if="loading"
          class="loading"
        >
          AI 正在思考...
        </div>

      </section>

      <!-- 输入框 -->
      <div class="input-area">

        <div class="input-wrapper">

          <textarea
            v-model="inputMessage"
            placeholder="输入你的问题..."
            rows="1"
            @keydown.enter.exact.prevent="send"
          />

          <button
            class="send-button"
            :disabled="!inputMessage.trim() || loading"
            @click="send"
          >
            ↑
          </button>

        </div>

        <div class="input-tip">
          Enter 发送 · Shift + Enter 换行
        </div>

      </div>

    </main>

  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, ref } from 'vue'

import ChatSidebar from './components/chat/ChatSidebar.vue'
import ChatMessage from './components/chat/ChatMessage.vue'
import AnkiGenerator from './pages/AnkiGenerator.vue'
import AnkiManager from './pages/AnkiManager.vue'
import AnkiReview from './pages/AnkiReview.vue'
import NewSession from './pages/NewSession.vue'
import Statistics from './pages/Statistics.vue'
import Materials from './pages/Materials.vue'

import type {
  AgentType,
  ChatMessage as ChatMessageType,
  ChatSession
} from './types/chat'
import type { Card } from './types/anki'

import { sendMessage } from './services/api'


/* =========================
   页面状态
========================= */

const showNewSession = ref(false)

const showAnkiGenerator = ref(false)

const showAnkiManager = ref(false)

const showReview = ref(false)

const showStatistics = ref(false)

const showMaterials = ref(false)

const editingCard = ref<Card | null>(null)

const activeView = ref<'home' | 'chat' | 'anki' | 'review' | 'statistics' | 'materials'>('home')

const currentAgent = ref<AgentType>('math')

const currentSession = ref('')

const inputMessage = ref('')

const loading = ref(false)

const messageContainer =
  ref<HTMLElement | null>(null)


/* =========================
   会话数据
========================= */

const sessions = ref<ChatSession[]>([])


/* =========================
   当前会话
========================= */

const currentMessages = computed(() => {
  const session = sessions.value.find(
    item => item.id === currentSession.value
  )

  return session?.messages || []
})


/* =========================
   当前 Agent
========================= */

const currentAgentInfo = computed(() => {
  if (currentAgent.value === 'math') {
    return {
      name: '数学老师',
      description: '数学问题与公式讲解',
      icon: '∑'
    }
  }

  return {
    name: '英语老师',
    description: '英语学习与语言辅导',
    icon: 'A'
  }
})


/* =========================
   打开新会话页面
========================= */

function openNewSession() {
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'chat'
  showNewSession.value = true
}

function openAnkiManager() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'anki'
  showAnkiManager.value = true
}

function openAnkiReview() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = true
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'review'
}

function openNewCard() {
  showNewSession.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'anki'
  showAnkiGenerator.value = true
}

function openStatistics() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showMaterials.value = false
  showStatistics.value = true
  editingCard.value = null
  activeView.value = 'statistics'
}

function openMaterials() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = true
  editingCard.value = null
  activeView.value = 'materials'
}

function openHome() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'home'
}

function openChat() {
  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'chat'
}

function openCardEditor(card: Card) {
  showNewSession.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = card
  showAnkiGenerator.value = true
}


/* =========================
   真正创建新会话
========================= */

async function startNewSession(
  agent: AgentType,
  question: string
) {
  const id = `session-${Date.now()}`

  const session: ChatSession = {
    id,

    title:
      question.length > 20
        ? question.substring(0, 20) + '...'
        : question,

    agent,

    createdAt: new Date().toISOString(),

    messages: []
  }

  sessions.value.unshift(session)

  currentSession.value = id

  currentAgent.value = agent

  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'chat'

  /*
   * 把用户第一次的问题
   * 放入新会话
   */
  const userMessage: ChatMessageType = {
    id: `user-${Date.now()}`,

    role: 'user',

    time: new Date().toISOString(),

    content: [
      {
        type: 'text',
        content: question
      }
    ]
  }

  session.messages.push(userMessage)

  /*
   * 第一次问题直接调用 Agent
   */
  await requestAI(session, question)
}


/* =========================
   发送普通消息
========================= */

async function send() {
  const text = inputMessage.value.trim()

  if (!text || loading.value) {
    return
  }

  const session = sessions.value.find(
    item => item.id === currentSession.value
  )

  if (!session) {
    return
  }

  const userMessage: ChatMessageType = {
    id: `user-${Date.now()}`,

    role: 'user',

    time: new Date().toISOString(),

    content: [
      {
        type: 'text',
        content: text
      }
    ]
  }

  session.messages.push(userMessage)

  inputMessage.value = ''

  scrollToBottom()

  await requestAI(session, text)
}


/* =========================
   调用 AI
========================= */

async function requestAI(
  session: ChatSession,
  question: string
) {
  loading.value = true

  try {

    const response = await sendMessage({
      agent: session.agent,

      message: question,

      sessionId: session.id
    })

    session.messages.push(
      response.message
    )

    await scrollToBottom()

  } catch (error) {

    console.error(error)

    session.messages.push({
      id: `error-${Date.now()}`,

      role: 'assistant',

      time: new Date().toISOString(),

      content: [
        {
          type: 'text',
          content:
            '抱歉，请求失败，请稍后重试。'
        }
      ]
    })

  } finally {

    loading.value = false
  }
}


/* =========================
   选择会话
========================= */

function selectSession(
  sessionId: string
) {
  const session = sessions.value.find(
    item => item.id === sessionId
  )

  if (!session) {
    return
  }

  currentSession.value = sessionId

  currentAgent.value = session.agent

  showNewSession.value = false
  showAnkiGenerator.value = false
  showAnkiManager.value = false
  showReview.value = false
  showStatistics.value = false
  showMaterials.value = false
  editingCard.value = null
  activeView.value = 'chat'
}


/* =========================
   重命名
========================= */

function renameSession(
  sessionId: string,
  title: string
) {
  const session = sessions.value.find(
    item => item.id === sessionId
  )

  if (!session) {
    return
  }

  session.title = title
}


/* =========================
   删除
========================= */

function deleteSession(
  sessionId: string
) {
  const index = sessions.value.findIndex(
    item => item.id === sessionId
  )

  if (index === -1) {
    return
  }

  sessions.value.splice(index, 1)

  /*
   * 如果删除的是当前会话
   */
  if (currentSession.value === sessionId) {

    if (sessions.value.length > 0) {

      currentSession.value =
        sessions.value[0].id

      currentAgent.value =
        sessions.value[0].agent

    } else {

      currentSession.value = ''

      showNewSession.value = true
    }
  }
}


/* =========================
   滚动到底部
========================= */

async function scrollToBottom() {
  await nextTick()

  if (!messageContainer.value) {
    return
  }

  messageContainer.value.scrollTop =
    messageContainer.value.scrollHeight
}
</script>

<style>
* {
  box-sizing: border-box;
}

html,
body,
#app {
  width: 100%;
  height: 100%;
  margin: 0;
  overflow: hidden;
}

body {
  font-family:
    Inter,
    -apple-system,
    BlinkMacSystemFont,
    "Segoe UI",
    sans-serif;

  color: #222;
}

button,
textarea {
  font-family: inherit;
}

/* =========================
   聊天页面
========================= */

.app {
  width: 100%;
  height: 100vh;
  overflow: hidden;

  display: flex;

  background: #ffffff;
}

/* =========================
   右侧聊天区域
========================= */

.chat-area {
  flex: 1;
  min-width: 0;

  display: flex;
  flex-direction: column;

  position: relative;

  background: #ffffff;
}

/* =========================
   顶部
========================= */

.chat-header {
  height: 64px;
  flex-shrink: 0;

  display: flex;
  align-items: center;

  padding: 0 30px;

  border-bottom: 1px solid #eeeeee;

  background: #ffffff;
}

.agent-title {
  font-size: 16px;
  font-weight: 600;

  color: #222222;
}

.agent-description {
  margin-top: 3px;

  font-size: 12px;

  color: #999999;
}

/* =========================
   消息区域
========================= */

.messages {
  flex: 1;

  overflow-y: auto;

  width: 100%;
  max-width: 900px;

  margin: 0 auto;

  padding: 30px 40px 150px;
}

/* =========================
   空聊天欢迎页面
========================= */

.welcome {
  height: 100%;

  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;

  color: #666666;
}

.welcome-icon {
  width: 64px;
  height: 64px;

  display: flex;
  align-items: center;
  justify-content: center;

  margin-bottom: 20px;

  border-radius: 18px;

  background: #111111;

  color: #ffffff;

  font-size: 26px;
}

.welcome h1 {
  margin: 0 0 8px;

  font-size: 24px;
  font-weight: 600;

  color: #222222;
}

.welcome p {
  margin: 0;

  font-size: 14px;

  color: #999999;
}

/* =========================
   AI 加载状态
========================= */

.loading {
  margin: 20px 46px;

  font-size: 14px;

  color: #999999;
}

/* =========================
   输入区域
========================= */

.input-area {
  position: absolute;

  left: 50%;
  bottom: 16px;

  transform: translateX(-50%);

  width: min(800px, calc(100% - 80px));
}

/* 输入框主体 */

.input-wrapper {
  display: flex;
  align-items: flex-end;

  padding: 8px 10px 8px 14px;

  background: #ffffff;

  border: 1px solid #d9d9d9;

  border-radius: 12px;

  box-shadow:
    0 4px 20px rgba(0, 0, 0, 0.08);
}

/* 输入框 */

textarea {
  flex: 1;

  min-height: 20px;
  height: clamp(36px, 6vh, 70px);
  max-height: 92px;

  resize: none;

  border: none;
  outline: none;

  background: transparent;

  font-size: 14px;

  line-height: 1.5;

  color: #222222;
}

textarea::placeholder {
  color: #aaaaaa;
}

/* 发送按钮 */

.send-button {
  width: 34px;
  height: 34px;

  flex-shrink: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  border: none;

  border-radius: 8px;

  background: #111111;

  color: #ffffff;

  font-size: 20px;

  cursor: pointer;
}

.send-button:hover:not(:disabled) {
  background: #333333;
}

.send-button:disabled {
  background: #dddddd;

  color: #ffffff;

  cursor: not-allowed;
}

/* 输入框提示 */

.input-tip {
  margin-top: 6px;

  text-align: center;

  font-size: 11px;

  color: #aaaaaa;
}

/* =========================
   滚动条
========================= */

.messages::-webkit-scrollbar {
  width: 6px;
}

.messages::-webkit-scrollbar-thumb {
  border-radius: 10px;

  background: #dddddd;
}

.messages::-webkit-scrollbar-track {
  background: transparent;
}

/* =========================
   小屏幕适配
========================= */

@media (max-width: 700px) {
  .messages {
    padding-left: 20px;
    padding-right: 20px;
  }

  .chat-header {
    padding: 0 20px;
  }

  .input-area {
    width: calc(100% - 40px);
  }
}
</style>