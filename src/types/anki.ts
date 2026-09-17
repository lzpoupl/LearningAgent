export type CardState = 'new' | 'learning' | 'review' | 'relearning'

export type CardGrade = 'again' | 'hard' | 'good' | 'easy'

export interface Deck {
  path: string
  name: string
  cardCount: number
  subdeckCount: number
  createdAt: string
}

export interface Card {
  id: string
  deckPath: string
  front: string
  back: string
  state: CardState
  dueAt: string | null
  createdAt: string
  updatedAt: string
}

export interface CardQuery {
  state?: CardState
  dueBefore?: string
  dueAfter?: string
  keyword?: string
  limit?: number
  offset?: number
}

export interface CardSearch {
  deckPath?: string
  front?: boolean
  back?: boolean
  limit?: number
  offset?: number
}

export interface NewCard {
  deckPath: string
  front: string
  back: string
}

export interface UpdateCardContent {
  front?: string
  back?: string
}

export interface ReviewOutcome {
  cardId: string
  state: CardState
  dueAt: string | null
}

export interface ReviewOption {
  grade: CardGrade
  intervalLabel: string
}

export interface AnkiError {
  code: string
  message: string
}

/**
 * 复习调度配置，对应后端 config 包的 `SchedulerConfig`。
 *
 * 该结构未启用 camelCase 序列化，字段名与 `config.toml` 保持 snake_case，
 * 因此这里直接沿用下划线命名，避免多一层映射。
 */
export interface SchedulerConfig {
  algorithm: string
  learning_again_minutes: number
  learning_hard_minutes: number
  learning_good_minutes: number
}
