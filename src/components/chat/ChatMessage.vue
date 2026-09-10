<template>
  <UserMessage v-if="message.role === 'user'" :content="getTextContent()" />
  <AIMessage v-else :message="message" />
</template>

<script setup lang="ts">
import type { ChatMessage as ChatMessageType } from '../../types/chat'

import AIMessage from './AIMessage.vue'
import UserMessage from './UserMessage.vue'

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
