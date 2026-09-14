export interface AgentIconOption {
  key: string
  label: string
}

/** 图标库：key 会存进 Agent 配置的 icon 字段，label 用于选择器上的说明。 */
export const agentIconOptions: AgentIconOption[] = [
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

export const defaultAgentIcon = 'sparkle'

const iconKeys = new Set(agentIconOptions.map(item => item.key))

/** 取图标地址；老的文字图标（如 ∑、En）会返回空串，由调用方回退成文字。 */
export function agentIconUrl(icon: string): string {
  return iconKeys.has(icon) ? `/agent-icons/${icon}.svg` : ''
}

export function agentIconLabel(icon: string): string {
  return agentIconOptions.find(item => item.key === icon)?.label ?? '自定义'
}
