import { invoke } from '@tauri-apps/api/core'

import type {
  AddedCardsStats,
  CardBreakdown,
  ReviewHistoryStats,
  TimeRange,
  TodayProgress,
} from '../types/statistics'

export function getTodayProgress(): Promise<TodayProgress> {
  return invoke<TodayProgress>('stats_get_today_progress')
}

export function getCardBreakdown(): Promise<CardBreakdown> {
  return invoke<CardBreakdown>('stats_get_card_breakdown')
}

export function getReviewHistory(range: TimeRange): Promise<ReviewHistoryStats> {
  return invoke<ReviewHistoryStats>('stats_get_review_history', { range })
}

export function getAddedCards(range: TimeRange): Promise<AddedCardsStats> {
  return invoke<AddedCardsStats>('stats_get_added_cards', { range })
}