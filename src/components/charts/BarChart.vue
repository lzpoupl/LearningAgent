<template>
  <div class="bar-chart" role="img" :aria-label="ariaLabel">
    <div class="plot">
      <div class="y-axis">
        <span v-for="tick in ticks" :key="tick">{{ tick }}</span>
      </div>

      <div class="canvas">
        <div class="grid" aria-hidden="true">
          <i v-for="line in [0, 50, 100]" :key="line" :style="{ top: `${line}%` }" />
        </div>

        <div class="scroll">
          <div class="track" :style="{ minWidth }">
            <div class="bars">
              <div
                v-for="day in days"
                :key="day.day"
                class="slot"
                :title="`${day.day}：${day.count}${unit}`"
              >
                <span
                  class="bar"
                  :class="{ empty: day.count === 0 }"
                  :style="barStyle(day.count)"
                />
              </div>
            </div>

            <div class="x-axis">
              <span
                v-for="label in labels"
                :key="label.daysAgo"
                :style="{ left: `${label.percent}%` }"
              >
                {{ label.daysAgo }}
              </span>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

import type { DailyCount } from '../../types/statistics'

const props = withDefaults(
  defineProps<{
    days: DailyCount[]
    color?: string
    unit?: string
    ariaLabel?: string
  }>(),
  {
    color: '#2f9e44',
    unit: '',
    ariaLabel: '按天统计的条形图',
  },
)

const PLOT_HEIGHT = 190

const plotHeight = `${PLOT_HEIGHT}px`
const barColor = computed(() => props.color)
const max = computed(() => Math.max(...props.days.map(day => day.count), 0))

/** 纵轴三条刻度：最大值、中值、0。 */
const ticks = computed(() => {
  if (max.value <= 0) {
    return [0, 0, 0]
  }
  return [max.value, Math.round(max.value / 2), 0]
})

/** 柱数很多时保持最小柱宽并允许横向滚动，柱数少时铺满容器。 */
const minWidth = computed(() => `${props.days.length * 2}px`)

function barStyle(count: number) {
  if (max.value <= 0 || count === 0) {
    return { height: '2px' }
  }
  return { height: `${(count / max.value) * 100}%` }
}

/** 最多 6 个横轴标签，等距取点并显示距今天数。 */
const labels = computed(() => {
  const total = props.days.length
  if (total === 0) {
    return []
  }

  const step = Math.max(1, Math.ceil(total / 6))
  const picked: Array<{ daysAgo: number; percent: number }> = []
  for (let index = total - 1; index >= 0; index -= step) {
    picked.push({
      daysAgo: props.days[index].daysAgo,
      percent: total === 1 ? 0 : (index / (total - 1)) * 100,
    })
  }
  return picked
})
</script>

<style scoped>
.bar-chart {
  --bar-color: v-bind(barColor);
  width: 100%;
}

.plot {
  display: flex;
  gap: 10px;
}

.y-axis {
  display: flex;
  height: v-bind(plotHeight);
  flex-direction: column;
  justify-content: space-between;
  color: var(--learning-text-muted);
  font-size: 10px;
  text-align: right;
}

.canvas {
  position: relative;
  flex: 1;
  min-width: 0;
}

.grid {
  position: absolute;
  top: 0;
  right: 0;
  left: 0;
  height: v-bind(plotHeight);
}

.grid i {
  position: absolute;
  right: 0;
  left: 0;
  border-top: 1px solid var(--learning-border-soft);
}

.scroll {
  overflow-x: auto;
  overflow-y: hidden;
}

.track {
  width: 100%;
}

.bars {
  display: flex;
  height: v-bind(plotHeight);
  align-items: flex-end;
  gap: 1px;
}

.slot {
  display: flex;
  flex: 1 1 0;
  min-width: 2px;
  height: 100%;
  align-items: flex-end;
}

.bar {
  width: 100%;
  border-radius: 2px 2px 0 0;
  background: var(--bar-color);
}

.bar.empty {
  border-radius: 0;
  background: var(--learning-border);
}

.x-axis {
  position: relative;
  height: 22px;
  margin-top: 6px;
}

.x-axis span {
  position: absolute;
  color: var(--learning-text-muted);
  font-size: 10px;
  transform: translateX(-50%);
  white-space: nowrap;
}
</style>