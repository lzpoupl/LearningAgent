import type { AgentInfo, AgentType } from '../types/chat'

export const agentCatalog: AgentInfo[] = [
  {
    id: 'math',
    name: '数学 Agent',
    description: '数学问题、公式推导与解题思路',
    icon: '∑',
    color: 'linear-gradient(135deg, #438fff, #5c6df5)',
    capabilities: ['题目解析', '知识点拆解', '错题复盘'],
  },
  {
    id: 'english',
    name: '英语 Agent',
    description: '英语词汇、语法、翻译与口语练习',
    icon: 'En',
    color: 'linear-gradient(135deg, #26b7bf, #16a5a8)',
    capabilities: ['词汇讲解', '句子分析', '口语练习'],
  },
]

export function getAgentInfo(agent: AgentType): AgentInfo {
  return agentCatalog.find(item => item.id === agent) ?? agentCatalog[0]
}
