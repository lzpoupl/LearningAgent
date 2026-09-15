import type {
  AgentConfigInput,
  AgentInfo,
  AgentToolPermission,
  AgentToolPermissionInput,
  ToolGroup,
  ToolInfo,
  ToolPermission,
} from '../types/chat'

/** 与 000002.sql 播种的工具目录一致，按 (group, id) 升序。 */
const seedTools: ToolInfo[] = [
  { group: 'anki', id: 'add_card', name: '新增卡片', description: '向牌组新增卡片。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'anki', id: 'add_deck', name: '新增牌组', description: '创建牌组。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'anki', id: 'delete_card', name: '删除卡片', description: '删除单张卡片。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'anki', id: 'get_card', name: '读取卡片', description: '按 id 读取单张卡片。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'anki', id: 'grade_card', name: '作答卡片', description: '记录一次作答并推进复习计划。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'anki', id: 'list_cards', name: '列出卡片', description: '列出牌组下的卡片，可按条件过滤。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'anki', id: 'list_decks', name: '列出牌组', description: '列出指定牌组下的子牌组。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'anki', id: 'move_card', name: '移动卡片', description: '把卡片移动到目标牌组。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'anki', id: 'search_cards', name: '搜索卡片', description: '按关键字搜索卡片。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'anki', id: 'update_card', name: '修改卡片', description: '修改卡片正面或背面。', parameters: {}, returns: {}, defaultPermission: 'ask' },
  { group: 'asset', id: 'list', name: '列出资产', description: '列出可访问的非结构化资产。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'asset', id: 'read', name: '读取资产', description: '读取资产文本内容。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'asset', id: 'search', name: '检索资产', description: '在知识库中检索资产内容。', parameters: {}, returns: {}, defaultPermission: 'allow' },
  { group: 'user', id: 'ask_question', name: '反问用户', description: '向用户提出澄清问题并等待回答。', parameters: {}, returns: {}, defaultPermission: 'allow' },
]

export const seedAgents: AgentInfo[] = [
  {
    id: 1,
    name: '数学 Agent',
    description: '数学问题、公式推导与解题思路',
    icon: '∑',
    color: 'linear-gradient(135deg, #438fff, #5c6df5)',
    builtin: true,
  },
  {
    id: 2,
    name: '英语 Agent',
    description: '英语词汇、语法、翻译与口语练习',
    icon: 'En',
    color: 'linear-gradient(135deg, #26b7bf, #16a5a8)',
    builtin: true,
  },
  {
    id: 3,
    name: '操作系统 Agent',
    description: '课程知识、知识点总结和习题解析。',
    icon: 'terminal',
    color: 'linear-gradient(135deg, #397bd9, #2654bd)',
    builtin: false,
  },
]

function toolId(tool: ToolInfo): string {
  return `${tool.group}.${tool.id}`
}

function cloneTool(tool: ToolInfo): ToolInfo {
  return { ...tool, parameters: { ...tool.parameters }, returns: { ...tool.returns } }
}

function cloneAgent(agent: AgentInfo): AgentInfo {
  return { ...agent }
}

/** 内置 Agent 的初始授权直接采用各工具的默认级别。 */
function defaultPermissions(): Map<string, ToolPermission> {
  const map = new Map<string, ToolPermission>()
  for (const tool of seedTools) {
    map.set(toolId(tool), tool.defaultPermission)
  }
  return map
}

/** 内存版 Agent 后端：Agent 元信息、工具目录与三级工具权限。 */
export class AgentMock {
  private agents = new Map<number, AgentInfo>(seedAgents.map(agent => [agent.id, cloneAgent(agent)]))
  private permissions = new Map<number, Map<string, ToolPermission>>()
  private nextAgentId = seedAgents.length + 1

  constructor() {
    for (const agent of seedAgents) {
      // 内置 Agent 播种默认授权；自定义 Agent 未配置等价于 deny。
      this.permissions.set(agent.id, agent.builtin ? defaultPermissions() : new Map())
    }
  }

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'agent_list':
        return [...this.agents.values()].map(cloneAgent)
      case 'agent_get':
        return cloneAgent(this.requireAgent(Number(payload.agentId ?? 0)))
      case 'agent_create':
        return this.createAgent((payload.input ?? {}) as Partial<AgentConfigInput>)
      case 'agent_update':
        return this.updateAgent(
          Number(payload.agentId ?? 0),
          (payload.input ?? {}) as Partial<AgentConfigInput>,
        )
      case 'agent_delete':
        return this.deleteAgent(Number(payload.agentId ?? 0))
      case 'tool_list':
        return seedTools.map(cloneTool)
      case 'get_all_tool_groups':
        return this.listToolGroups()
      case 'agent_get_tool_permissions':
        return this.getToolPermissions(Number(payload.agentId ?? 0))
      case 'agent_set_tool_permission':
        return this.setToolPermission(
          Number(payload.agentId ?? 0),
          String(payload.toolId ?? ''),
          payload.permission as ToolPermission,
        )
      case 'agent_set_tool_permissions':
        return this.setToolPermissions(
          Number(payload.agentId ?? 0),
          (payload.items ?? []) as AgentToolPermissionInput[],
        )
      case 'agent_resolve_tool_permission':
        return this.resolvePermission(Number(payload.agentId ?? 0), String(payload.toolId ?? ''))
      default:
        return undefined
    }
  }

  private requireAgent(agentId: number): AgentInfo {
    const agent = this.agents.get(agentId)
    if (!agent) {
      throw new Error(`Agent 不存在: ${agentId}`)
    }
    return agent
  }

  private requireTool(id: string): ToolInfo {
    const tool = seedTools.find(item => toolId(item) === id)
    if (!tool) {
      throw new Error(`工具不存在: ${id}`)
    }
    return tool
  }

  private requireUniqueName(name: string, excludeId?: number) {
    if ([...this.agents.values()].some(agent => agent.name === name && agent.id !== excludeId)) {
      throw new Error(`Agent 名称已存在: ${name}`)
    }
  }

  private permissionOf(agentId: number, id: string): ToolPermission {
    return this.permissions.get(agentId)?.get(id) ?? 'deny'
  }

  private createAgent(input: Partial<AgentConfigInput>): AgentInfo {
    const name = String(input.name ?? '').trim()
    const description = String(input.description ?? '').trim()
    if (!name || !description) {
      throw new Error('Agent 配置不完整')
    }
    this.requireUniqueName(name)

    const agent: AgentInfo = {
      id: this.nextAgentId++,
      name,
      description,
      icon: String(input.icon ?? 'AI'),
      color: String(input.color ?? ''),
      builtin: false,
    }
    this.agents.set(agent.id, agent)

    const permissions = new Map<string, ToolPermission>()
    for (const item of input.toolPermissions ?? []) {
      this.requireTool(item.toolId)
      permissions.set(item.toolId, item.permission)
    }
    this.permissions.set(agent.id, permissions)

    return cloneAgent(agent)
  }

  private updateAgent(agentId: number, input: Partial<AgentConfigInput>): AgentInfo {
    const current = this.requireAgent(agentId)
    const name = input.name === undefined ? current.name : String(input.name).trim()
    const description =
      input.description === undefined ? current.description : String(input.description).trim()
    if (!name || !description) {
      throw new Error('Agent 配置不完整')
    }
    this.requireUniqueName(name, agentId)

    const updated: AgentInfo = {
      ...current,
      name,
      description,
      icon: input.icon ?? current.icon,
      color: input.color ?? current.color,
    }
    this.agents.set(agentId, updated)

    if (input.toolPermissions) {
      this.permissions.set(agentId, this.buildPermissionMap(input.toolPermissions))
    }

    return cloneAgent(updated)
  }

  private deleteAgent(agentId: number): void {
    const agent = this.requireAgent(agentId)
    if (agent.builtin) {
      throw new Error('内置 Agent 不可删除')
    }
    this.agents.delete(agentId)
    this.permissions.delete(agentId)
  }

  private listToolGroups(): ToolGroup[] {
    const groups = new Map<string, number>()
    for (const tool of seedTools) {
      groups.set(tool.group, (groups.get(tool.group) ?? 0) + 1)
    }
    return [...groups.entries()]
      .map(([name, toolCount]) => ({ name, toolCount }))
      .sort((left, right) => left.name.localeCompare(right.name))
  }

  private getToolPermissions(agentId: number): AgentToolPermission[] {
    this.requireAgent(agentId)
    return seedTools.map(tool => ({
      tool: cloneTool(tool),
      permission: this.permissionOf(agentId, toolId(tool)),
    }))
  }

  private resolvePermission(agentId: number, id: string): ToolPermission {
    this.requireAgent(agentId)
    this.requireTool(id)
    return this.permissionOf(agentId, id)
  }

  private setToolPermission(
    agentId: number,
    id: string,
    permission: ToolPermission,
  ): AgentToolPermission {
    this.requireAgent(agentId)
    const tool = this.requireTool(id)
    const map = this.permissions.get(agentId) ?? new Map<string, ToolPermission>()
    map.set(id, permission)
    this.permissions.set(agentId, map)
    return { tool: cloneTool(tool), permission }
  }

  private setToolPermissions(
    agentId: number,
    items: AgentToolPermissionInput[],
  ): AgentToolPermission[] {
    this.requireAgent(agentId)
    const map = this.buildPermissionMap(items)
    this.permissions.set(agentId, map)
    return items.map(item => ({
      tool: cloneTool(this.requireTool(item.toolId)),
      permission: item.permission,
    }))
  }

  private buildPermissionMap(items: AgentToolPermissionInput[]): Map<string, ToolPermission> {
    const map = new Map<string, ToolPermission>()
    for (const item of items) {
      this.requireTool(item.toolId)
      map.set(item.toolId, item.permission)
    }
    return map
  }
}
