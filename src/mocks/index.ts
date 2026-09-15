import { mockIPC } from '@tauri-apps/api/mocks'
import { AnkiMock } from './anki'
import { AgentMock } from './agents'
import { AssetMock } from './assets'
import { ChatMock } from './chat'
import { SettingsMock } from './settings'
import { StudyMock } from './study'
import { UserMock } from './user'

/** 启用前端 Mock：按领域拦截 Tauri invoke，用内存假数据响应服务请求。 */
export function setupMocks(): void {
  const anki = new AnkiMock()
  const agents = new AgentMock()
  const assets = new AssetMock()
  const chat = new ChatMock()
  const settings = new SettingsMock()
  const study = new StudyMock()
  const user = new UserMock()

  mockIPC((cmd, payload) => {
    const args =
      payload && typeof payload === 'object' && !Array.isArray(payload)
        ? (payload as Record<string, unknown>)
        : {}

    if (cmd.startsWith('anki_')) return anki.handle(cmd, args)
    if (cmd.startsWith('agent_')) return agents.handle(cmd, args)
    if (cmd === 'tool_list' || cmd === 'get_all_tool_groups') return agents.handle(cmd, args)
    if (cmd.startsWith('asset_')) return assets.handle(cmd, args)
    if (cmd.startsWith('bucket_')) return assets.handle(cmd, args)
    if (cmd.startsWith('chat_')) return chat.handle(cmd, args)
    if (cmd.startsWith('settings_')) return settings.handle(cmd, args)
    if (cmd.startsWith('study_')) return study.handle(cmd, args)
    if (cmd.startsWith('user_')) return user.handle(cmd)

    console.warn(`[mock] 未处理的命令: ${cmd}`)
    return undefined
  })

  console.info('[mock] 已启用前端 Mock，服务接口将返回假数据。')
}
