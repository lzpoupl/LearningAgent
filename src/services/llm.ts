import { invoke } from '@tauri-apps/api/core'

import type { LlmConfigView, LlmProviderInput, ProbeResult } from '../types/chat'

/** LLM 配置视图；密钥已脱敏。 */
export function getLlmConfig(): Promise<LlmConfigView> {
  return invoke<LlmConfigView>('llm_get_config')
}

/** 新增或更新 provider；`apiKey` 缺省表示保留原密钥。 */
export function upsertLlmProvider(input: LlmProviderInput): Promise<LlmConfigView> {
  return invoke<LlmConfigView>('llm_upsert_provider', { input })
}

export function setDefaultLlmProvider(name: string): Promise<LlmConfigView> {
  return invoke<LlmConfigView>('llm_set_default_provider', { name })
}

/** 连通性探测；`name` 缺省使用默认 provider。 */
export function testLlmProvider(name?: string): Promise<ProbeResult> {
  return invoke<ProbeResult>('llm_test_provider', { name: name ?? null })
}
