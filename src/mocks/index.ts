import { mockIPC } from '@tauri-apps/api/mocks'
import { AnkiMock } from './anki'

/** 启用前端 Mock：拦截 Tauri 的 invoke，用内存假数据响应 Anki 命令。 */
export function setupMocks(): void {
  const anki = new AnkiMock()

  mockIPC((cmd, payload) => {
    const args =
      payload && typeof payload === 'object' && !Array.isArray(payload)
        ? (payload as Record<string, unknown>)
        : {}
    return anki.handle(cmd, args)
  })

  console.info('[mock] 已启用前端 Mock，Anki 接口将返回假数据。')
}
