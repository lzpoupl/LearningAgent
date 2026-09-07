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
  font-size: 15px;
  line-height: 1.8;
  color: #2c2c2c;
  white-space: pre-wrap;
}
</style>