<template>
  <el-card class="history-panel" shadow="never">
    <template #header>
      <strong>历史会话</strong>
    </template>

    <div class="scope-row">
      <span class="scope-label">查看助手</span>
      <el-select v-model="scopeAgent" class="agent-filter" placeholder="选择学习助手" size="small"
        :disabled="!agents.length">
        <el-option v-for="agent in agents" :key="agent.id" :label="agent.name" :value="agent.id">
          <span class="agent-option">
            <AgentIcon :color="agent.color" :icon="agent.icon" :size="18" />
            {{ agent.name }}
          </span>
        </el-option>
      </el-select>
    </div>

    <div v-if="!historySessions.length" class="panel-state">该学习助手还没有历史会话</div>
    <div v-else class="session-list">
      <button v-for="session in historySessions" :key="session.id" class="session-item"
        :class="{ active: session.id === currentSessionId }" type="button"
        @click="emit('select-session', session.id)">
        <strong>{{ session.title }}</strong>
        <span>{{ sessionTime(session.createdAt) }} · {{ session.messages.length }} 条消息</span>
      </button>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import AgentIcon from '../agent/AgentIcon.vue'
import type { AgentInfo, ChatSession } from '../../types/chat'

const props = defineProps<{
  agents: AgentInfo[]
  sessions: ChatSession[]
  currentSessionId: string
  defaultAgentId?: string
}>()

const emit = defineEmits<{
  'select-session': [sessionId: string]
}>()

const scopeAgent = ref(props.defaultAgentId || props.agents[0]?.id || '')

const historySessions = computed(() =>
  props.sessions.filter(session => session.agent === scopeAgent.value)
)

// 右栏只跟随当前打开的会话，左上手选助手不会带动它
watch(
  () => props.defaultAgentId,
  agentId => {
    if (agentId) {
      scopeAgent.value = agentId
    }
  },
)

watch(
  () => props.agents,
  agents => {
    if (!scopeAgent.value && agents.length) {
      scopeAgent.value = agents[0].id
    }
  },
)

function sessionTime(value: string): string {
  const date = new Date(value)

  if (Number.isNaN(date.getTime())) {
    return ''
  }

  const time = `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
  const isToday = date.toDateString() === new Date().toDateString()

  return isToday ? `今天 ${time}` : `${date.getMonth() + 1}月${date.getDate()}日 ${time}`
}
</script>

<style scoped>
.history-panel {
  min-width: 0;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.history-panel :deep(.el-card__header) {
  padding: 16px 17px;
  border-bottom-color: var(--learning-border);
}

.history-panel :deep(.el-card__body) {
  padding: 14px;
}

.history-panel :deep(.el-card__header) strong {
  color: var(--learning-text);
  font-size: 12px;
}

.scope-row {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 12px;
}

.scope-label {
  flex: 0 0 auto;
  color: var(--learning-text-muted);
  font-size: 10px;
}

.agent-filter {
  min-width: 0;
  flex: 1;
}

.agent-option {
  display: inline-flex;
  align-items: center;
  gap: 8px;
}

.panel-state {
  color: var(--learning-text-muted);
  font-size: 10px;
  line-height: 1.6;
}

.session-list {
  display: flex;
  max-height: 320px;
  flex-direction: column;
  gap: 6px;
  overflow-y: auto;
}

.session-item {
  display: flex;
  width: 100%;
  flex-direction: column;
  gap: 3px;
  padding: 9px 10px;
  border: 1px solid var(--learning-border);
  border-radius: 8px;
  background: var(--learning-surface-soft);
  text-align: left;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease;
}

.session-item:hover {
  border-color: #b7d0f8;
  background: #eef5ff;
}

.session-item.active {
  border-color: var(--el-color-primary);
  background: #eaf2ff;
}

.session-item strong {
  overflow: hidden;
  color: var(--learning-text);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.session-item span {
  color: var(--learning-text-muted);
  font-size: 9px;
}
</style>
