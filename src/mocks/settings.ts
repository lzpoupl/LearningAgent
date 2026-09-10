import type { UserSettings } from '../types/settings'

const seedSettings: UserSettings = {
  goal: '考研',
  dailyHours: 4,
  reminderTime: '09:00',
  theme: 'light',
  subjects: ['数学', '英语', '计算机'],
  enableSpacedRepetition: true,
  showStatistics: true,
  autoReference: true,
}

function cloneSettings(settings: UserSettings): UserSettings {
  return { ...settings, subjects: [...settings.subjects] }
}

/** 内存版用户设置后端。 */
export class SettingsMock {
  private settings = cloneSettings(seedSettings)

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'settings_get':
        return cloneSettings(this.settings)
      case 'settings_update':
        return this.updateSettings((payload.settings ?? {}) as Partial<UserSettings>)
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
    return cloneSettings(this.settings)
  }
}
