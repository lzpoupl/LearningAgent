import { computed, ref } from 'vue'

import { getSettings, updateSettings } from '../services/settings'
import type { UserTheme } from '../types/settings'

const STORAGE_KEY = 'learning-agent-theme'
const THEMES: UserTheme[] = ['light', 'dark', 'mountain']

function normalizeTheme(value: unknown): UserTheme {
  return THEMES.includes(value as UserTheme) ? (value as UserTheme) : 'light'
}

function readStoredTheme(): UserTheme {
  try {
    return normalizeTheme(localStorage.getItem(STORAGE_KEY))
  } catch {
    return 'light'
  }
}

const currentTheme = ref<UserTheme>(readStoredTheme())

const isDark = computed(() => currentTheme.value === 'dark')

function applyTheme(theme: UserTheme): void {
  if (typeof document === 'undefined') {
    return
  }

  const root = document.documentElement
  root.classList.toggle('dark', theme === 'dark')
  root.dataset.theme = theme

  try {
    localStorage.setItem(STORAGE_KEY, theme)
  } catch {
    // 忽略存储不可用的情况
  }
}

/** 将主题应用到文档根节点，用于应用启动时恢复上次选择。 */
export function setTheme(theme: UserTheme): void {
  const normalized = normalizeTheme(theme)
  currentTheme.value = normalized
  applyTheme(normalized)
}

export function initTheme(): void {
  applyTheme(currentTheme.value)
}

/** 将当前主题写回用户设置，保持快速切换与设置页一致。 */
export async function persistTheme(theme: UserTheme): Promise<void> {
  const settings = await getSettings()

  if (settings.theme === theme) {
    return
  }

  await updateSettings({ ...settings, theme })
}

export function useTheme() {
  function toggleDark(): void {
    setTheme(currentTheme.value === 'dark' ? 'light' : 'dark')
  }

  return {
    theme: currentTheme,
    isDark,
    setTheme,
    toggleDark,
  }
}
