/** Agent 引用：后端 agent.id 为自增整数。 */
export type AgentType = number

/** 工具权限三级：允许 / 询问 / 拒绝。 */
export type ToolPermission = 'allow' | 'ask' | 'deny'

export interface AgentInfo {
  id: AgentType
  name: string
  description: string
  /** 图标 key（对应 public/agent-icons）或文字图标。 */
  icon: string
  /** 图标底色（CSS 渐变）。 */
  color: string
  builtin: boolean
}

/** `<group>.<id>` 形式的工具引用，如 `anki.add_card`。 */
export interface AgentToolPermissionInput {
  toolId: string
  permission: ToolPermission
}

export interface AgentConfigInput {
  name: string
  description: string
  icon?: string
  color?: string
  systemPrompt?: string
  /** 覆盖式设置该 Agent 的工具权限；缺省表示不改动。 */
  toolPermissions?: AgentToolPermissionInput[]
}

export interface ToolInfo {
  group: string
  id: string
  name: string
  description: string
  parameters: Record<string, unknown>
  returns: Record<string, unknown>
  defaultPermission: ToolPermission
}

/** 某 Agent 对某工具的生效权限。 */
export interface AgentToolPermission {
  tool: ToolInfo
  permission: ToolPermission
}

export interface ToolGroup {
  name: string
  toolCount: number
}

export type MessageRole = 'user' | 'assistant'

export type ContentType = 'text' | 'latex' | 'result'

export interface ContentBlock {
  type: ContentType
  content?: string
  title?: string
  data?: Record<string, any>
}

export interface ChatMessage {
  id: string
  role: MessageRole
  content: ContentBlock[]
  time: string
}

export interface ChatSession {
  id: string
  title: string
  agent: AgentType
  messages: ChatMessage[]
  createdAt: string
}
