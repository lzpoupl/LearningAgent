<template>
  <el-card class="result-card" :body-style="{ padding: '0' }" shadow="never">
    <div class="result-header">
      <el-icon class="result-icon"><Tools /></el-icon>
      <span>{{ title || 'AI调用结果' }}</span>
    </div>

    <pre>{{ formattedData }}</pre>
  </el-card>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { Tools } from '@element-plus/icons-vue'

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
  margin-top: 10px;
  border-color: var(--learning-border);
}

.result-header {
  padding: 9px 12px;
  background: var(--learning-surface-muted);
  border-bottom: 1px solid var(--learning-border);

  display: flex;
  align-items: center;
  gap: 8px;

  font-size: 11px;
  font-weight: 600;
}

.result-icon {
  color: var(--el-color-primary);
  font-size: 14px;
}

pre {
  margin: 0;
  padding: 14px;
  font-size: 10px;
  line-height: 1.6;
  overflow-x: auto;
  color: var(--learning-text-secondary);
}
</style>
