import { invoke } from '@tauri-apps/api/core'

import type {
  AgentConfigInput,
  AgentContext,
  AgentInfo,
  AgentPermission,
  AgentType,
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

export function setAgentEnabled(agentId: AgentType, enabled: boolean): Promise<AgentInfo> {
  return invoke<AgentInfo>('agent_set_enabled', { agentId, enabled })
}

export function getAgentContext(agentId: AgentType): Promise<AgentContext> {
  return invoke<AgentContext>('agent_get_context', { agentId })
}

export function getAgentPermissions(): Promise<AgentPermission[]> {
  return invoke<AgentPermission[]>('agent_get_permissions')
}

export function updateAgentPermissions(permissions: AgentPermission[]): Promise<AgentPermission[]> {
  return invoke<AgentPermission[]>('agent_update_permissions', { permissions })
}
