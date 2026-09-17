import type { LlmConfigView, LlmProviderInput, LlmProviderView, ProbeResult } from '../types/chat'

interface StoredProvider {
  baseUrl: string
  model: string
  apiKey: string
  compressionModel: string | null
}

function maskApiKey(apiKey: string): string {
  if (!apiKey) {
    return ''
  }
  if (apiKey.length <= 7) {
    return '***'
  }
  return `${apiKey.slice(0, 3)}***${apiKey.slice(-4)}`
}

/** 内存版 LLM 配置后端；密钥脱敏规则与 Rust 侧保持一致。 */
export class LlmMock {
  private defaultProvider = 'edgee'
  private providers = new Map<string, StoredProvider>([
    [
      'edgee',
      {
        baseUrl: 'https://edgee.io',
        model: 'anthropic/claude-haiku-4-5',
        apiKey: 'sk-mock-0000',
        compressionModel: null,
      },
    ],
  ])

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'llm_get_config':
        return this.view()
      case 'llm_upsert_provider':
        return this.upsert((payload.input ?? {}) as LlmProviderInput)
      case 'llm_set_default_provider':
        return this.setDefault(String(payload.name ?? ''))
      case 'llm_test_provider':
        return this.probe(payload.name)
      default:
        return undefined
    }
  }

  private view(): LlmConfigView {
    const providers: LlmProviderView[] = [...this.providers.entries()]
      .sort(([left], [right]) => left.localeCompare(right))
      .map(([name, provider]) => ({
        name,
        baseUrl: provider.baseUrl,
        model: provider.model,
        apiKeyConfigured: provider.apiKey.length > 0,
        apiKeyMasked: maskApiKey(provider.apiKey),
        compressionModel: provider.compressionModel,
      }))

    return {
      defaultProvider: this.defaultProvider,
      maxSteps: 8,
      allowStreaming: true,
      providers,
    }
  }

  private upsert(input: LlmProviderInput): LlmConfigView {
    const name = input.name.trim()
    if (!name) {
      throw new Error('provider 名称不能为空')
    }
    const existing = this.providers.get(name)
    this.providers.set(name, {
      baseUrl: input.baseUrl.trim(),
      model: input.model.trim(),
      apiKey: input.apiKey?.trim() || existing?.apiKey || '',
      compressionModel: input.compressionModel?.trim() || null,
    })
    return this.view()
  }

  private setDefault(name: string): LlmConfigView {
    if (!this.providers.has(name)) {
      throw new Error(`provider 不存在: ${name}`)
    }
    this.defaultProvider = name
    return this.view()
  }

  private probe(name: unknown): ProbeResult {
    const provider = typeof name === 'string' && name ? name : this.defaultProvider
    const config = this.providers.get(provider)
    if (!config) {
      throw new Error(`provider 不存在: ${provider}`)
    }
    return {
      provider,
      model: config.model,
      latencyMs: 42,
      reply: 'pong',
    }
  }
}
