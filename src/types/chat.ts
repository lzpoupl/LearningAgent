export type AgentType = 'math' | 'english'

export interface AgentInfo {
  id: AgentType
  name: string
  description: string
  icon: string
  color: string
  capabilities: string[]
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
