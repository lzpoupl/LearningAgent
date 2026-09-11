<template>
  <div class="latex-container">
    <div ref="latexRef" class="latex-content"></div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onMounted, ref, watch } from 'vue'
import katex from 'katex'
import 'katex/dist/katex.min.css'

const props = defineProps<{
  content: string
}>()

const latexRef = ref<HTMLElement | null>(null)

const renderLatex = async () => {
  await nextTick()

  if (!latexRef.value) return

  try {
    katex.render(props.content, latexRef.value, {
      throwOnError: false,
      displayMode: true
    })
  } catch (error) {
    console.error('LaTeX render error:', error)
    latexRef.value.textContent = props.content
  }
}

onMounted(renderLatex)

watch(
  () => props.content,
  renderLatex
)
</script>

<style scoped>
.latex-container {
  margin: 12px 0;
  padding: 16px;
  background: var(--learning-surface-muted);
  border-radius: 8px;
  overflow-x: auto;
}

.latex-content {
  text-align: center;
}
</style>