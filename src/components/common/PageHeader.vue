<template>
  <header class="page-header">
    <div>
      <span v-if="eyebrow" class="page-eyebrow">{{ eyebrow }}</span>
      <h1>{{ title }}</h1>
      <p v-if="description">{{ description }}</p>
    </div>

    <div v-if="$slots.default" class="page-actions">
      <slot />
    </div>
  </header>
</template>

<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  eyebrow?: string
}>()
</script>

<style scoped>
.page-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 24px;
}

.page-eyebrow {
  color: var(--el-color-primary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
  text-transform: uppercase;
}

h1,
p {
  margin: 0;
}

h1 {
  margin-top: 8px;
  color: var(--learning-text);
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(28px, 4vw, 42px);
  font-weight: 500;
  line-height: 1.12;
}

p {
  margin-top: 9px;
  color: var(--learning-text-secondary);
  font-size: 13px;
}

.page-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

@media (max-width: 680px) {
  .page-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .page-actions {
    width: 100%;
  }
}
</style>
