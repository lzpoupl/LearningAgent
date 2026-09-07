<template>
  <!-- 用户消息 -->
  <UserMessage
    v-if="message.role === 'user'"
    :content="getTextContent()"
  />

  <!-- AI 消息 -->
  <div v-else class="ai-message">
    <div class="avatar">
      AI
    </div>

    <div class="ai-content">
      <template
        v-for="(block, index) in message.content"
        :key="index"
      >
        <!-- AI文字 -->
        <AITextMessage
          v-if="block.type === 'text'"
          :content="block.content || ''"
        />

        <!-- LaTeX -->
        <LatexMessage
          v-else-if="block.type === 'latex'"
          :content="block.content || ''"
        />

        <!-- AI调用结果 -->
        <AIResultMessage
          v-else-if="block.type === 'result'"
          :title="block.title"
          :data="block.data"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import type { ChatMessage as ChatMessageType } from '../../types/chat'

import UserMessage from './UserMessage.vue'
import AITextMessage from './AITextMessage.vue'
import LatexMessage from './LatexMessage.vue'
import AIResultMessage from './AIResultMessage.vue'

const props = defineProps<{
  message: ChatMessageType
}>()

const getTextContent = () => {
  return props.message.content
    .filter(item => item.type === 'text')
    .map(item => item.content || '')
    .join('')
}
</script>

<style scoped>
.ai-message {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  margin: 20px 0;
}

.avatar {
  width: 34px;
  height: 34px;
  flex-shrink: 0;

  display: flex;
  align-items: center;
  justify-content: center;

  border-radius: 50%;
  background: #4f46e5;

  color: white;
  font-size: 11px;
  font-weight: 600;
}

.ai-content {
  max-width: 75%;
  padding-top: 6px;
}
</style>