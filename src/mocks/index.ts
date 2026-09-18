import { mockIPC } from '@tauri-apps/api/mocks'
import { AnkiMock } from './anki'
import { AgentMock } from './agents'
import { AssetMock } from './assets'
import { ChatMock } from './chat'
import { LlmMock } from './llm'
import { SettingsMock } from './settings'
import { StatisticsMock } from './statistics'
import { StudyMock } from './study'
import { UserMock } from './user'

const TURN_COMMANDS = [
  'agent_start_session',
  'agent_send_message',
  'agent_cancel_turn',
  'agent_approve_tool_call',
  'agent_answer_question',
  'agent_skip_question',
]

/** 浏览器 Mock 下模拟系统目录选择器，返回假路径，方便走通「添加目录」流程。 */
const MOCK_DIRECTORIES = [
  'D:\\学习资料\\操作系统',
  'D:\\学习资料\\数学',
  'D:\\学习资料\\英语',
]
let mockDirectoryIndex = 0

function mockDialogOpen(payload: Record<string, unknown>): string | string[] {
  const options = (payload.options ?? {}) as { multiple?: boolean }
  const path = MOCK_DIRECTORIES[mockDirectoryIndex % MOCK_DIRECTORIES.length]
  mockDirectoryIndex += 1
  return options.multiple ? [path] : path
}

/** 启用前端 Mock：按领域拦截 Tauri invoke，用内存假数据响应服务请求。 */
export function setupMocks(): void {
  const anki = new AnkiMock()
  const agents = new AgentMock()
  const assets = new AssetMock()
  const chat = new ChatMock()
  const llm = new LlmMock()
  const settings = new SettingsMock()
  const study = new StudyMock()
  const statistics = new StatisticsMock(anki)
  const user = new UserMock()

  mockIPC((cmd, payload) => {
    const args =
      payload && typeof payload === 'object' && !Array.isArray(payload)
        ? (payload as Record<string, unknown>)
        : {}

    if (cmd.startsWith('anki_')) return anki.handle(cmd, args)
    if (TURN_COMMANDS.includes(cmd)) return chat.handle(cmd, args)
    if (cmd.startsWith('session_')) return chat.handle(cmd, args)
    if (cmd.startsWith('agent_')) return agents.handle(cmd, args)
    if (cmd === 'tool_list' || cmd === 'get_all_tool_groups') return agents.handle(cmd, args)
    if (cmd.startsWith('asset_')) return assets.handle(cmd, args)
    if (cmd.startsWith('bucket_')) return assets.handle(cmd, args)
    if (cmd.startsWith('chat_')) return chat.handle(cmd, args)
    if (cmd.startsWith('llm_')) return llm.handle(cmd, args)
    if (cmd.startsWith('settings_')) return settings.handle(cmd, args)
    if (cmd.startsWith('ui_')) return settings.handle(cmd, args)
    if (cmd.startsWith('study_')) return study.handle(cmd, args)
    if (cmd.startsWith('stats_')) return statistics.handle(cmd, args)
    if (cmd.startsWith('user_')) return user.handle(cmd)
    if (cmd === 'plugin:dialog|open') return mockDialogOpen(args)

    console.warn(`[mock] 未处理的命令: ${cmd}`)
    return undefined
  })

  console.info('[mock] 已启用前端 Mock，服务接口将返回假数据。')
}
