<template>
  <el-avatar class="agent-icon" :shape="shape" :size="size" :style="{ background: color }">
    <AgentGlyph :icon="icon" :size="glyphSize" />
  </el-avatar>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import AgentGlyph from './AgentGlyph.vue'

const props = withDefaults(defineProps<{
  icon: string
  color?: string
  size?: number
  shape?: 'circle' | 'square'
}>(), {
  color: 'linear-gradient(135deg, #9a7bea, #6c5ce7)',
  size: 38,
  shape: 'circle',
})

const glyphSize = computed(() => Math.round(props.size * 0.6))
</script>

<style scoped>
.agent-icon {
  flex: 0 0 auto;
  color: #fff;
}
</style>
