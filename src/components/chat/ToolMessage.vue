<template>
  <div class="tool-message">
    <el-card class="tool-card" :body-style="{ padding: '0' }" shadow="never">
      <div class="tool-header">
        <el-icon class="tool-icon"><Tools /></el-icon>
        <span class="tool-name">{{ message.toolName || '工具调用' }}</span>
        <el-tag :type="ok ? 'success' : 'danger'" size="small">
          {{ ok ? '成功' : '失败' }}
        </el-tag>
      </div>

      <pre v-if="summary">{{ summary }}</pre>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Tools } from '@element-plus/icons-vue'

import type { MessageInfo } from '../../types/chat'

const props = defineProps<{
  message: MessageInfo
}>()

interface ToolPayload {
  ok?: boolean
  result?: unknown
  error?: { code?: string; message?: string }
}

const payload = computed<ToolPayload>(() => {
  try {
    return JSON.parse(props.message.content) as ToolPayload
  } catch {
    return { ok: true, result: props.message.content }
  }
})

const ok = computed(() => payload.value.ok !== false)

const summary = computed(() => {
  const body = ok.value ? payload.value.result : payload.value.error
  if (body === undefined || body === null) {
    return ''
  }
  return typeof body === 'string' ? body : JSON.stringify(body, null, 2)
})
</script>

<style scoped>
.tool-message {
  margin: 0 0 18px 40px;
}

.tool-card {
  border-color: var(--learning-border);
}

.tool-header {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 9px 12px;
  border-bottom: 1px solid var(--learning-border);
  background: var(--learning-surface-muted);
}

.tool-icon {
  color: var(--el-color-primary);
  font-size: 14px;
}

.tool-name {
  flex: 1;
  overflow: hidden;
  color: var(--learning-text);
  font-size: 11px;
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

pre {
  margin: 0;
  padding: 12px 14px;
  overflow-x: auto;
  color: var(--learning-text-secondary);
  font-size: 10px;
  line-height: 1.6;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
