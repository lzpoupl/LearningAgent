<template>
  <div class="result-card">
    <div class="result-header">
      <span class="result-icon">⚙</span>
      <span>{{ title || 'AI调用结果' }}</span>
    </div>

    <pre>{{ formattedData }}</pre>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  title?: string
  data?: Record<string, any>
}>()

const formattedData = computed(() => {
  if (!props.data) {
    return ''
  }

  return JSON.stringify(props.data, null, 2)
})
</script>

<style scoped>
.result-card {
  margin: 12px 0;
  border: 1px solid #e5e7eb;
  border-radius: 10px;
  overflow: hidden;
  background: #fff;
}

.result-header {
  padding: 10px 14px;
  background: #f5f6f8;
  border-bottom: 1px solid #e5e7eb;

  display: flex;
  align-items: center;
  gap: 8px;

  font-size: 14px;
  font-weight: 600;
}

.result-icon {
  font-size: 16px;
}

pre {
  margin: 0;
  padding: 14px;
  font-size: 13px;
  line-height: 1.6;
  overflow-x: auto;
  color: #444;
}
</style>