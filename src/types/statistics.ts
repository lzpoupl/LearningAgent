/** 条形图与饼图使用的时间范围。 */
export type TimeRange = 'last_month' | 'last_three_months' | 'last_year' | 'all'

/** 卡片分类，与「卡片数量」图例一一对应。 */
export type CardCategory = 'new' | 'learning' | 'review' | 'relearning'

/** 今日学习统计。 */
export interface TodayProgress {
  date: string
  reviewedCards: number
  pendingCards: number
  totalCards: number
  newRemaining: number
  dueRemaining: number
  completionPercent: number
}

export interface CardCategoryCount {
  category: CardCategory
  label: string
  count: number
  percent: number
}

export interface CardBreakdown {
  total: number
  categories: CardCategoryCount[]
}

/** 条形图中的一天。 */
export interface DailyCount {
  day: string
  /** 距今天数：0 为今天，-N 为 N 天前。 */
  daysAgo: number
  count: number
}

export interface ReviewHistoryStats {
  range: TimeRange
  days: DailyCount[]
  totalReviews: number
  studiedDays: number
  elapsedDays: number
  studiedDayPercent: number
  averagePerElapsedDay: number
  averagePerStudiedDay: number
}

export interface AddedCardsStats {
  range: TimeRange
  days: DailyCount[]
  total: number
  elapsedDays: number
  averagePerDay: number
}