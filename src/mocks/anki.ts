import type {
  UploadImageRequest,
  UploadedImage,
} from '../types/assets'
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
import { seedCards, seedDecks, type MockDeck } from './data'

function normalizePath(path: string): string {
  const segments = path.split('/').filter(Boolean)
  return `/${segments.join('/')}`
}

function parentPath(path: string): string {
  const segments = path.split('/').filter(Boolean)
  segments.pop()
  return `/${segments.join('/')}`
}

function joinPath(parent: string, name: string): string {
  return normalizePath(`${parent}/${name}`)
}

function nowIso(): string {
  return new Date().toISOString()
}

/** 内存版 Anki 后端：拦截 invoke 后返回假数据，并支持增删改查。 */
export class AnkiMock {
  private decks = new Map<string, MockDeck>()
  private cards = new Map<string, Card>()
  private nextCardId = 100

  constructor() {
    for (const deck of seedDecks) {
      this.decks.set(deck.path, { ...deck })
    }
    for (const card of seedCards) {
      this.cards.set(card.id, { ...card })
    }
  }

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'anki_get_subdecks':
        return this.getSubdecks(String(payload.deckPath ?? ''))
      case 'anki_get_cards':
        return this.getCards(String(payload.deckPath ?? ''), (payload.query ?? {}) as CardQuery)
      case 'anki_get_card':
        return this.getCard(String(payload.cardId ?? ''))
      case 'anki_search_cards':
        return this.searchCards(String(payload.keyword ?? ''), (payload.search ?? {}) as CardSearch)
      case 'anki_create_deck':
        return this.createDeck(String(payload.deckPath ?? ''))
      case 'anki_create_card':
        return this.createCard(payload.newCard as NewCard)
      case 'anki_move_deck':
        return this.moveDeck(String(payload.sourcePath ?? ''), String(payload.targetPath ?? ''))
      case 'anki_move_card':
        return this.moveCard(String(payload.cardId ?? ''), String(payload.targetDeckPath ?? ''))
      case 'anki_grade_card':
        return this.gradeCard(String(payload.cardId ?? ''), payload.grade as CardGrade)
      case 'anki_get_review_options':
        return this.getReviewOptions(String(payload.cardId ?? ''))
      case 'anki_reset_card':
        return this.resetCard(String(payload.cardId ?? ''))
      case 'anki_update_card_content':
        return this.updateCardContent(
          String(payload.cardId ?? ''),
          (payload.content ?? {}) as UpdateCardContent,
        )
      case 'anki_delete_deck':
        return this.deleteDeck(String(payload.deckPath ?? ''))
      case 'anki_delete_card':
        return this.deleteCard(String(payload.cardId ?? ''))
      case 'anki_upload_image':
        return this.uploadImage((payload.input ?? {}) as Partial<UploadImageRequest>)
      default:
        return undefined
    }
  }

  // ---- 查询 ----

  private getSubdecks(deckPath: string): Deck[] {
    const path = normalizePath(deckPath)
    const result: Deck[] = []
    for (const deck of this.decks.values()) {
      if (parentPath(deck.path) === path) {
        result.push(this.toDeck(deck))
      }
    }
    return result.sort((a, b) => a.name.localeCompare(b.name))
  }

  private getCards(deckPath: string, query: CardQuery): Card[] {
    const path = normalizePath(deckPath)
    let list = [...this.cards.values()].filter((card) => card.deckPath === path)

    if (query.state) {
      const state = query.state
      list = list.filter((card) => card.state === state)
    }
    if (query.dueBefore) {
      const dueBefore = query.dueBefore
      list = list.filter((card) => card.dueAt !== null && card.dueAt <= dueBefore)
    }
    if (query.dueAfter) {
      const dueAfter = query.dueAfter
      list = list.filter((card) => card.dueAt !== null && card.dueAt >= dueAfter)
    }
    if (query.keyword) {
      const keyword = query.keyword
      list = list.filter((card) => card.front.includes(keyword) || card.back.includes(keyword))
    }

    return this.paginate(list, query.offset, query.limit).map((card) => ({ ...card }))
  }

  private getCard(cardId: string): Card {
    const card = this.cards.get(cardId)
    if (!card) throw new Error(`卡片不存在: ${cardId}`)
    return { ...card }
  }

  private searchCards(keyword: string, search: CardSearch): Card[] {
    let list = [...this.cards.values()]
    if (search.deckPath) {
      const path = normalizePath(search.deckPath)
      list = list.filter((card) => card.deckPath === path)
    }
    const searchFront = search.front !== false
    const searchBack = search.back !== false
    if (searchFront || searchBack) {
      list = list.filter(
        (card) =>
          (searchFront && card.front.includes(keyword)) ||
          (searchBack && card.back.includes(keyword)),
      )
    }
    return this.paginate(list, search.offset, search.limit).map((card) => ({ ...card }))
  }

  // ---- 写操作 ----

  private createDeck(deckPath: string): string {
    const path = normalizePath(deckPath)
    if (path === '/') throw new Error('牌组路径不能为空')

    const segments = path.split('/').filter(Boolean)
    let current = ''
    for (const segment of segments) {
      current = joinPath(current, segment)
      if (!this.decks.has(current)) {
        this.decks.set(current, { path: current, name: segment, createdAt: nowIso() })
      }
    }
    return path
  }

  private createCard(newCard: NewCard): string {
    const deckPath = normalizePath(newCard.deckPath)
    if (deckPath === '/' || !this.decks.has(deckPath)) {
      throw new Error(`牌组不存在: ${newCard.deckPath}`)
    }
    const id = String(this.nextCardId++)
    const now = nowIso()
    this.cards.set(id, {
      id,
      deckPath,
      front: newCard.front,
      back: newCard.back,
      state: 'new',
      dueAt: null,
      createdAt: now,
      updatedAt: now,
    })
    return id
  }

  private moveDeck(sourcePath: string, targetPath: string): void {
    const source = normalizePath(sourcePath)
    const target = normalizePath(targetPath)
    const sourceDeck = this.decks.get(source)
    if (!sourceDeck) throw new Error(`牌组不存在: ${sourcePath}`)
    if (source === target) throw new Error('不能把牌组移动到自身')
    if (target !== '/' && !this.decks.has(target)) throw new Error(`目标牌组不存在: ${targetPath}`)
    if (target === source || target.startsWith(`${source}/`)) {
      throw new Error('不能把牌组移动到它的子牌组下')
    }

    const newPath = joinPath(target, sourceDeck.name)
    if (newPath !== source && this.decks.has(newPath)) {
      throw new Error('目标牌组下已存在同名子牌组')
    }

    // Mock 内部按路径存储，移动时重写整棵子树的路径。
    const mapping = new Map<string, string>()
    for (const deck of this.decks.values()) {
      if (deck.path === source || deck.path.startsWith(`${source}/`)) {
        mapping.set(deck.path, newPath + deck.path.slice(source.length))
      }
    }
    for (const [oldPath, renamedPath] of mapping) {
      const deck = this.decks.get(oldPath)
      if (!deck) continue
      this.decks.delete(oldPath)
      deck.path = renamedPath
      this.decks.set(renamedPath, deck)
    }
    for (const card of this.cards.values()) {
      if (card.deckPath === source || card.deckPath.startsWith(`${source}/`)) {
        card.deckPath = newPath + card.deckPath.slice(source.length)
      }
    }
  }

  private moveCard(cardId: string, targetDeckPath: string): void {
    const card = this.cards.get(cardId)
    if (!card) throw new Error(`卡片不存在: ${cardId}`)
    const target = normalizePath(targetDeckPath)
    if (target === '/' || !this.decks.has(target)) {
      throw new Error(`牌组不存在: ${targetDeckPath}`)
    }
    card.deckPath = target
    card.updatedAt = nowIso()
  }

  private updateCardContent(cardId: string, content: UpdateCardContent): void {
    const card = this.cards.get(cardId)
    if (!card) throw new Error(`卡片不存在: ${cardId}`)
    if (content.front !== undefined) card.front = content.front
    if (content.back !== undefined) card.back = content.back
    card.updatedAt = nowIso()
  }

  private deleteDeck(deckPath: string): void {
    const path = normalizePath(deckPath)
    if (path === '/') throw new Error('根牌组不可删除')
    if (!this.decks.has(path)) throw new Error(`牌组不存在: ${deckPath}`)

    for (const key of [...this.decks.keys()]) {
      if (key === path || key.startsWith(`${path}/`)) this.decks.delete(key)
    }
    for (const [id, card] of [...this.cards.entries()]) {
      if (card.deckPath === path || card.deckPath.startsWith(`${path}/`)) this.cards.delete(id)
    }
  }

  private deleteCard(cardId: string): void {
    if (!this.cards.delete(cardId)) throw new Error(`卡片不存在: ${cardId}`)
  }

  private gradeCard(cardId: string, grade: CardGrade): ReviewOutcome {
    const card = this.cards.get(cardId)
    if (!card) throw new Error(`卡片不存在: ${cardId}`)
    const days: Record<CardGrade, number> = { again: 1, hard: 3, good: 7, easy: 14 }
    const state: Card['state'] = grade === 'again' ? 'relearning' : 'review'
    const dueAt = new Date(Date.now() + days[grade] * 86_400_000).toISOString()
    card.state = state
    card.dueAt = dueAt
    card.updatedAt = nowIso()
    return { cardId, state, dueAt }
  }

  private getReviewOptions(cardId: string): ReviewOption[] {
    this.getCard(cardId)
    const intervals: Record<CardGrade, { days: number; label: string }> = {
      again: { days: 0, label: '< 10 分' },
      hard: { days: 3, label: '3 天' },
      good: { days: 7, label: '7 天' },
      easy: { days: 14, label: '14 天' },
    }

    return (Object.keys(intervals) as CardGrade[]).map(grade => ({
      grade,
      intervalLabel: intervals[grade].label,
      dueAt: intervals[grade].days === 0
        ? new Date(Date.now() + 10 * 60_000).toISOString()
        : new Date(Date.now() + intervals[grade].days * 86_400_000).toISOString(),
    }))
  }

  private resetCard(cardId: string): ReviewOutcome {
    const card = this.cards.get(cardId)
    if (!card) throw new Error(`卡片不存在: ${cardId}`)
    card.state = 'new'
    card.dueAt = null
    card.updatedAt = nowIso()
    return { cardId, state: 'new', dueAt: null }
  }

  private uploadImage(input: Partial<UploadImageRequest>): UploadedImage {
    const name = String(input.name ?? '').trim()
    const mimeType = String(input.mimeType ?? '')
    const contentBase64 = String(input.contentBase64 ?? '')
    if (!name || !mimeType.startsWith('image/') || !contentBase64) {
      throw new Error('图片上传参数不完整')
    }

    return {
      name,
      url: `data:${mimeType};base64,${contentBase64}`,
    }
  }

  // ---- 辅助 ----

  private toDeck(deck: MockDeck): Deck {
    return {
      path: deck.path,
      name: deck.name,
      cardCount: this.countCards(deck.path),
      subdeckCount: this.countSubdecks(deck.path),
      createdAt: deck.createdAt,
    }
  }

  private countCards(deckPath: string): number {
    let count = 0
    for (const card of this.cards.values()) {
      if (card.deckPath === deckPath) count += 1
    }
    return count
  }

  private countSubdecks(deckPath: string): number {
    let count = 0
    for (const deck of this.decks.values()) {
      if (parentPath(deck.path) === deckPath) count += 1
    }
    return count
  }

  private paginate<T>(list: T[], offset?: number, limit?: number): T[] {
    const start = offset ?? 0
    return limit === undefined ? list.slice(start) : list.slice(start, start + limit)
  }
}
