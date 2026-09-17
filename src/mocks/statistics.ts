import type {
  AddedCardsStats,
  CardBreakdown,
  CardCategory,
  CardCategoryCount,
  DailyCount,
  ReviewHistoryStats,
  TimeRange,
  TodayProgress,
} from '../types/statistics'

import type { AnkiMock } from './anki'
import type { Card } from '../types/anki'

/** 历史数据覆盖的天数。 */
const HISTORY_DAYS = 365

const CATEGORY_LABELS: Record<CardCategory, string> = {
  new: '新卡',
  learning: '学习中',
  review: '复习中',
  relearning: '重新学习',
}

function localDay(date: Date): string {
  const month = `${date.getMonth() + 1}`.padStart(2, '0')
  const day = `${date.getDate()}`.padStart(2, '0')
  return `${date.getFullYear()}-${month}-${day}`
}

function dayOffset(days: number): string {
  const date = new Date()
  date.setDate(date.getDate() + days)
  return localDay(date)
}

/** 固定种子的线性同余发生器：保证每次刷新看到相同的历史曲线。 */
function createRandom(seed: number) {
  let state = seed
  return () => {
    state = (state * 1664525 + 1013904223) % 4294967296
    return state / 4294967296
  }
}

/** 生成近一年的每日计数（最旧在前）。 */
function seedDailyCounts(seed: number, maxPerDay: number, idleChance: number): number[] {
  const random = createRandom(seed)
  const counts: number[] = []
  for (let index = 0; index < HISTORY_DAYS; index += 1) {
    counts.push(random() < idleChance ? 0 : Math.floor(random() * maxPerDay) + 1)
  }
  return counts
}

/** 内存版统计后端：卡片分布取自 AnkiMock，历史曲线为固定种子的假数据。 */
export class StatisticsMock {
  private reviewCounts = seedDailyCounts(20260101, 12, 0.55)
  private addedCounts = seedDailyCounts(20260917, 5, 0.82)
  private reviewedToday = new Set<string>()

  constructor(private anki: AnkiMock) {
    anki.setReviewListener((cardId) => {
      this.reviewedToday.add(cardId)
      this.reviewCounts[HISTORY_DAYS - 1] += 1
    })
  }

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'stats_get_today_progress':
        return this.todayProgress()
      case 'stats_get_card_breakdown':
        return this.breakdown()
      case 'stats_get_review_history':
        return this.reviewHistory(String(payload.range ?? 'last_year') as TimeRange)
      case 'stats_get_added_cards':
        return this.addedCards(String(payload.range ?? 'last_year') as TimeRange)
      default:
        return undefined
    }
  }

  private cards(): Card[] {
    return this.anki.snapshotCards()
  }

  private todayProgress(): TodayProgress {
    const now = Date.now()
    const cards = this.cards()
    const newRemaining = cards.filter((card) => card.state === 'new').length
    const dueRemaining = cards.filter(
      (card) =>
        card.state !== 'new' &&
        card.dueAt !== null &&
        new Date(card.dueAt).getTime() <= now,
    ).length

    const pendingCards = newRemaining + dueRemaining
    const reviewedCards = this.reviewedToday.size
    const answered = reviewedCards + pendingCards

    return {
      date: localDay(new Date()),
      reviewedCards,
      pendingCards,
      totalCards: this.cards().length,
      newRemaining,
      dueRemaining,
      completionPercent: answered === 0
        ? 0
        : Math.round((reviewedCards / answered) * 1000) / 10,
    }
  }

  private breakdown(): CardBreakdown {
    const buckets = new Map<CardCategory, number>()
    const add = (category: CardCategory) =>
      buckets.set(category, (buckets.get(category) ?? 0) + 1)

    for (const card of this.cards()) {
      add(card.state)
    }

    const categories = (Object.keys(CATEGORY_LABELS) as CardCategory[])
      .map<CardCategoryCount>((category) => ({
        category,
        label: CATEGORY_LABELS[category],
        count: buckets.get(category) ?? 0,
        percent: 0,
      }))

    const total = categories.reduce((sum, item) => sum + item.count, 0)
    for (const item of categories) {
      item.percent = total === 0 ? 0 : Math.round((item.count / total) * 1000) / 10
    }

    return { total, categories }
  }

  private reviewHistory(range: TimeRange): ReviewHistoryStats {
    const days = this.slice(range, this.reviewCounts)
    const totalReviews = days.reduce((sum, day) => sum + day.count, 0)
    const studiedDays = days.filter((day) => day.count > 0).length
    const elapsedDays = days.length

    return {
      range,
      days,
      totalReviews,
      studiedDays,
      elapsedDays,
      studiedDayPercent: elapsedDays === 0
        ? 0
        : Math.round((studiedDays / elapsedDays) * 1000) / 10,
      averagePerElapsedDay: elapsedDays === 0
        ? 0
        : Math.round((totalReviews / elapsedDays) * 100) / 100,
      averagePerStudiedDay: studiedDays === 0
        ? 0
        : Math.round((totalReviews / studiedDays) * 100) / 100,
    }
  }

  private addedCards(range: TimeRange): AddedCardsStats {
    const days = this.slice(range, this.withRealCards(this.addedCounts))
    const total = days.reduce((sum, day) => sum + day.count, 0)

    return {
      range,
      days,
      total,
      elapsedDays: days.length,
      averagePerDay: days.length === 0 ? 0 : Math.round((total / days.length) * 100) / 100,
    }
  }

  /** 把真实存在的卡片按其创建日计入新增曲线，今天新建的卡片会立即反映出来。 */
  private withRealCards(counts: number[]): number[] {
    const merged = [...counts]
    for (const card of this.cards()) {
      const day = localDay(new Date(card.createdAt))
      for (let index = 0; index < HISTORY_DAYS; index += 1) {
        if (dayOffset(index - HISTORY_DAYS + 1) === day) {
          merged[index] += 1
          break
        }
      }
    }
    return merged
  }

  private slice(range: TimeRange, counts: number[]): DailyCount[] {
    const size = range === 'last_month'
      ? 30
      : range === 'last_three_months'
        ? 90
        : HISTORY_DAYS
    const days: DailyCount[] = []

    for (let index = HISTORY_DAYS - size; index < HISTORY_DAYS; index += 1) {
      days.push({
        day: dayOffset(index - HISTORY_DAYS + 1),
        daysAgo: index - HISTORY_DAYS + 1,
        count: counts[index] ?? 0,
      })
    }
    return days
  }
}