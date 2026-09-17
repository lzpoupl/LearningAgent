export interface StudyTask {
  id: string
  title: string
  detail: string
  minutes: number
  done: boolean
}

/** 首页学习计划与今日概览；卡片统计见 types/statistics.ts。 */
export interface TodayOverview {
  date: string
  tasks: StudyTask[]
  studyMinutes: number
  targetMinutes: number
}