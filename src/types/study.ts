export interface StudyTask {
  id: string
  title: string
  detail: string
  minutes: number
  done: boolean
}

export interface TodayOverview {
  date: string
  tasks: StudyTask[]
  studyMinutes: number
  targetMinutes: number
  dueCardCount: number
  totalCardCount: number
}

export interface StudyDay {
  day: string
  minutes: number
  today: boolean
}

export interface StudySubject {
  id: string
  name: string
  icon: string
  theme: string
  minutes: number
  percentage: number
}

export interface StudyStatistics {
  today: {
    minutes: number
    changePercent: number
  }
  week: {
    minutes: number
    targetMinutes: number
    progressPercent: number
  }
  streakDays: number
  masteredKnowledgePoints: number
  masteredThisWeek: number
  weeklyStudy: StudyDay[]
  subjects: StudySubject[]
  insight: {
    title: string
    description: string
    progressPercent: number
  }
}
