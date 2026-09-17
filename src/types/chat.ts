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

// ---------- 会话与消息 ----------

/** 消息角色；`system` 为会话创建时固化的系统提示词快照。 */
export type MessageRole = 'system' | 'user' | 'assistant' | 'tool'

/** 消息状态：流式中 / 完成 / 出错 / 被中断。 */
export type MessageStatus = 'streaming' | 'complete' | 'error' | 'interrupted'

/** 助手发起的一次工具调用。 */
export interface ToolCall {
  /** 模型侧调用 id，回填 tool 消息时使用。 */
  id: string
  /** `<group>.<id>` 形式的工具引用。 */
  toolId: string
  arguments: Record<string, unknown>
}

export interface MessageInfo {
  id: number
  sessionId: number
  turnId: string | null
  role: MessageRole
  /** 助手回答是 Markdown 文本，工具结果是一段 JSON 文本。 */
  content: string
  toolCalls: ToolCall[]
  toolCallId: string | null
  /** 工具消息对应的 `<group>.<id>`。 */
  toolName: string | null
  status: MessageStatus
  promptTokens: number | null
  completionTokens: number | null
  createdAt: string
}

export interface SessionInfo {
  id: number
  agentId: AgentType
  title: string
  /** 最近一轮使用的 provider 名；未调用过为 null。 */
  lastProvider: string | null
  /** 最近一轮使用的模型名；未调用过为 null。 */
  lastModel: string | null
  messageCount: number
  lastMessageAt: string | null
  createdAt: string
  updatedAt: string
}

/** 会话 + 全部消息；前端打开会话时一次性读取。 */
export interface SessionDetail {
  session: SessionInfo
  messages: MessageInfo[]
}

export interface StartSessionInput {
  agentId: AgentType
  content: string
}

export interface SendMessageInput {
  sessionId: number
  content: string
}

/** 一次用户输入的受理结果；后续内容通过 Channel 推送。 */
export interface TurnHandle {
  sessionId: number
  turnId: string
}

export type TurnStatus = 'completed' | 'cancelled' | 'failed' | 'step_limit'

/** 工具权限为 `ask` 时的用户裁决。 */
export type ApprovalDecision = 'allow_once' | 'allow_always' | 'deny'

export interface ApiErrorShape {
  code: string
  message: string
}

/** 一轮对话的进度事件；由 Tauri Channel 推送。 */
export type AgentEvent =
  | { type: 'turn-started'; sessionId: number; turnId: string }
  | { type: 'message-delta'; sessionId: number; turnId: string; messageId: number; delta: string }
  | { type: 'message-completed'; sessionId: number; turnId: string; message: MessageInfo }
  | { type: 'tool-call'; sessionId: number; turnId: string; call: ToolCall }
  | {
      type: 'tool-approval-required'
      sessionId: number
      turnId: string
      callId: string
      toolId: string
      arguments: Record<string, unknown>
    }
  | {
      type: 'tool-result'
      sessionId: number
      turnId: string
      callId: string
      toolId: string
      ok: boolean
      result?: unknown
      error?: ApiErrorShape
    }
  | { type: 'turn-ended'; sessionId: number; turnId: string; status: TurnStatus; error?: ApiErrorShape }

/** 等待用户裁决的工具调用。 */
export interface PendingApproval {
  callId: string
  toolId: string
  arguments: Record<string, unknown>
  turnId: string
}

/** 工具执行的即时结果，用于在气泡内展示状态。 */
export interface ToolCallResult {
  ok: boolean
  result?: unknown
  error?: ApiErrorShape
}

// ---------- 前端视图模型（由 content 文本渲染） ----------

export type ContentType = 'text' | 'latex' | 'result'

export interface ContentBlock {
  type: ContentType
  content?: string
  title?: string
  data?: Record<string, any>
}

// ---------- LLM provider 配置 ----------

export interface LlmProviderView {
  name: string
  baseUrl: string
  model: string
  apiKeyConfigured: boolean
  apiKeyMasked: string
  compressionModel: string | null
}

export interface LlmConfigView {
  defaultProvider: string
  maxSteps: number
  allowStreaming: boolean
  providers: LlmProviderView[]
}

export interface LlmProviderInput {
  name: string
  baseUrl: string
  model: string
  apiKey?: string
  compressionModel?: string
}

export interface ProbeResult {
  provider: string
  model: string
  latencyMs: number
  reply: string
}
