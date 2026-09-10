export type AgentType = string

export interface AgentConfigInput {
  name: string
  subject: string
  description: string
  capabilities: string[]
}

export interface AgentPermission {
  key: string
  label: string
  description: string
  enabled: boolean
}

export interface AgentContextAsset {
  id: string
  name: string
  type: 'document' | 'collection' | 'deck'
  access: string
}

export interface AgentContext {
  assets: AgentContextAsset[]
  permissions: AgentPermission[]
}

export interface AgentInfo {
  id: AgentType
  name: string
  description: string
  icon: string
  color: string
  capabilities: string[]
  subject: string
  enabled: boolean
  builtin: boolean
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
