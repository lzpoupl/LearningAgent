<template>
  <div
    ref="host"
    class="echart"
    :style="{ height }"
    role="img"
    :aria-label="ariaLabel"
  />
</template>

<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { echarts, type EChartsInstance, type EChartsOption } from './echarts'

const props = withDefaults(
  defineProps<{
    option: EChartsOption
    height?: string
    ariaLabel?: string
  }>(),
  {
    height: '220px',
    ariaLabel: '统计图表',
  },
)

const host = ref<HTMLDivElement | null>(null)
let chart: EChartsInstance | null = null
let resizeObserver: ResizeObserver | null = null

/** 整体替换配置，避免范围切换时残留上一份数据。 */
function draw() {
  chart?.setOption(props.option, true)
}

onMounted(() => {
  if (!host.value) {
    return
  }

  chart = echarts.init(host.value)
  draw()

  resizeObserver = new ResizeObserver(() => chart?.resize())
  resizeObserver.observe(host.value)
})

watch(() => props.option, draw)

onBeforeUnmount(() => {
  resizeObserver?.disconnect()
  resizeObserver = null
  chart?.dispose()
  chart = null
})
</script>

<style scoped>
.echart {
  width: 100%;
  min-width: 0;
}
</style>