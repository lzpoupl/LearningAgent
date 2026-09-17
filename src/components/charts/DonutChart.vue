<template>
  <div
    class="donut"
    :style="{ width: `${size}px`, height: `${size}px` }"
    role="img"
    :aria-label="ariaLabel"
  >
    <svg :width="size" :height="size" :viewBox="`0 0 ${size} ${size}`">
      <circle
        class="donut-track"
        :cx="center"
        :cy="center"
        :r="radius"
        :stroke-width="thickness"
      />
      <circle
        v-for="segment in segments"
        :key="segment.key"
        class="donut-segment"
        :cx="center"
        :cy="center"
        :r="radius"
        :stroke="segment.color"
        :stroke-width="thickness"
        :stroke-dasharray="`${segment.length} ${circumference - segment.length}`"
        :stroke-dashoffset="segment.offset"
      />
    </svg>
    <div class="donut-center">
      <slot />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { DonutSegment } from './chartTypes'

const props = withDefaults(
  defineProps<{
    segments: DonutSegment[]
    size?: number
    thickness?: number
    ariaLabel?: string
  }>(),
  {
    size: 200,
    thickness: 26,
    ariaLabel: '占比环形图',
  },
)

const center = computed(() => props.size / 2)
const radius = computed(() => (props.size - props.thickness) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)

/** 按数值占比把圆环切成若干段，段与段之间留出 2px 间隙。 */
const segments = computed(() => {
  const visible = props.segments.filter(segment => segment.value > 0)
  const total = visible.reduce((sum, segment) => sum + segment.value, 0)
  if (total <= 0) {
    return []
  }

  const gap = visible.length > 1 ? 2 : 0
  let consumed = 0

  return visible.map(segment => {
    const raw = (segment.value / total) * circumference.value
    const length = Math.max(raw - gap, 1)
    const piece = {
      key: segment.key,
      color: segment.color,
      length,
      offset: -consumed,
    }
    consumed += raw
    return piece
  })
})
</script>

<style scoped>
.donut {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}

.donut svg {
  transform: rotate(-90deg);
}

.donut-track {
  fill: none;
  stroke: var(--learning-border-soft);
}

.donut-segment {
  fill: none;
  stroke-linecap: butt;
  transition: stroke-dasharray 200ms ease;
}

.donut-center {
  position: absolute;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  color: var(--learning-text);
  text-align: center;
}
</style>