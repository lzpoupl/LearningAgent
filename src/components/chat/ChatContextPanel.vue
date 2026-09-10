<template>
  <el-card class="context-panel" shadow="never">
    <template #header>
      <strong>当前学习上下文</strong>
    </template>

    <div class="context-agent">
      <el-avatar :size="34" :style="{ background: agent.color }">{{ agent.icon }}</el-avatar>
      <div>
        <strong>{{ agent.name }}</strong>
        <span>{{ hasSession ? '会话已连接' : '等待新的会话' }}</span>
      </div>
    </div>

    <div class="context-section">
      <span class="context-label">可用学习资产</span>
      <div v-if="contextLoading" class="context-state">正在加载学习资产...</div>
      <div v-else-if="contextError" class="context-state error-state" role="alert">{{ contextError }}</div>
      <template v-else-if="context?.assets.length">
        <div v-for="asset in context.assets" :key="asset.id" class="context-item">
          <el-icon>
            <Document v-if="asset.type === 'document'" />
            <Notebook v-else-if="asset.type === 'collection'" />
            <Collection v-else />
          </el-icon>
          <div>
            <strong>{{ asset.name }}</strong>
            <span>{{ asset.access }}</span>
          </div>
        </div>
      </template>
      <div v-else class="context-state">暂无可用学习资产</div>
    </div>

    <div class="permission-list">
      <span class="context-label">Agent 权限</span>
      <span v-for="permission in context?.permissions ?? []" :key="permission.key" :class="{ disabled: !permission.enabled }">
        <el-icon><Check v-if="permission.enabled" /><Close v-else /></el-icon>
        {{ permission.label }}
      </span>
      <span v-if="!contextLoading && !context?.permissions.length" class="context-state">暂无权限配置</span>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { Check, Close, Collection, Document, Notebook } from '@element-plus/icons-vue'
import { getAgentContext } from '../../services/agent'
import type { AgentContext, AgentInfo } from '../../types/chat'

const props = defineProps<{
  agent: AgentInfo
  hasSession: boolean
}>()

const context = ref<AgentContext | null>(null)
const contextLoading = ref(false)
const contextError = ref('')
let contextRequestId = 0

async function loadContext(agentId: string) {
  const requestId = ++contextRequestId
  contextLoading.value = true
  contextError.value = ''

  try {
    const loadedContext = await getAgentContext(agentId)
    if (requestId === contextRequestId) {
      context.value = loadedContext
    }
  } catch (error) {
    console.error(error)
    if (requestId === contextRequestId) {
      context.value = null
      contextError.value = '学习上下文加载失败'
    }
  } finally {
    if (requestId === contextRequestId) {
      contextLoading.value = false
    }
  }
}

watch(
  () => props.agent.id,
  agentId => void loadContext(agentId),
  { immediate: true },
)
</script>

<style scoped>
.context-panel {
  min-width: 0;
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.context-panel :deep(.el-card__header) {
  padding: 16px 17px;
  border-bottom-color: var(--learning-border);
}

.context-panel :deep(.el-card__body) {
  padding: 16px;
}

.context-panel :deep(.el-card__header) strong {
  color: var(--learning-text);
  font-size: 12px;
}

.context-agent,
.context-item,
.permission-list > span:not(.context-label) {
  display: flex;
  align-items: center;
}

.context-agent {
  gap: 9px;
  padding-bottom: 15px;
  border-bottom: 1px solid var(--learning-border);
}

.context-agent > div,
.context-item > div {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 3px;
}

.context-agent strong,
.context-item strong {
  overflow: hidden;
  color: var(--learning-text);
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.context-agent span,
.context-item span {
  color: var(--learning-text-muted);
  font-size: 9px;
}

.context-section {
  padding: 15px 0 8px;
}

.context-label {
  display: block;
  margin-bottom: 9px;
  color: var(--learning-text-muted);
  font-size: 9px;
}

.context-state {
  color: var(--learning-text-muted);
  font-size: 10px;
  line-height: 1.5;
}

.error-state {
  color: var(--el-color-danger);
}

.context-item {
  gap: 8px;
  padding: 9px;
  border: 1px solid var(--learning-border);
  border-radius: 8px;
  background: #fafcff;
}

.context-item + .context-item {
  margin-top: 8px;
}

.context-item > .el-icon {
  flex: 0 0 auto;
  color: var(--el-color-primary);
}

.permission-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding-top: 14px;
  border-top: 1px solid var(--learning-border);
}

.permission-list > span:not(.context-label) {
  gap: 5px;
  color: var(--learning-text-secondary);
  font-size: 10px;
}

.permission-list > span.disabled {
  color: var(--learning-text-muted);
}

.permission-list .el-icon {
  color: #36a26f;
}
</style>
