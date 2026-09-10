<template>
  <div class="ai-message">
    <el-avatar :size="30" class="ai-avatar">AI</el-avatar>

    <div class="ai-content">
      <template v-for="(block, index) in message.content" :key="`${message.id}-${index}`">
        <AITextMessage v-if="block.type === 'text'" :content="block.content || ''" />
        <LatexMessage v-else-if="block.type === 'latex'" :content="block.content || ''" />
        <AIResultMessage
          v-else-if="block.type === 'result'"
          :data="block.data"
          :title="block.title"
        />
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import AIResultMessage from './AIResultMessage.vue'
import AITextMessage from './AITextMessage.vue'
import LatexMessage from './LatexMessage.vue'
import type { ChatMessage } from '../../types/chat'

defineProps<{
  message: ChatMessage
}>()
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
</style>
