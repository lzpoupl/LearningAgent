export interface AgentColorPreset {
  key: string
  label: string
  color: string
}

/** 取自现有界面的主色与快捷入口配色，保持整体视觉一致。 */
export const agentColorPresets: AgentColorPreset[] = [
  { key: 'blue', label: '经典蓝', color: 'linear-gradient(135deg, #438fff, #5c6df5)' },
  { key: 'teal', label: '湖水青', color: 'linear-gradient(135deg, #26b7bf, #16a5a8)' },
  { key: 'indigo', label: '深海蓝', color: 'linear-gradient(135deg, #397bd9, #2654bd)' },
  { key: 'violet', label: '紫罗兰', color: 'linear-gradient(135deg, #9a7bea, #6c5ce7)' },
  { key: 'green', label: '森林绿', color: 'linear-gradient(135deg, #46b884, #2d9a6d)' },
  { key: 'amber', label: '琥珀橙', color: 'linear-gradient(135deg, #f0ab4b, #d98b13)' },
]

export const defaultAgentColor = agentColorPresets[0].color
