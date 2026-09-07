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

export interface AnkiError {
  code: string
  message: string
}
