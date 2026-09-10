<template>
  <main class="review-page">
    <header class="review-header">
      <div class="review-brand">
        <span class="eyebrow">ANKI / REVIEW</span>
        <h1>Anki 复习</h1>
      </div>
      <nav class="review-tools" aria-label="复习工具">
        <div class="deck-picker">
          <button class="deck-picker-button" type="button" :disabled="loading" @click="deckMenuOpen = !deckMenuOpen">
            <span>{{ selectedDeck?.path || '选择牌组' }}</span>
            <span aria-hidden="true">⌄</span>
          </button>
          <div v-if="deckMenuOpen" class="deck-menu" role="tree">
            <button
              v-for="row in deckRows"
              :key="row.deck.path"
              class="deck-option"
              type="button"
              role="treeitem"
              :style="{ paddingLeft: `${10 + row.depth * 17}px` }"
              @click="selectDeck(row.deck.path)"
            >
              <span class="deck-option-icon">{{ row.hasChildren ? '▱' : '·' }}</span>
              <span>{{ row.deck.name || row.deck.path }}</span>
              <small>{{ row.cardCount }}</small>
            </button>
          </div>
        </div>
        <button class="tool-button" type="button" @click="$emit('browse')">浏览</button>
      </nav>
    </header>

    <section class="review-stage">
      <div v-if="loading" class="review-message">正在加载复习卡片...</div>
      <div v-else-if="errorMessage" class="review-message error-message">{{ errorMessage }}</div>
      <template v-else-if="currentCard">
        <div class="card-position">随机复习 · {{ currentIndex + 1 }} / {{ reviewCards.length }}</div>
        <article class="review-card" :class="{ revealed }" @click="revealed = !revealed">
          <div class="card-half card-front">
            <div class="card-label">问题</div>
             <CardContent class="card-content" :content="currentCard.front" />
          </div>
          <div class="card-divider"><span>{{ revealed ? '点击卡片隐藏答案' : '点击卡片查看答案' }}</span></div>
          <div class="card-half card-back" :class="{ hidden: !revealed }">
            <div class="card-label">答案</div>
             <CardContent class="card-content" :content="currentCard.back" />
          </div>
        </article>
      </template>
      <div v-else class="review-message">
        <strong>{{ selectedDeckPath ? '今天没有待复习卡片' : '选择一个牌组开始复习' }}</strong>
        <span>{{ selectedDeckPath ? '可以浏览其他卡片，或稍后继续。' : '先从上方选择牌组。' }}</span>
      </div>
    </section>

    <footer class="review-footer">
      <button class="edit-button" type="button" :disabled="!currentCard" @click="editCurrentCard">编辑</button>
      <div v-if="currentCard" class="grade-actions" aria-label="复习评分">
        <button class="grade-button again" type="button" :disabled="grading" @click="gradeCurrentCard('again')">
          <span class="grade-time">&lt; 10 分</span>
          <strong>重来</strong>
        </button>
        <button class="grade-button hard" type="button" :disabled="grading" @click="gradeCurrentCard('hard')">
          <span class="grade-time">6.3 个⽉</span>
          <strong>困难</strong>
        </button>
        <button class="grade-button good" type="button" :disabled="grading" @click="gradeCurrentCard('good')">
          <span class="grade-time">1.7 年</span>
          <strong>良好</strong>
        </button>
        <button class="grade-button easy" type="button" :disabled="grading" @click="gradeCurrentCard('easy')">
          <span class="grade-time">2.7 年</span>
          <strong>简单</strong>
        </button>
      </div>
      <button class="more-button" type="button" title="更多操作">更多 ▾</button>
    </footer>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import { getCards, gradeCard } from '../services/anki'
import type { Card, CardGrade, Deck } from '../types/anki'
import CardContent from '../components/anki/CardContent.vue'
import {
  flattenDecks,
  flattenDeckRows,
  getDescendantPaths,
  loadDeckTree,
  type DeckNode,
} from '../composables/useDeckTree'

const emit = defineEmits<{
  browse: []
  'edit-card': [card: Card]
}>()

const decks = ref<Deck[]>([])
const reviewCards = ref<Card[]>([])
const selectedDeckPath = ref('')
const deckMenuOpen = ref(false)
const currentIndex = ref(0)
const revealed = ref(false)
const loading = ref(false)
const grading = ref(false)
const errorMessage = ref('')

const currentCard = computed(() => reviewCards.value[currentIndex.value] || null)
const selectedDeck = computed(() => decks.value.find(deck => deck.path === selectedDeckPath.value) || null)
const deckTree = ref<DeckNode[]>([])
const deckRows = computed(() => flattenDeckRows(deckTree.value))

async function loadDecks() {
  loading.value = true
  errorMessage.value = ''

  try {
    const tree = await loadDeckTree()
    deckTree.value = tree
    decks.value = flattenDecks(tree)
    if (!selectedDeckPath.value && decks.value.length) {
      selectedDeckPath.value = decks.value[0].path
    }
    await loadReviewCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组加载失败，请确认 Anki 服务已连接。'
  } finally {
    loading.value = false
  }
}

async function loadReviewCards(excludedCardId?: string) {
  if (!selectedDeckPath.value) {
    reviewCards.value = []
    return
  }

  loading.value = true
  errorMessage.value = ''
  try {
    const deckPaths = getDescendantPaths(selectedDeckPath.value, deckTree.value)
    const loadedCards = await Promise.all(deckPaths.map(deckPath => getCards(deckPath, {})))
    const cardsById = new Map<string, Card>()
    loadedCards.flat().forEach(card => cardsById.set(card.id, card))
    const cards = Array.from(cardsById.values())
    const now = Date.now()
    reviewCards.value = cards.filter(card => {
      if (card.id === excludedCardId) {
        return false
      }

      if (card.state === 'new') {
        return false
      }

      return !card.dueAt || new Date(card.dueAt).getTime() <= now
    })
    currentIndex.value = reviewCards.value.length
      ? Math.floor(Math.random() * reviewCards.value.length)
      : 0
    revealed.value = false
    deckMenuOpen.value = false
  } catch (error) {
    console.error(error)
    errorMessage.value = '复习卡片加载失败，请重试。'
  } finally {
    loading.value = false
  }
}

async function selectDeck(deckPath: string) {
  selectedDeckPath.value = deckPath
  await loadReviewCards()
}

async function gradeCurrentCard(grade: CardGrade) {
  if (!currentCard.value || grading.value) {
    return
  }

  grading.value = true
  try {
    const gradedCardId = currentCard.value.id
    const result = await gradeCard(gradedCardId, grade)
    currentCard.value.state = result.state
    currentCard.value.dueAt = result.dueAt
    reviewCards.value.splice(currentIndex.value, 1)

    if (reviewCards.value.length > 0) {
      currentIndex.value = Math.floor(Math.random() * reviewCards.value.length)
    } else {
      await loadReviewCards(gradedCardId)
    }
    revealed.value = false
  } catch (error) {
    console.error(error)
    errorMessage.value = '复习结果保存失败，请重试。'
  } finally {
    grading.value = false
  }
}

function editCurrentCard() {
  if (currentCard.value) {
    emit('edit-card', currentCard.value)
  }
}

onMounted(loadDecks)
</script>

<style scoped>
.review-page {
  flex: 1;
  min-width: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  background: var(--learning-bg);
  color: var(--learning-text);
}

.review-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  padding: 12px clamp(24px, 6vw, 76px);
  border-bottom: 1px solid var(--learning-border);
  background: rgba(255, 255, 255, 0.94);
}

.eyebrow {
  color: var(--learning-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

h1 {
  margin: 5px 0 0;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 22px;
  font-weight: 500;
}

.review-tools {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tool-button,
.edit-button,
.more-button {
  height: 36px;
  padding: 0 14px;
  border: 1px solid var(--learning-border);
  border-radius: 6px;
  background: var(--learning-surface);
  color: var(--learning-text-secondary);
  cursor: pointer;
  font-size: 12px;
}

.deck-picker {
  position: relative;
  width: 220px;
}

.deck-picker-button {
  width: 100%;
  height: 36px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 0 12px;
  overflow: hidden;
  border: 1px solid var(--learning-border);
  border-radius: 6px;
  background: var(--learning-surface);
  color: var(--learning-text-secondary);
  cursor: pointer;
  font-size: 12px;
  text-align: left;
}

.deck-picker-button span:first-child {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.deck-menu {
  position: absolute;
  z-index: 10;
  top: calc(100% + 6px);
  right: 0;
  left: 0;
  max-height: 300px;
  overflow-y: auto;
  padding: 5px;
  border: 1px solid #e1dbd2;
  border-radius: 7px;
  background: #fff;
  box-shadow: 0 14px 28px rgba(61, 49, 36, 0.14);
}

.deck-option {
  width: 100%;
  min-height: 34px;
  display: flex;
  align-items: center;
  gap: 7px;
  border: 0;
  border-radius: 5px;
  background: transparent;
  color: #5c5146;
  cursor: pointer;
  font-size: 12px;
  text-align: left;
}

.deck-option:hover {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.deck-option-icon {
  width: 14px;
  color: #a08c75;
  text-align: center;
}

.deck-option small {
  margin-left: auto;
  padding-right: 8px;
  color: #a69b90;
  font-size: 10px;
}

.tool-button:hover,
.edit-button:hover,
.more-button:hover {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.tool-button:focus-visible,
.edit-button:focus-visible,
.more-button:focus-visible,
.deck-picker-button:focus-visible,
.deck-option:focus-visible,
.grade-button:focus-visible,
.review-card:focus-visible {
  outline: 3px solid rgba(40, 125, 245, 0.25);
  outline-offset: 2px;
}

.review-stage {
  position: relative;
  flex: 1;
  display: flex;
  align-items: flex-start;
  justify-content: center;
  overflow: auto;
  padding: 42px 24px 68px;
}

.review-card {
  width: min(620px, 76vw);
  height: min(560px, calc(100vh - 190px));
  min-height: 420px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  align-items: stretch;
  justify-content: flex-start;
  overflow: hidden;
  border: 1px solid #49443b;
  border-radius: 10px;
  background: #272521;
  box-shadow: 0 16px 42px rgba(49, 41, 32, 0.16);
  text-align: center;
}

.card-position {
  position: absolute;
  top: 18px;
  color: #a1978d;
  font-size: 12px;
}

.card-label {
  color: #c7a77e;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.card-content {
  width: 100%;
  margin-top: 18px;
  color: #f9f4eb;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(18px, 2.6vw, 29px);
  line-height: 1.55;
}

.card-content :deep(p) { margin: 0 0 12px; }
.card-content :deep(ul),
.card-content :deep(ol) { text-align: left; }
.card-content :deep(img) { max-width: 100%; height: auto; }
.card-content :deep(.katex-display) { overflow-x: auto; margin: 20px 0; }

.card-half {
  flex: 1 1 0;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  padding: 30px clamp(24px, 6vw, 72px);
}

.card-front {
  background: #302e29;
}

.card-back {
  background: #34312b;
}

.card-divider {
  position: relative;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #272521;
  color: #918878;
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.card-divider::before {
  position: absolute;
  right: 32px;
  left: 32px;
  height: 1px;
  background: #49443b;
  content: '';
}

.card-divider span {
  z-index: 1;
  padding: 5px 10px;
  background: #272521;
}

.review-card.revealed .card-half {
  min-height: 0;
}

.card-back.hidden .card-label,
.card-back.hidden .card-content {
  visibility: hidden;
}

.review-message {
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 9px;
  color: #9a9187;
  font-size: 13px;
}

.review-message strong { color: #62584d; font-size: 18px; }
.error-message { color: #a65345; }

.review-footer {
  display: grid;
  grid-template-columns: 72px 1fr 72px;
  align-items: center;
  gap: 8px;
  padding: 5px clamp(12px, 4vw, 52px) 7px;
  border-top: 1px solid #e1dcd4;
  background: rgba(255, 255, 255, 0.94);
}

.edit-button,
.more-button {
  justify-self: start;
  min-width: 62px;
  height: 30px;
  padding: 0 7px;
  font-size: 11px;
}

.more-button { justify-self: end; }

.grade-actions {
  display: grid;
  grid-template-columns: repeat(4, minmax(90px, 1fr));
  gap: 6px;
}

.grade-button {
  min-height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 2px;
  border: 1px solid #e0dbd3;
  border-radius: 9px;
  background: #fff;
  color: #4f473e;
  cursor: pointer;
}

.grade-button:hover:not(:disabled) { background: #f0ede8; }
.grade-button.again { border-bottom: 3px solid #c78368; }
.grade-button.hard { border-bottom: 3px solid #c2a36c; }
.grade-button.good { border-bottom: 3px solid #91a17f; }
.grade-button.easy { border-bottom: 3px solid #789386; }
.grade-time { color: #9b9187; font-size: 9px; }
.grade-button strong { font-size: 11px; font-weight: 600; }

@media (max-width: 700px) {
  .review-header { align-items: flex-start; flex-direction: column; }
  .review-tools { width: 100%; }
  .deck-picker { flex: 1; width: auto; }
  .review-footer { grid-template-columns: 1fr; gap: 10px; }
  .edit-button,
  .more-button { justify-self: stretch; }
  .grade-actions { grid-template-columns: repeat(2, 1fr); }
}

.review-page .deck-option-icon,
.review-page .deck-option small,
.review-page .card-position,
.review-page .review-message,
.review-page .grade-time {
  color: var(--learning-text-muted);
}

.review-page .review-message strong {
  color: var(--learning-primary);
}

.review-page .card-label {
  color: #8db7f4;
}

.review-page .review-card {
  border-color: #1d3557;
  background: #17233b;
}

.review-page .card-front {
  background: #1b2b46;
}

.review-page .card-back {
  background: #203653;
}

.review-page .card-divider {
  background: #17233b;
  color: #8fa4c2;
}

.review-page .card-divider::before {
  background: #385579;
}

.review-page .card-divider span {
  background: #17233b;
}

.review-page .grade-button:hover:not(:disabled) {
  background: #eef5ff;
}

.review-page .grade-button.again,
.review-page .grade-button.hard,
.review-page .grade-button.good,
.review-page .grade-button.easy {
  border-bottom-color: var(--learning-primary);
}
</style>
