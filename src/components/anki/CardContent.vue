<template>
  <div v-html="renderedContent" />
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

const renderedContent = computed(() => renderCardContent(props.content))

function renderCardContent(content: string): string {
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
