const sources = import.meta.glob('../../assets/agent-icons/*.svg', {
  eager: true,
  query: '?raw',
  import: 'default',
}) as Record<string, string>

export interface AgentIconOption {
  key: string
  label: string
  svg: string
}

/** 图标库：key 会存进 Agent 配置的 icon 字段，label 用于选择器上的说明。 */
const catalog: Array<{ key: string; label: string }> = [
  { key: 'integral', label: '积分' },
  { key: 'sum', label: '求和' },
  { key: 'function', label: '函数' },
  { key: 'geometry', label: '几何' },
  { key: 'atom', label: '物理' },
  { key: 'flask', label: '化学' },
  { key: 'dna', label: '生物' },
  { key: 'code', label: '编程' },
  { key: 'terminal', label: '系统' },
  { key: 'globe', label: '地理' },
  { key: 'language', label: '语言' },
  { key: 'book', label: '语文' },
  { key: 'landmark', label: '历史' },
  { key: 'chart', label: '经济' },
  { key: 'music', label: '音乐' },
  { key: 'palette', label: '美术' },
  { key: 'bulb', label: '思维' },
  { key: 'sparkle', label: '通用' },
]

export const agentIconOptions: AgentIconOption[] = catalog
  .map(item => ({ ...item, svg: sources[`../../assets/agent-icons/${item.key}.svg`] ?? '' }))
  .filter(item => item.svg)

export const defaultAgentIcon = 'sparkle'

const iconSvgMap = new Map(agentIconOptions.map(item => [item.key, item.svg]))

/** 取图标源码；老的文字图标（如 ∑、En）会返回空串，由调用方回退成文字。 */
export function agentIconSvg(icon: string): string {
  return iconSvgMap.get(icon) ?? ''
}

export function agentIconLabel(icon: string): string {
  return agentIconOptions.find(item => item.key === icon)?.label ?? '自定义'
}
