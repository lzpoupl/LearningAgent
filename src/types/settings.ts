export type UserTheme = 'light' | 'dark' | 'mountain'

export interface UserSettings {
  goal: string
  dailyHours: number
  reminderTime: string
  theme: UserTheme
  subjects: string[]
  enableSpacedRepetition: boolean
  showStatistics: boolean
  autoReference: boolean
}
