<template>
  <UserMessage v-if="message.role === 'user'" :content="message.content" />
  <ToolMessage v-else-if="message.role === 'tool'" :message="message" />
  <AIMessage
    v-else
    :content="message.content"
    :streaming="message.status === 'streaming'"
    :tool-calls="message.toolCalls"
    :tool-results="toolResults"
  />
</template>

<script setup lang="ts">
import type { MessageInfo, ToolCallResult } from '../../types/chat'

import AIMessage from './AIMessage.vue'
import ToolMessage from './ToolMessage.vue'
import UserMessage from './UserMessage.vue'

defineProps<{
  message: MessageInfo
  toolResults?: Record<string, ToolCallResult>
}>()
</script>
