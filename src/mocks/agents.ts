import type {
  AgentConfigInput,
  AgentContext,
  AgentInfo,
  AgentPermission,
} from '../types/chat'

export const seedAgents: AgentInfo[] = [
  {
    id: 'math',
    name: '数学 Agent',
    subject: '数学',
    description: '数学问题、公式推导与解题思路',
    icon: 'integral',
    color: 'linear-gradient(135deg, #438fff, #5c6df5)',
    capabilities: ['题目解析', '知识点拆解', '错题复盘'],
    enabled: true,
    builtin: true,
  },
  {
    id: 'english',
    name: '英语 Agent',
    subject: '英语',
    description: '英语词汇、语法、翻译与口语练习',
    icon: 'language',
    color: 'linear-gradient(135deg, #26b7bf, #16a5a8)',
    capabilities: ['词汇讲解', '句子分析', '口语练习'],
    enabled: true,
    builtin: true,
  },
  {
    id: 'custom-os',
    name: '操作系统 Agent',
    subject: '计算机',
    description: '课程知识、知识点总结和习题解析。',
    icon: 'terminal',
    color: 'linear-gradient(135deg, #397bd9, #2654bd)',
    capabilities: ['课程知识', '知识点总结'],
    enabled: true,
    builtin: false,
  },
]

const seedPermissions: AgentPermission[] = [
  {
    key: 'read-assets',
    label: '读取学习资料',
    description: '允许 Agent 检索 PDF、笔记和课件。',
    enabled: true,
  },
  {
    key: 'read-notes',
    label: '读取错题与笔记',
    description: '允许 Agent 参考你的结构化学习记录。',
    enabled: true,
  },
  {
    key: 'write-anki',
    label: '创建 Anki 卡片',
    description: '允许 Agent 将结论整理为待复习卡片。',
    enabled: true,
  },
]

function cloneAgent(agent: AgentInfo): AgentInfo {
  return { ...agent, capabilities: [...agent.capabilities] }
}

function clonePermissions(permissions: AgentPermission[]): AgentPermission[] {
  return permissions.map(permission => ({ ...permission }))
}

function nowId(prefix: string): string {
  return `${prefix}-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
}

/** 内存版 Agent 后端：业务代码通过 invoke 访问，假数据只在 mock 层维护。 */
export class AgentMock {
  private agents = new Map(seedAgents.map(agent => [agent.id, cloneAgent(agent)]))
  private permissions = clonePermissions(seedPermissions)

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'agent_list':
        return [...this.agents.values()].map(cloneAgent)
      case 'agent_get':
        return cloneAgent(this.requireAgent(String(payload.agentId ?? '')))
      case 'agent_create':
        return this.createAgent((payload.input ?? {}) as Partial<AgentConfigInput>)
      case 'agent_update':
        return this.updateAgent(
          String(payload.agentId ?? ''),
          (payload.input ?? {}) as Partial<AgentConfigInput>,
        )
      case 'agent_delete':
        return this.deleteAgent(String(payload.agentId ?? ''))
      case 'agent_set_enabled':
        return this.setEnabled(String(payload.agentId ?? ''), Boolean(payload.enabled))
      case 'agent_get_context':
        return this.getContext(String(payload.agentId ?? ''))
      case 'agent_get_permissions':
        return clonePermissions(this.permissions)
      case 'agent_update_permissions':
        return this.updatePermissions((payload.permissions ?? []) as AgentPermission[])
      default:
        return undefined
    }
  }

  private requireAgent(agentId: string): AgentInfo {
    const agent = this.agents.get(agentId)
    if (!agent) {
      throw new Error(`Agent 不存在: ${agentId}`)
    }
    return agent
  }

  private createAgent(input: Partial<AgentConfigInput>): AgentInfo {
    const agent = this.buildAgent(input, false)
    this.agents.set(agent.id, agent)
    return cloneAgent(agent)
  }

  private updateAgent(agentId: string, input: Partial<AgentConfigInput>): AgentInfo {
    const current = this.requireAgent(agentId)
    const updated = this.buildAgent({
      name: input.name ?? current.name,
      subject: input.subject ?? current.subject,
      description: input.description ?? current.description,
      capabilities: input.capabilities ?? current.capabilities,
      icon: input.icon ?? current.icon,
      color: input.color ?? current.color,
    }, current.builtin, current)
    this.agents.set(agentId, updated)
    return cloneAgent(updated)
  }

  private buildAgent(
    input: Partial<AgentConfigInput>,
    builtin: boolean,
    current?: AgentInfo,
  ): AgentInfo {
    const name = String(input.name ?? '').trim()
    const subject = String(input.subject ?? '').trim()
    const description = String(input.description ?? '').trim()
    const capabilities = (input.capabilities ?? [])
      .map(capability => capability.trim())
      .filter(Boolean)

    if (!name || !subject || !description) {
      throw new Error('Agent 配置不完整')
    }

    return {
      id: current?.id ?? nowId('agent'),
      name,
      subject,
      description,
      icon: String(input.icon ?? current?.icon ?? 'sparkle'),
      color: String(input.color ?? current?.color ?? 'linear-gradient(135deg, #9a7bea, #6c5ce7)'),
      capabilities,
      enabled: current?.enabled ?? true,
      builtin,
    }
  }

  private deleteAgent(agentId: string): void {
    const agent = this.requireAgent(agentId)
    if (agent.builtin) {
      throw new Error('内置 Agent 不可删除')
    }
    this.agents.delete(agentId)
  }

  private setEnabled(agentId: string, enabled: boolean): AgentInfo {
    const agent = this.requireAgent(agentId)
    agent.enabled = enabled
    return cloneAgent(agent)
  }

  private getContext(agentId: string): AgentContext {
    const agent = this.requireAgent(agentId)
    const assets = agent.id === 'math'
      ? [
          { id: 'asset-math-pdf', name: '高等数学基础.pdf', type: 'document' as const, access: '知识库 · 已启用' },
          { id: 'asset-math-notes', name: '数学错题本', type: 'collection' as const, access: '可读取 / 可写入' },
          { id: 'asset-math-anki', name: '数学 Anki', type: 'deck' as const, access: '可读取 / 可写入' },
        ]
      : agent.id === 'english'
        ? [
            { id: 'asset-english-pdf', name: '考研英语词汇.pdf', type: 'document' as const, access: '知识库 · 已启用' },
            { id: 'asset-english-notes', name: '英语例句本', type: 'collection' as const, access: '可读取 / 可写入' },
            { id: 'asset-english-anki', name: '英语 Anki', type: 'deck' as const, access: '可读取 / 可写入' },
          ]
        : [
            { id: `asset-${agent.id}-documents`, name: `${agent.subject}课程资料`, type: 'document' as const, access: '知识库 · 已启用' },
            { id: `asset-${agent.id}-notes`, name: `${agent.subject}学习笔记`, type: 'collection' as const, access: '可读取 / 可写入' },
            { id: `asset-${agent.id}-anki`, name: `${agent.subject} Anki`, type: 'deck' as const, access: '可读取 / 可写入' },
          ]

    return {
      assets,
      permissions: clonePermissions(this.permissions),
    }
  }

  private updatePermissions(permissions: AgentPermission[]): AgentPermission[] {
    this.permissions = permissions.map(permission => ({ ...permission }))
    return clonePermissions(this.permissions)
  }
}
