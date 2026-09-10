import type { UserProfile } from '../types/user'

const profile: UserProfile = {
  displayName: 'Wannamai',
  role: '考研学习者',
  initials: 'W',
}

/** 内存版用户资料后端。 */
export class UserMock {
  handle(cmd: string): unknown {
    if (cmd !== 'user_get_profile') {
      return undefined
    }
    return { ...profile }
  }
}
