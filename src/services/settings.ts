import { invoke } from '@tauri-apps/api/core'

import type { UserSettings, UserTheme } from '../types/settings'

export function getSettings(): Promise<UserSettings> {
  return invoke<UserSettings>('settings_get')
}

export function updateSettings(settings: UserSettings): Promise<UserSettings> {
  return invoke<UserSettings>('settings_update', { settings })
}

/** 读取 config.toml 中保存的皮肤模式。 */
export function getTheme(): Promise<UserTheme> {
  return invoke<UserTheme>('ui_get_theme')
}

/** 把皮肤模式写入 config.toml，使其在下次启动时恢复。 */
export function saveTheme(theme: UserTheme): Promise<void> {
  return invoke<void>('ui_set_theme', { theme })
}
