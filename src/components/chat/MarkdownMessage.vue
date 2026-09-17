<template>
  <div class="markdown-message" v-html="renderedContent" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import DOMPurify from 'dompurify'
import katex from 'katex'
import 'katex/dist/katex.min.css'
import { marked } from 'marked'

const props = defineProps<{
  content: string
}>()

const renderedContent = computed(() => renderMarkdown(props.content))

/**
 * 把 Markdown 与 LaTeX 统一渲染成 HTML：
 * `$$...$$` 视为块级公式，`$...$` 视为行内公式，其余交给 marked。
 */
function renderMarkdown(content: string): string {
  const formulas: string[] = []

  const replaceFormula = (formula: string, displayMode: boolean) => {
    const index = formulas.length
    formulas.push(
      katex.renderToString(formula.trim(), {
        displayMode,
        throwOnError: false,
      }),
    )
    return `LEARNING_AGENT_FORMULA_${index}`
  }

  const withPlaceholders = content
    .replace(/\$\$([\s\S]*?)\$\$/g, (_, formula: string) => replaceFormula(formula, true))
    .replace(/\$([^$\n]+?)\$/g, (_, formula: string) => replaceFormula(formula, false))

  let html = marked.parse(withPlaceholders, { async: false })
  formulas.forEach((formula, index) => {
    html = html.replace(`LEARNING_AGENT_FORMULA_${index}`, formula)
  })

  return DOMPurify.sanitize(html)
}
</script>

<style scoped>
.markdown-message {
  padding: 11px 13px;
  border: 1px solid var(--learning-border);
  border-radius: 3px 11px 11px 11px;
  background: var(--learning-surface);
  box-shadow: 0 2px 8px rgba(35, 75, 130, 0.03);
  font-size: 12px;
  line-height: 1.7;
  color: var(--learning-text);
  overflow-wrap: anywhere;
}

.markdown-message :deep(p) {
  margin: 0 0 8px;
}

.markdown-message :deep(p:last-child) {
  margin-bottom: 0;
}

.markdown-message :deep(h1),
.markdown-message :deep(h2),
.markdown-message :deep(h3),
.markdown-message :deep(h4) {
  margin: 12px 0 6px;
  font-size: 13px;
  line-height: 1.5;
}

.markdown-message :deep(h1:first-child),
.markdown-message :deep(h2:first-child),
.markdown-message :deep(h3:first-child),
.markdown-message :deep(h4:first-child) {
  margin-top: 0;
}

.markdown-message :deep(ul),
.markdown-message :deep(ol) {
  margin: 0 0 8px;
  padding-left: 20px;
}

.markdown-message :deep(li) {
  margin: 2px 0;
}

.markdown-message :deep(code) {
  padding: 1px 4px;
  border-radius: 4px;
  background: var(--learning-surface-muted);
  font-size: 11px;
}

.markdown-message :deep(pre) {
  margin: 8px 0;
  padding: 10px;
  border-radius: 6px;
  background: var(--learning-surface-muted);
  overflow-x: auto;
}

.markdown-message :deep(pre code) {
  padding: 0;
  background: transparent;
}

.markdown-message :deep(blockquote) {
  margin: 8px 0;
  padding: 2px 0 2px 10px;
  border-left: 3px solid var(--learning-border);
  color: var(--learning-text-secondary);
}

.markdown-message :deep(a) {
  color: var(--el-color-primary);
}

.markdown-message :deep(table) {
  border-collapse: collapse;
  margin: 8px 0;
}

.markdown-message :deep(th),
.markdown-message :deep(td) {
  padding: 4px 8px;
  border: 1px solid var(--learning-border);
}

.markdown-message :deep(.katex-display) {
  margin: 8px 0;
  overflow-x: auto;
  overflow-y: hidden;
}

.markdown-message :deep(img) {
  max-width: 100%;
}
</style>
