<template>
  <span v-if="svg" class="agent-glyph" :style="glyphStyle" v-html="svg" />
  <span v-else class="agent-glyph agent-glyph--text" :style="glyphStyle">{{ icon }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { agentIconSvg } from './agentIcons'

const props = withDefaults(defineProps<{
  icon: string
  size?: number
}>(), {
  size: 20,
})

const svg = computed(() => agentIconSvg(props.icon))

const glyphStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  fontSize: `${Math.round(props.size * 0.72)}px`,
}))
</script>

<style scoped>
.agent-glyph {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  line-height: 1;
}

.agent-glyph :deep(svg) {
  display: block;
  width: 100%;
  height: 100%;
}

.agent-glyph--text {
  font-weight: 600;
  letter-spacing: -0.02em;
}
</style>
