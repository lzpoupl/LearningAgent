import { invoke } from '@tauri-apps/api/core'

import type {
  AgentConfigInput,
  AgentInfo,
  AgentToolPermission,
  AgentToolPermissionInput,
  AgentType,
  ToolGroup,
  ToolInfo,
  ToolPermission,
} from '../types/chat'

export function listAgents(): Promise<AgentInfo[]> {
  return invoke<AgentInfo[]>('agent_list')
}

export function getAgent(agentId: AgentType): Promise<AgentInfo> {
  return invoke<AgentInfo>('agent_get', { agentId })
}

export function createAgent(input: AgentConfigInput): Promise<AgentInfo> {
  return invoke<AgentInfo>('agent_create', { input })
}

export function updateAgent(agentId: AgentType, input: AgentConfigInput): Promise<AgentInfo> {
  return invoke<AgentInfo>('agent_update', { agentId, input })
}

export function deleteAgent(agentId: AgentType): Promise<void> {
  return invoke<void>('agent_delete', { agentId })
}

/** 工具目录，按工具组、id 升序。 */
export function listTools(): Promise<ToolInfo[]> {
  return invoke<ToolInfo[]>('tool_list')
}

/** 全部工具组及组内工具数量。 */
export function listToolGroups(): Promise<ToolGroup[]> {
  return invoke<ToolGroup[]>('get_all_tool_groups')
}

/** 工具目录全集 + 该 Agent 的生效级别（未配置为 deny）。 */
export function getAgentToolPermissions(agentId: AgentType): Promise<AgentToolPermission[]> {
  return invoke<AgentToolPermission[]>('agent_get_tool_permissions', { agentId })
}

export function setAgentToolPermission(
  agentId: AgentType,
  toolId: string,
  permission: ToolPermission,
): Promise<AgentToolPermission> {
  return invoke<AgentToolPermission>('agent_set_tool_permission', { agentId, toolId, permission })
}

/** 覆盖式批量设置工具权限。 */
export function setAgentToolPermissions(
  agentId: AgentType,
  items: AgentToolPermissionInput[],
): Promise<AgentToolPermission[]> {
  return invoke<AgentToolPermission[]>('agent_set_tool_permissions', { agentId, items })
}

/** 纯查询某 Agent 对某工具的生效级别。 */
export function resolveAgentToolPermission(
  agentId: AgentType,
  toolId: string,
): Promise<ToolPermission> {
  return invoke<ToolPermission>('agent_resolve_tool_permission', { agentId, toolId })
}
