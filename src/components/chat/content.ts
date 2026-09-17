import type { ContentBlock } from '../../types/chat'

/**
 * 把助手回答的 Markdown 文本拆成文本与公式块：
 * `$$...$$` 视为块级公式，`$...$` 视为行内公式，其余按文本渲染。
 */
export function toContentBlocks(content: string): ContentBlock[] {
  if (!content) {
    return []
  }

  const blocks: ContentBlock[] = []
  const pattern = /\$\$([\s\S]+?)\$\$|\$([^$\n]+?)\$/g
  let lastIndex = 0
  let match: RegExpExecArray | null

  while ((match = pattern.exec(content)) !== null) {
    if (match.index > lastIndex) {
      blocks.push({ type: 'text', content: content.slice(lastIndex, match.index) })
    }
    blocks.push({ type: 'latex', content: (match[1] ?? match[2] ?? '').trim() })
    lastIndex = match.index + match[0].length
  }

  if (lastIndex < content.length) {
    blocks.push({ type: 'text', content: content.slice(lastIndex) })
  }

  return blocks.length ? blocks : [{ type: 'text', content }]
}
