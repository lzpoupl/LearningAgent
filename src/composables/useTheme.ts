import { computed, ref } from 'vue'

import { getTheme, saveTheme } from '../services/settings'
import type { UserTheme } from '../types/settings'

const STORAGE_KEY = 'learning-agent-theme'
const THEMES: UserTheme[] = ['light', 'dark', 'system']
const DARK_SCHEME_QUERY = '(prefers-color-scheme: dark)'
const DEFAULT_THEME: UserTheme = 'system'

/** 用户选择的皮肤模式；`system` 表示跟随系统的浅色/深色偏好。 */
const currentTheme = ref<UserTheme>(DEFAULT_THEME)

/** 系统当前是否处于深色模式，跟随系统时以此为准。 */
const systemPrefersDark = ref(false)

/** 实际生效的皮肤结果，跟随系统时由系统偏好决定。 */
const resolvedTheme = computed<'light' | 'dark'>(() =>
  currentTheme.value === 'system'
    ? (systemPrefersDark.value ? 'dark' : 'light')
    : currentTheme.value,
)

const isDark = computed(() => resolvedTheme.value === 'dark')

function normalizeTheme(value: unknown): UserTheme {
  return THEMES.includes(value as UserTheme) ? (value as UserTheme) : DEFAULT_THEME
}

function readSystemPrefersDark(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false
  }

  return window.matchMedia(DARK_SCHEME_QUERY).matches
}

function readStoredTheme(): UserTheme {
  try {
    return normalizeTheme(localStorage.getItem(STORAGE_KEY))
  } catch {
    return DEFAULT_THEME
  }
}

/** 把生效中的皮肤写到文档根节点，并缓存模式供下次启动即时着色。 */
function applyTheme(): void {
  if (typeof document === 'undefined') {
    return
  }

  const root = document.documentElement
  root.classList.toggle('dark', isDark.value)
  root.dataset.theme = resolvedTheme.value

  try {
    localStorage.setItem(STORAGE_KEY, currentTheme.value)
  } catch {
    // 忽略存储不可用的情况
  }
}

/** 应用新的皮肤模式并立即重新着色。 */
function applyThemeMode(next: UserTheme): void {
  currentTheme.value = next
  applyTheme()
}

/** 把皮肤模式写回配置，使其在下次启动时依然生效。 */
async function persistTheme(next: UserTheme): Promise<void> {
  try {
    await saveTheme(next)
  } catch (error) {
    console.warn('[theme] 皮肤模式写回配置失败，本地缓存仍然生效。', error)
  }
}

/** 设定皮肤模式并立即生效；`system` 跟随系统的浅色/深色偏好。 */
export function setTheme(next: UserTheme): void {
  const normalized = normalizeTheme(next)
  applyThemeMode(normalized)
  void persistTheme(normalized)
}

/** 系统深浅色偏好变化时立即重新着色。 */
function handleSystemSchemeChange(event: MediaQueryListEvent): void {
  systemPrefersDark.value = event.matches

  if (currentTheme.value === 'system') {
    applyTheme()
  }
}

let systemSchemeQuery: MediaQueryList | null = null

function watchSystemScheme(): void {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return
  }

  systemPrefersDark.value = readSystemPrefersDark()

  if (systemSchemeQuery) {
    return
  }

  systemSchemeQuery = window.matchMedia(DARK_SCHEME_QUERY)
  systemSchemeQuery.addEventListener('change', handleSystemSchemeChange)
}

/** 采用配置中的皮肤模式；配置不可用时保留本地缓存的模式。 */
async function adoptConfiguredTheme(): Promise<void> {
  try {
    const configured = normalizeTheme(await getTheme())

    if (configured !== currentTheme.value) {
      applyThemeMode(configured)
    }
  } catch (error) {
    console.warn('[theme] 读取配置中的皮肤模式失败，继续使用本地缓存。', error)
  }
}

/** 启动时恢复皮肤：先按本地缓存着色避免闪烁，再以配置为准。 */
export async function initTheme(): Promise<void> {
  watchSystemScheme()
  applyThemeMode(readStoredTheme())
  await adoptConfiguredTheme()
}

export function useTheme() {
  function toggleDark(): void {
    setTheme(isDark.value ? 'light' : 'dark')
  }

  return {
    theme: currentTheme,
    resolvedTheme,
    isDark,
    setTheme,
    toggleDark,
  }
}
