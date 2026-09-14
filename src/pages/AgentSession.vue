<template>
  <main class="session-page">
    <section class="session-layout">
      <el-card class="conversation-card" shadow="never">
        <template v-if="currentMessages.length" #header>
          <div class="conversation-header">
            <div class="agent-heading">
              <AgentIcon v-if="currentAgentInfo" :color="currentAgentInfo.color" :icon="currentAgentInfo.icon"
                :size="38" />
              <el-avatar v-else class="empty-avatar" :size="38" />
              <div>
                <strong>{{ currentAgentInfo?.name ?? currentAgent }}</strong>
                <span><i />{{ agentError || '基于学习资产回答' }}</span>
              </div>
            </div>

            <el-button plain size="small" @click="emit('new-session')">
              <el-icon>
                <Plus />
              </el-icon>
              新会话
            </el-button>
          </div>
        </template>

        <div ref="messageContainer" class="message-list" aria-live="polite">
          <div v-if="currentMessages.length === 0" class="new-session-intro">
            <div class="welcome-mark">✦</div>
            <span class="eyebrow">NEW LEARNING SESSION</span>
            <h1>开始新的学习</h1>
            <p class="subtitle">选择一个学习助手，然后输入你想学习的问题</p>

            <el-select v-model="selectedAgent" class="agent-select" popper-class="agent-select-popper"
              placeholder="请选择学习助手" :disabled="!agents.length">
              <template #prefix>
                <AgentIcon v-if="selectedAgentInfo" :color="selectedAgentInfo.color" :icon="selectedAgentInfo.icon"
                  :size="18" />
              </template>
              <el-option v-for="agent in agents" :key="agent.id" :label="agent.name" :value="agent.id">
                <span class="agent-option-label">
                  <AgentIcon :color="agent.color" :icon="agent.icon" :size="20" />
                  {{ agent.name }}
                </span>
                <span class="agent-option-value">{{ agent.description }}</span>
              </el-option>
            </el-select>
            <p v-if="agentsError" class="intro-error" role="alert">{{ agentsError }}</p>
          </div>

          <ChatMessage v-for="message in currentMessages" :key="message.id" :message="message" />

          <div v-if="loading" class="loading-state">
            <el-icon class="is-loading">
              <Loading />
            </el-icon>
            AI 正在整理答案...
          </div>
        </div>

        <template #footer>
          <form class="composer" @submit.prevent="send">
            <el-input v-model="inputMessage" :disabled="loading" :rows="2" maxlength="4000"
              placeholder="输入你的问题..." resize="none" show-word-limit type="textarea"
              @keydown.enter.exact.prevent="send" />
            <el-button class="send-button" circle :disabled="!inputMessage.trim() || loading || !activeAgent"
              native-type="submit" type="primary">
              <el-icon>
                <Promotion />
              </el-icon>
            </el-button>
          </form>
          <div class="composer-tip">
            <template v-if="activeAgent">Enter 发送 · Shift + Enter 换行</template>
            <template v-else>请先选择学习助手</template>
          </div>
        </template>
      </el-card>

      <ChatContextPanel :agents="agents" :current-session-id="currentSessionId" :default-agent-id="conversationAgent"
        :sessions="sessions" @delete-session="emit('delete-session', $event)"
        @rename-session="(sessionId, title) => emit('rename-session', sessionId, title)"
        @select-session="emit('select-session', $event)" />
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { Loading, Plus, Promotion } from '@element-plus/icons-vue'

import AgentIcon from '../components/agent/AgentIcon.vue'
import ChatContextPanel from '../components/chat/ChatContextPanel.vue'
import ChatMessage from '../components/chat/ChatMessage.vue'
import { getAgent, listAgents } from '../services/agent'
import type { AgentInfo, AgentType, ChatSession } from '../types/chat'

const props = defineProps<{
  currentAgent: AgentType
  currentSession: ChatSession | null
  loading: boolean
  initialAgent?: AgentType
  sessions: ChatSession[]
  currentSessionId: string
}>()

const emit = defineEmits<{
  'new-session': []
  send: [content: string]
  start: [agent: AgentType, content: string]
  'select-session': [sessionId: string]
  'rename-session': [sessionId: string, title: string]
  'delete-session': [sessionId: string]
}>()

const agents = ref<AgentInfo[]>([])
const agentsError = ref('')
const selectedAgent = ref<AgentType>(props.initialAgent || props.currentAgent)
const inputMessage = ref('')
const messageContainer = ref<HTMLElement | null>(null)

const currentAgentInfo = ref<AgentInfo | null>(null)
const currentMessages = computed(() => props.currentSession?.messages ?? [])
const activeAgent = computed(() => (props.currentSession ? props.currentAgent : selectedAgent.value))
const selectedAgentInfo = computed(() => agents.value.find(agent => agent.id === selectedAgent.value) ?? null)
// 右栏只跟随当前打开的会话，左上手选助手不会带动它
const conversationAgent = computed(() => props.currentSession?.agent ?? '')
const agentError = ref('')

let agentRequestId = 0

async function loadAgents() {
  agentsError.value = ''

  try {
    agents.value = (await listAgents()).filter(agent => agent.enabled)
  } catch (error) {
    console.error(error)
    agentsError.value = '学习助手加载失败，请稍后重试。'
    return
  }

  adoptLastConversationAgent()
}

// 没有选中的助手时，沿用最近一次会话使用的助手；没有历史会话则保持空白
function adoptLastConversationAgent() {
  if (selectedAgent.value && agents.value.some(agent => agent.id === selectedAgent.value)) {
    return
  }

  const latestSession = [...props.sessions]
    .sort((left, right) => Date.parse(right.createdAt) - Date.parse(left.createdAt))
    .find(session => agents.value.some(agent => agent.id === session.agent))

  selectedAgent.value = latestSession?.agent ?? ''
}

async function loadAgent(agentId: AgentType) {
  if (!agentId) {
    currentAgentInfo.value = null
    return
  }

  const requestId = ++agentRequestId
  agentError.value = ''

  try {
    const agent = await getAgent(agentId)
    if (requestId === agentRequestId) {
      currentAgentInfo.value = agent
    }
  } catch (error) {
    console.error(error)
    if (requestId === agentRequestId) {
      currentAgentInfo.value = null
      agentError.value = '学习助手信息加载失败'
    }
  }
}

function send() {
  const content = inputMessage.value.trim()
  const agent = activeAgent.value

  if (!content || props.loading || !agent) {
    return
  }

  inputMessage.value = ''

  if (props.currentSession) {
    emit('send', content)
    return
  }

  emit('start', agent, content)
}

async function scrollToBottom() {
  await nextTick()

  if (messageContainer.value) {
    messageContainer.value.scrollTop = messageContainer.value.scrollHeight
  }
}

watch(
  () => [props.currentSession?.id, currentMessages.value.length, props.loading],
  scrollToBottom,
)

watch(
  () => props.currentAgent,
  agent => {
    if (agent) {
      selectedAgent.value = agent
    }
  },
  { immediate: true },
)

watch(
  () => props.initialAgent,
  agent => {
    if (agent) {
      selectedAgent.value = agent
    }
  },
)

watch(
  () => props.sessions.length,
  () => {
    if (!selectedAgent.value) {
      adoptLastConversationAgent()
    }
  },
)

watch(activeAgent, agent => void loadAgent(agent), { immediate: true })

onMounted(loadAgents)
</script>

<style scoped>
.session-page {
  width: 100%;
  height: 100%;
  min-height: 0;
  padding: 18px 24px 24px;
  overflow: hidden;
  background: var(--learning-bg);
}

.session-layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) 260px;
  gap: 16px;
  width: min(1280px, 100%);
  height: 100%;
  min-height: 0;
  margin: 0 auto;
}

.conversation-card {
  display: flex;
  min-width: 0;
  min-height: 0;
  flex-direction: column;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.conversation-card :deep(.el-card__header) {
  padding: 14px 18px;
  border-bottom-color: var(--learning-border);
}

.conversation-card :deep(.el-card__body) {
  display: flex;
  min-height: 0;
  flex: 1;
  flex-direction: column;
  padding: 0;
}

.conversation-card :deep(.el-card__footer) {
  padding: 12px 14px 10px;
  border-top-color: var(--learning-border);
}

.conversation-header,
.agent-heading,
.composer {
  display: flex;
  align-items: center;
}

.conversation-header {
  justify-content: space-between;
  gap: 12px;
}

.agent-heading {
  gap: 10px;
}

.agent-heading>div {
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.agent-heading strong {
  color: var(--learning-text);
  font-size: 13px;
}

.agent-heading span {
  color: #36a26f;
  font-size: 10px;
}

.agent-heading i {
  display: inline-block;
  width: 6px;
  height: 6px;
  margin-right: 4px;
  border-radius: 50%;
  background: currentColor;
}

.empty-avatar {
  background: var(--learning-surface-muted);
  box-shadow: inset 0 0 0 1px var(--learning-border);
  color: transparent;
}

.message-list {
  min-height: 0;
  flex: 1;
  padding: 20px 22px;
  overflow-y: auto;
  background: var(--learning-surface-muted);
}

.new-session-intro {
  display: flex;
  height: 100%;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 24px;
  text-align: center;
}

.welcome-mark {
  display: grid;
  width: 48px;
  height: 48px;
  margin-bottom: 16px;
  place-items: center;
  border-radius: 15px;
  background: linear-gradient(135deg, #438fff, #5c6df5);
  box-shadow: 0 10px 25px rgba(50, 120, 240, 0.22);
  color: #fff;
  font-size: 22px;
}

.eyebrow {
  color: var(--el-color-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.new-session-intro h1 {
  margin: 9px 0 0;
  color: var(--learning-text);
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(26px, 3.4vw, 36px);
  font-weight: 500;
}

.new-session-intro .subtitle {
  margin: 10px 0 0;
  color: var(--learning-text-secondary);
  font-size: 13px;
}

.new-session-intro .agent-select {
  width: min(420px, 100%);
  margin-top: 22px;
}

.agent-option-label {
  display: inline-flex;
  float: left;
  align-items: center;
  gap: 8px;
}

.agent-option-value {
  float: right;
  color: #8492a6;
  font-size: 13px;
}

/* 下拉选项留出图标与右对齐描述的空间 */
:global(.agent-select-popper .el-select-dropdown__item) {
  height: auto;
  min-height: 40px;
  padding: 8px 16px;
  line-height: 1.5;
}

.intro-error {
  margin: 12px 0 0;
  color: var(--el-color-danger);
  font-size: 11px;
}

.loading-state {
  display: flex;
  align-items: center;
  gap: 7px;
  margin: 8px 0 0 46px;
  color: var(--learning-text-muted);
  font-size: 12px;
}

.composer {
  align-items: flex-end;
  gap: 10px;
}

.composer :deep(.el-textarea__inner) {
  min-height: 50px !important;
  padding: 10px 12px;
  border-color: var(--learning-border);
  box-shadow: none;
  font-size: 12px;
  line-height: 1.6;
}

.composer :deep(.el-textarea__inner:focus) {
  border-color: var(--el-color-primary);
}

.composer :deep(.el-input__count) {
  background: transparent;
  font-size: 9px;
}

.send-button {
  flex: 0 0 auto;
}

.composer-tip {
  margin-top: 5px;
  color: var(--learning-text-muted);
  font-size: 10px;
  text-align: center;
}

@media (max-width: 1050px) {
  .session-layout {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 620px) {
  .session-page {
    padding: 10px;
  }

  .message-list {
    padding: 16px 12px;
  }

  .new-session-intro {
    padding: 12px;
  }

  .conversation-header :deep(.el-button) {
    padding-right: 8px;
    padding-left: 8px;
  }
}
</style>
