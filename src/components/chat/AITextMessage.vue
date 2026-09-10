<template>
  <div class="ai-text">
    {{ content }}
  </div>
</template>

<script setup lang="ts">
defineProps<{
  content: string
}>()
</script>

<style scoped>
.ai-text {
  padding: 11px 13px;
  border: 1px solid var(--learning-border);
  border-radius: 3px 11px 11px 11px;
  background: #fff;
  box-shadow: 0 2px 8px rgba(35, 75, 130, 0.03);
  font-size: 12px;
  line-height: 1.7;
  color: var(--learning-text);
  white-space: pre-wrap;
}
</style>
