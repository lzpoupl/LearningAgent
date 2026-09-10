import { invoke } from '@tauri-apps/api/core'

import type { UserSettings } from '../types/settings'

export function getSettings(): Promise<UserSettings> {
  return invoke<UserSettings>('settings_get')
}

export function updateSettings(settings: UserSettings): Promise<UserSettings> {
  return invoke<UserSettings>('settings_update', { settings })
}
