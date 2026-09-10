import { invoke } from '@tauri-apps/api/core'

import type { UserProfile } from '../types/user'

export function getCurrentUser(): Promise<UserProfile> {
  return invoke<UserProfile>('user_get_profile')
}
