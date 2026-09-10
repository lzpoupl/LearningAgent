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
      <div class="context-item">
        <el-icon><Document /></el-icon>
        <div>
          <strong>{{ agent.id === 'math' ? '高等数学基础.pdf' : '考研英语词汇.pdf' }}</strong>
          <span>知识库 · 已启用</span>
        </div>
      </div>
      <div class="context-item">
        <el-icon><Notebook /></el-icon>
        <div>
          <strong>{{ agent.id === 'math' ? '数学错题本' : '英语例句本' }}</strong>
          <span>可读取 / 可写入</span>
        </div>
      </div>
      <div class="context-item">
        <el-icon><Collection /></el-icon>
        <div>
          <strong>{{ agent.id === 'math' ? '数学 Anki' : '英语 Anki' }}</strong>
          <span>可读取 / 可写入</span>
        </div>
      </div>
    </div>

    <div class="permission-list">
      <span class="context-label">Agent 权限</span>
      <span><el-icon><Check /></el-icon>读取学习资料</span>
      <span><el-icon><Check /></el-icon>读取错题</span>
      <span><el-icon><Check /></el-icon>创建 Anki 卡片</span>
    </div>
  </el-card>
</template>

<script setup lang="ts">
import { Check, Collection, Document, Notebook } from '@element-plus/icons-vue'
import type { AgentInfo } from '../../types/chat'

defineProps<{
  agent: AgentInfo
  hasSession: boolean
}>()
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

.permission-list .el-icon {
  color: #36a26f;
}
</style>
