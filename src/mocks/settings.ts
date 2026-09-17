import type { UserSettings, UserTheme } from '../types/settings'

const STORAGE_KEY = 'learning-agent-mock-settings'

const seedSettings: UserSettings = {
  goal: '考研',
  dailyHours: 4,
  reminderTime: '09:00',
  theme: 'system',
  subjects: ['数学', '英语', '计算机'],
  enableSpacedRepetition: true,
  showStatistics: true,
  autoReference: true,
}

function cloneSettings(settings: UserSettings): UserSettings {
  return { ...settings, subjects: [...settings.subjects] }
}

/** 读取上次写入的假配置，模拟 config.toml 在重启后依然存在。 */
function readStoredSettings(): UserSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)

    if (!raw) {
      return cloneSettings(seedSettings)
    }

    const stored = JSON.parse(raw) as Partial<UserSettings>
    return { ...cloneSettings(seedSettings), ...stored }
  } catch {
    return cloneSettings(seedSettings)
  }
}

/** 用户设置 Mock 后端：写入内容缓存在本地，模拟配置文件的持久化行为。 */
export class SettingsMock {
  private settings = readStoredSettings()

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'settings_get':
        return cloneSettings(this.settings)
      case 'settings_update':
        return this.updateSettings((payload.settings ?? {}) as Partial<UserSettings>)
      case 'ui_get_theme':
        return this.settings.theme
      case 'ui_set_theme':
        this.updateSettings({ theme: payload.theme as UserTheme })
        return undefined
      default:
        return undefined
    }
  }

  private updateSettings(input: Partial<UserSettings>): UserSettings {
    this.settings = {
      ...this.settings,
      ...input,
      subjects: input.subjects ? [...input.subjects] : [...this.settings.subjects],
    }
    this.persist()
    return cloneSettings(this.settings)
  }

  private persist(): void {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(this.settings))
    } catch {
      // 忽略存储不可用的情况
    }
  }
}
