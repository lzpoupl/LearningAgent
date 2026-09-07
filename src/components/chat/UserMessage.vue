<template>
  <div class="user-message">
    <div class="avatar">
      U
    </div>

    <div class="message-content">
      {{ content }}
    </div>
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
  gap: 12px;
  margin: 20px 0;
}

.avatar {
  width: 34px;
  height: 34px;
  border-radius: 50%;

  display: flex;
  align-items: center;
  justify-content: center;

  background: #333;
  color: white;
  font-size: 13px;
}

.message-content {
  max-width: 70%;
  padding: 11px 15px;

  background: #f1f1f1;
  border-radius: 12px;

  line-height: 1.6;
  font-size: 15px;
  color: #222;
}
</style>