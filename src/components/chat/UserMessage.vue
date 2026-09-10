<template>
  <div class="user-message">
    <div class="message-content">
      {{ content }}
    </div>

    <el-avatar :size="30" class="avatar">W</el-avatar>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  content: string
}>()
</script>

<style scoped>
.user-message {
  display: flex;
  justify-content: flex-end;
  align-items: flex-start;
  gap: 10px;
  margin: 0 0 18px;
}

.avatar {
  flex-shrink: 0;
  background: linear-gradient(135deg, #5b8def, #8069e8);
  color: white;
  font-size: 10px;
}

.message-content {
  max-width: min(72%, 680px);
  padding: 10px 13px;
  border-radius: 11px 11px 3px 11px;
  background: var(--el-color-primary);
  line-height: 1.6;
  font-size: 12px;
  color: #fff;
  white-space: pre-wrap;
}
</style>
