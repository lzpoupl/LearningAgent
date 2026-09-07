import type { AgentType, ChatMessage } from '../types/chat'

export interface ChatRequest {
  agent: AgentType
  message: string
  sessionId: string
}

export interface ChatResponse {
  message: ChatMessage
}

export async function sendMessage(
  request: ChatRequest
): Promise<ChatResponse> {

  // TODO:
  // 后面替换成真实后端 API

  console.log('发送给后端：', request)

  return {
    message: {
      id: Date.now().toString(),
      role: 'assistant',
      time: new Date().toISOString(),
      content: [
        {
          type: 'text',
          content: '让我来帮你分析这个问题。'
        },
        {
          type: 'latex',
          content: '\\int_0^1 x^2 dx = \\frac{1}{3}'
        },
        {
          type: 'result',
          title: '计算结果',
          data: {
            answer: '1/3',
            status: 'success'
          }
        }
      ]
    }
  }
}