import { invoke } from '@tauri-apps/api/core'

import type { UploadImageRequest, UploadedImage } from '../types/assets'
import type {
  Card,
  CardGrade,
  CardQuery,
  CardSearch,
  Deck,
  NewCard,
  ReviewOption,
  ReviewOutcome,
  UpdateCardContent,
} from '../types/anki'

export function getSubdecks(deckPath: string): Promise<Deck[]> {
  return invoke<Deck[]>('anki_get_subdecks', { deckPath })
}

export function getCards(deckPath: string, query: CardQuery = {}): Promise<Card[]> {
  return invoke<Card[]>('anki_get_cards', { deckPath, query })
}

export function getCard(cardId: string): Promise<Card> {
  return invoke<Card>('anki_get_card', { cardId })
}

export function searchCards(keyword: string, search: CardSearch = {}): Promise<Card[]> {
  const normalizedSearch: CardSearch = {
    front: true,
    back: true,
    ...search,
  }

  return invoke<Card[]>('anki_search_cards', { keyword, search: normalizedSearch })
}

export function createDeck(deckPath: string): Promise<string> {
  return invoke<string>('anki_create_deck', { deckPath })
}

export function createCard(newCard: NewCard): Promise<string> {
  return invoke<string>('anki_create_card', { newCard })
}

/**
 * 移动牌组到目标父牌组下。
 *
 * 注意：`targetParentPath` 是「目标父牌组」的路径，而不是移动后的完整路径。
 * 后端会保留源牌组自身的名字，因此 `a -> /path/for` 的结果是 `/path/for/a`。
 * 若要移到根目录，请传 `/`。
 */
export function moveDeck(sourcePath: string, targetParentPath: string): Promise<void> {
  return invoke<void>('anki_move_deck', { sourcePath, targetPath: targetParentPath })
}

/**
 * 移动卡片到目标牌组。
 *
 * 注意：`targetDeckPath` 是卡片要放入的具体牌组本身（不能是根 `/`）。
 */
export function moveCard(cardId: string, targetDeckPath: string): Promise<void> {
  return invoke<void>('anki_move_card', { cardId, targetDeckPath })
}

export function gradeCard(cardId: string, grade: CardGrade): Promise<ReviewOutcome> {
  return invoke<ReviewOutcome>('anki_grade_card', { cardId, grade })
}

export function getReviewOptions(cardId: string): Promise<ReviewOption[]> {
  return invoke<ReviewOption[]>('anki_get_review_options', { cardId })
}

export function resetCard(cardId: string): Promise<ReviewOutcome> {
  return invoke<ReviewOutcome>('anki_reset_card', { cardId })
}

export function updateCardContent(cardId: string, content: UpdateCardContent): Promise<void> {
  return invoke<void>('anki_update_card_content', { cardId, content })
}

export function deleteDeck(deckPath: string): Promise<void> {
  return invoke<void>('anki_delete_deck', { deckPath })
}

export function deleteCard(cardId: string): Promise<void> {
  return invoke<void>('anki_delete_card', { cardId })
}

export function uploadImage(input: UploadImageRequest): Promise<UploadedImage> {
  return invoke<UploadedImage>('anki_upload_image', { input })
}
