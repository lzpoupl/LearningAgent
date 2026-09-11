<template>
  <main class="session-page">
    <section class="session-layout">
      <el-card class="conversation-card" shadow="never">
        <template #header>
          <div class="conversation-header">
            <div class="agent-heading">
              <el-avatar v-if="currentAgentInfo" :size="38" :style="{ background: currentAgentInfo.color }">
                {{ currentAgentInfo.icon }}
              </el-avatar>
              <el-avatar v-else :size="38">?</el-avatar>
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
          <el-empty v-if="currentMessages.length === 0" :image-size="72" description="从一个问题开始今天的学习">
            <el-button type="primary" plain @click="emit('new-session')">
              开始新的学习
            </el-button>
          </el-empty>

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
            <el-input v-model="inputMessage" :disabled="loading || !currentSession" :rows="2" maxlength="4000"
              placeholder="输入你的问题..." resize="none" show-word-limit type="textarea"
              @keydown.enter.exact.prevent="send" />
            <el-button class="send-button" circle :disabled="!inputMessage.trim() || loading || !currentSession"
              native-type="submit" type="primary">
              <el-icon>
                <Promotion />
              </el-icon>
            </el-button>
          </form>
          <div class="composer-tip">Enter 发送 · Shift + Enter 换行</div>
        </template>
      </el-card>

       <ChatContextPanel v-if="currentAgentInfo" :agent="currentAgentInfo" :has-session="Boolean(currentSession)" />
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { Loading, Plus, Promotion } from '@element-plus/icons-vue'

import ChatContextPanel from '../components/chat/ChatContextPanel.vue'
import ChatMessage from '../components/chat/ChatMessage.vue'
import { getAgent } from '../services/agent'
import type { AgentInfo, AgentType, ChatSession } from '../types/chat'

const props = defineProps<{
  currentAgent: AgentType
  currentSession: ChatSession | null
  loading: boolean
}>()

const emit = defineEmits<{
  'new-session': []
  send: [content: string]
}>()

const inputMessage = ref('')
const messageContainer = ref<HTMLElement | null>(null)

const currentAgentInfo = ref<AgentInfo | null>(null)
const currentMessages = computed(() => props.currentSession?.messages ?? [])
const agentError = ref('')

let agentRequestId = 0

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

  if (!content || props.loading || !props.currentSession) {
    return
  }

  inputMessage.value = ''
  emit('send', content)
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
  agent => void loadAgent(agent),
  { immediate: true },
)
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

.message-list {
  min-height: 0;
  flex: 1;
  padding: 20px 22px;
  overflow-y: auto;
  background: var(--learning-surface-muted);
}

.message-list :deep(.el-empty) {
  height: 100%;
  padding: 20px;
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

  .conversation-header :deep(.el-button) {
    padding-right: 8px;
    padding-left: 8px;
  }
}
</style>
