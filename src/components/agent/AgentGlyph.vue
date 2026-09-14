<template>
  <span v-if="url" class="agent-glyph agent-glyph--mask" :style="glyphStyle" />
  <span v-else class="agent-glyph agent-glyph--text" :style="textStyle">{{ icon }}</span>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import { agentIconUrl } from './agentIcons'

const props = withDefaults(defineProps<{
  icon: string
  size?: number
}>(), {
  size: 20,
})

const url = computed(() => agentIconUrl(props.icon))

const glyphStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  maskImage: `url(${url.value})`,
  WebkitMaskImage: `url(${url.value})`,
}))

const textStyle = computed(() => ({
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

/* 用图标自身的透明通道做遮罩，颜色跟随 currentColor，深浅主题都能自适应 */
.agent-glyph--mask {
  background-color: currentColor;
  mask-position: center;
  mask-repeat: no-repeat;
  mask-size: contain;
  -webkit-mask-position: center;
  -webkit-mask-repeat: no-repeat;
  -webkit-mask-size: contain;
}

.agent-glyph--text {
  font-weight: 600;
  letter-spacing: -0.02em;
}
</style>
