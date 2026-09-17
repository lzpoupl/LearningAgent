<template>
  <div class="ai-message">
    <el-avatar :size="30" class="ai-avatar">AI</el-avatar>

    <div class="ai-content">
      <template v-for="(block, index) in blocks" :key="`${index}-${block.type}`">
        <AITextMessage v-if="block.type === 'text'" :content="block.content || ''" />
        <LatexMessage v-else-if="block.type === 'latex'" :content="block.content || ''" />
      </template>

      <span v-if="streaming" class="streaming-cursor" aria-hidden="true" />

      <div v-for="call in toolCalls" :key="call.id" class="tool-call" :class="statusClass(call.id)">
        <el-icon class="tool-call-icon"><Tools /></el-icon>
        <span class="tool-call-name">{{ call.toolId }}</span>
        <span class="tool-call-status">{{ statusText(call.id) }}</span>
        <pre v-if="hasArguments(call)" class="tool-call-args">{{ formatArguments(call) }}</pre>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Tools } from '@element-plus/icons-vue'

import AITextMessage from './AITextMessage.vue'
import LatexMessage from './LatexMessage.vue'
import { toContentBlocks } from './content'
import type { ToolCall, ToolCallResult } from '../../types/chat'

const props = defineProps<{
  content: string
  streaming?: boolean
  toolCalls?: ToolCall[]
  toolResults?: Record<string, ToolCallResult>
}>()

const blocks = computed(() => toContentBlocks(props.content))

function hasArguments(call: ToolCall): boolean {
  return Object.keys(call.arguments ?? {}).length > 0
}

function formatArguments(call: ToolCall): string {
  return JSON.stringify(call.arguments ?? {}, null, 2)
}

function statusText(callId: string): string {
  const result = props.toolResults?.[callId]
  if (!result) {
    return '执行中'
  }
  if (result.ok) {
    return '已完成'
  }
  return result.error?.message ?? '执行失败'
}

function statusClass(callId: string): string {
  const result = props.toolResults?.[callId]
  if (!result) {
    return 'is-pending'
  }
  return result.ok ? 'is-ok' : 'is-error'
}
</script>

<style scoped>
.ai-message {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  margin: 0 0 18px;
}

.ai-avatar {
  flex: 0 0 auto;
  background: linear-gradient(135deg, #438fff, #5c6df5);
  font-size: 10px;
  font-weight: 700;
}

.ai-content {
  min-width: 0;
  max-width: min(78%, 720px);
  padding-top: 2px;
}

.streaming-cursor {
  display: inline-block;
  width: 7px;
  height: 13px;
  margin-left: 2px;
  vertical-align: text-bottom;
  background: var(--el-color-primary);
  animation: blink 1s steps(2, start) infinite;
}

@keyframes blink {
  to {
    visibility: hidden;
  }
}

.tool-call {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 6px;
  margin-top: 8px;
  padding: 8px 10px;
  border: 1px solid var(--learning-border);
  border-radius: 8px;
  background: var(--learning-surface-muted);
  font-size: 11px;
}

.tool-call-icon {
  color: var(--el-color-primary);
  font-size: 13px;
}

.tool-call-name {
  overflow: hidden;
  color: var(--learning-text);
  font-weight: 600;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tool-call-status {
  color: var(--learning-text-muted);
  font-size: 10px;
}

.tool-call.is-ok .tool-call-status {
  color: #36a26f;
}

.tool-call.is-error .tool-call-status {
  color: var(--el-color-danger);
}

.tool-call-args {
  grid-column: 1 / -1;
  margin: 4px 0 0;
  color: var(--learning-text-secondary);
  font-size: 10px;
  line-height: 1.5;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
