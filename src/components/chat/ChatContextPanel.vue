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
      <div v-for="session in historySessions" :key="session.id" class="session-item"
        :class="{ active: session.id === currentSessionId }" role="button" tabindex="0"
        @click="emit('select-session', session.id)" @keydown.enter.prevent="emit('select-session', session.id)">
        <div class="session-main">
          <strong>{{ session.title }}</strong>
          <span>{{ sessionTime(session.createdAt) }} · {{ session.messages.length }} 条消息</span>
        </div>

        <el-dropdown class="session-actions" trigger="click" @command="command => handleCommand(command, session)"
          @click.stop>
          <button class="session-more" type="button" :title="`${session.title} 的更多操作`">
            <el-icon>
              <MoreFilled />
            </el-icon>
          </button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="rename">
                <el-icon>
                  <EditPen />
                </el-icon>
                重命名
              </el-dropdown-item>
              <el-dropdown-item command="delete" divided>
                <el-icon>
                  <Delete />
                </el-icon>
                删除
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </div>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { ElMessageBox } from 'element-plus'
import { Delete, EditPen, MoreFilled } from '@element-plus/icons-vue'

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
  'rename-session': [sessionId: string, title: string]
  'delete-session': [sessionId: string]
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

function handleCommand(command: string, session: ChatSession) {
  if (command === 'rename') {
    void renameSession(session)
    return
  }

  if (command === 'delete') {
    void removeSession(session)
  }
}

async function renameSession(session: ChatSession) {
  try {
    const result = await ElMessageBox.prompt('输入新的会话名称', '重命名会话', {
      confirmButtonText: '保存',
      cancelButtonText: '取消',
      inputValue: session.title,
      inputPlaceholder: '会话名称',
      inputValidator: value => (value && value.trim() ? true : '会话名称不能为空'),
    })

    const title = String(result.value ?? '').trim()

    if (title && title !== session.title) {
      emit('rename-session', session.id, title)
    }
  } catch {
    // 取消重命名时保持原样
  }
}

async function removeSession(session: ChatSession) {
  try {
    await ElMessageBox.confirm(`删除后“${session.title}”的对话记录将不再显示。`, '删除会话', {
      confirmButtonText: '删除',
      cancelButtonText: '取消',
      type: 'warning',
    })

    emit('delete-session', session.id)
  } catch {
    // 取消删除时保持原样
  }
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
  align-items: center;
  gap: 8px;
  padding: 9px 10px;
  border: 1px solid var(--learning-border);
  border-radius: 8px;
  background: var(--learning-surface-soft);
  text-align: left;
  cursor: pointer;
  transition: border-color 140ms ease, background 140ms ease;
}

.session-item:focus-visible {
  outline: 2px solid var(--el-color-primary);
  outline-offset: 1px;
}

.session-item:hover {
  border-color: #b7d0f8;
  background: #eef5ff;
}

.session-main {
  display: flex;
  min-width: 0;
  flex: 1;
  flex-direction: column;
  gap: 3px;
}

.session-more {
  display: inline-flex;
  width: 22px;
  height: 22px;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--learning-text-muted);
  cursor: pointer;
  opacity: 0.75;
  transition: background 140ms ease, color 140ms ease, opacity 140ms ease;
}

.session-item:hover .session-more {
  opacity: 1;
}

.session-more:hover {
  background: rgba(40, 125, 245, 0.12);
  color: var(--el-color-primary);
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
