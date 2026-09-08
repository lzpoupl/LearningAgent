<template>
  <main class="manager-page">
    <header class="manager-header">
      <div>
        <div class="eyebrow">学习资产 / ANKI</div>
        <h1>卡片管理</h1>
        <p>按牌组整理、检索和维护你的记忆卡片。</p>
      </div>
      <div class="header-actions">
        <button class="outline-button" type="button" :disabled="loading" @click="reloadDecks">↻ 刷新</button>
        <button class="dark-button" type="button" @click="showCreateDeck = true">＋ 新建牌组</button>
      </div>
    </header>

    <section class="manager-layout">
      <aside class="deck-panel">
        <div class="panel-title-row">
          <div>
            <span class="panel-kicker">DECK TREE</span>
            <h2>牌组</h2>
          </div>
          <span class="count-label">{{ deckCount }} 个</span>
        </div>

        <div v-if="loading" class="state-message">正在加载牌组...</div>
        <div v-else-if="deckRows.length === 0" class="state-message empty-state">
          <strong>还没有牌组</strong>
          <span>创建一个牌组开始整理卡片。</span>
          <button class="text-button" type="button" @click="showCreateDeck = true">创建牌组 →</button>
        </div>
        <div v-else class="deck-tree" role="tree">
          <button
            class="deck-row all-cards-row"
            :class="{ active: selectedDeckPath === '' }"
            type="button"
            role="treeitem"
            @click="selectDeck('')"
          >
            <span class="tree-toggle invisible" aria-hidden="true">›</span>
            <span class="deck-icon" aria-hidden="true">▦</span>
            <span class="deck-name">全部卡片</span>
            <span class="deck-card-count">{{ totalCardCount }}</span>
          </button>
          <button
            v-for="row in deckRows"
            :key="row.deck.path"
            class="deck-row"
            :class="{ active: selectedDeckPath === row.deck.path }"
            :style="{ paddingLeft: `${14 + row.depth * 18}px` }"
            type="button"
            role="treeitem"
            :aria-expanded="row.hasChildren ? row.expanded : undefined"
            @click="selectDeck(row.deck.path)"
          >
            <span
              class="tree-toggle"
              :class="{ invisible: !row.hasChildren }"
              aria-hidden="true"
              @click.stop="toggleDeck(row.deck.path)"
            >{{ row.expanded ? '⌄' : '›' }}</span>
            <span class="deck-icon" aria-hidden="true">▱</span>
            <span class="deck-name">{{ row.deck.name || row.deck.path }}</span>
            <span class="deck-card-count">{{ row.deck.cardCount }}</span>
          </button>
        </div>
      </aside>

      <section class="cards-panel">
        <div class="cards-toolbar">
          <div>
            <span class="panel-kicker">CARD LIBRARY</span>
            <h2>{{ selectedDeckPath || '全部卡片' }}</h2>
          </div>
          <div v-if="selectedDeckPath" class="toolbar-actions">
            <button class="outline-button" type="button" @click="showMoveDeck = true">移动牌组</button>
            <button class="danger-outline" type="button" @click="deleteSelectedDeck">删除牌组</button>
          </div>
        </div>

        <div class="filters">
          <div class="search-box">
            <span aria-hidden="true">⌕</span>
            <input v-model="keyword" type="search" placeholder="搜索正面或背面" @keydown.enter="loadCards" />
            <button v-if="keyword" type="button" title="清除搜索" @click="clearSearch">×</button>
          </div>
          <select v-model="stateFilter" aria-label="卡片状态" @change="loadCards">
            <option value="">全部状态</option>
            <option value="new">新卡</option>
            <option value="learning">学习中</option>
            <option value="review">复习中</option>
            <option value="relearning">重新学习</option>
          </select>
          <button class="outline-button filter-button" type="button" :disabled="loadingCards" @click="loadCards">查询</button>
        </div>

        <div v-if="errorMessage" class="feedback error" role="alert">{{ errorMessage }}</div>
        <div v-if="successMessage" class="feedback success" role="status">{{ successMessage }}</div>

        <div v-if="loadingCards" class="cards-state">正在加载卡片...</div>
        <div v-else-if="cards.length === 0" class="cards-state empty-cards">
          <strong>没有找到卡片</strong>
          <span>{{ keyword ? '试试换一个搜索词。' : '这个牌组还没有卡片。' }}</span>
        </div>
        <div v-else class="card-list">
          <article
            v-for="card in cards"
            :key="card.id"
            class="card-item"
            tabindex="0"
            @click="openPreview(card)"
            @keydown.enter="openPreview(card)"
          >
            <div class="card-content">
              <div class="card-side">
                <span class="side-label">正面</span>
                <p>{{ card.front }}</p>
              </div>
              <div class="card-side back-side">
                <span class="side-label">背面</span>
                <p>{{ card.back }}</p>
              </div>
            </div>
            <div class="card-meta">
              <span class="state-badge" :class="`state-${card.state}`">{{ stateLabel(card.state) }}</span>
              <span>下次复习：{{ formatDueAt(card.dueAt) }}</span>
              <span class="card-actions">
                <button type="button" title="编辑卡片" @click.stop="openEditCard(card)">编辑</button>
                <button type="button" title="移动卡片" @click.stop="openMoveCard(card)">移动</button>
                <button type="button" title="重置记忆" @click.stop="resetSelectedCard(card)">忘记</button>
                <button class="delete-text" type="button" title="删除卡片" @click.stop="deleteCardItem(card)">删除</button>
              </span>
            </div>
            <div class="review-actions">
              <span>复习评分</span>
              <button type="button" @click.stop="gradeSelectedCard(card, 'again')">重来</button>
              <button type="button" @click.stop="gradeSelectedCard(card, 'hard')">困难</button>
              <button type="button" @click.stop="gradeSelectedCard(card, 'good')">良好</button>
              <button type="button" @click.stop="gradeSelectedCard(card, 'easy')">简单</button>
            </div>
          </article>
        </div>
      </section>
    </section>

    <div v-if="previewCard" class="modal-backdrop" @click.self="previewCard = null">
      <section class="modal preview-modal">
        <div class="modal-heading">
          <div>
            <span class="panel-kicker">CARD PREVIEW</span>
            <h2>卡片预览</h2>
          </div>
          <button class="close-button" type="button" aria-label="关闭" @click="previewCard = null">×</button>
        </div>
        <div class="preview-meta">
          <span class="state-badge" :class="`state-${previewCard.state}`">{{ stateLabel(previewCard.state) }}</span>
          <span>{{ previewCard.deckPath }}</span>
          <span>下次复习：{{ formatDueAt(previewCard.dueAt) }}</span>
        </div>
        <div class="preview-side">
          <span class="side-label">正面</span>
          <div class="rendered-card-content" v-html="renderCardContent(previewCard.front)" />
        </div>
        <div class="preview-side preview-side-back">
          <span class="side-label">背面</span>
          <div class="rendered-card-content" v-html="renderCardContent(previewCard.back)" />
        </div>
        <div class="modal-actions">
          <button class="outline-button" type="button" @click="previewCard = null">关闭</button>
          <button class="dark-button" type="button" @click="editPreviewCard">编辑卡片</button>
        </div>
      </section>
    </div>

    <div v-if="showCreateDeck" class="modal-backdrop" @click.self="showCreateDeck = false">
      <form class="modal" @submit.prevent="createDeckItem">
        <div class="modal-heading">
          <div>
            <span class="panel-kicker">NEW DECK</span>
            <h2>新建牌组</h2>
          </div>
          <button class="close-button" type="button" aria-label="关闭" @click="showCreateDeck = false">×</button>
        </div>
        <label class="field-label" for="new-deck-path">牌组路径</label>
        <input id="new-deck-path" v-model="newDeckPath" autofocus placeholder="例如：数学 / 错题" />
        <p class="modal-hint">使用 / 创建层级牌组，例如“英语 / 例句”。</p>
        <div class="modal-actions">
          <button class="outline-button" type="button" @click="showCreateDeck = false">取消</button>
          <button class="dark-button" type="submit" :disabled="!newDeckPath.trim() || actionLoading">{{ actionLoading ? '创建中...' : '创建牌组' }}</button>
        </div>
      </form>
    </div>

    <div v-if="editingCard" class="modal-backdrop" @click.self="editingCard = null">
      <form class="modal edit-modal" @submit.prevent="saveCardEdit">
        <div class="modal-heading">
          <div>
            <span class="panel-kicker">EDIT CARD</span>
            <h2>编辑卡片</h2>
          </div>
          <button class="close-button" type="button" aria-label="关闭" @click="editingCard = null">×</button>
        </div>
        <label class="field-label" for="edit-front">正面</label>
        <textarea id="edit-front" v-model="editFront" rows="5" />
        <label class="field-label" for="edit-back">背面</label>
        <textarea id="edit-back" v-model="editBack" rows="7" />
        <div class="modal-actions">
          <button class="outline-button" type="button" @click="editingCard = null">取消</button>
          <button class="dark-button" type="submit" :disabled="!editFront.trim() || !editBack.trim() || actionLoading">{{ actionLoading ? '保存中...' : '保存修改' }}</button>
        </div>
      </form>
    </div>

    <div v-if="movingCard" class="modal-backdrop" @click.self="movingCard = null">
      <form class="modal" @submit.prevent="moveSelectedCard">
        <div class="modal-heading">
          <div>
            <span class="panel-kicker">MOVE CARD</span>
            <h2>移动卡片</h2>
          </div>
          <button class="close-button" type="button" aria-label="关闭" @click="movingCard = null">×</button>
        </div>
        <label class="field-label" for="move-target">目标牌组</label>
        <select id="move-target" v-model="moveTargetPath">
          <option value="" disabled>选择目标牌组</option>
          <option v-for="deck in allDecks" :key="deck.path" :value="deck.path">{{ deck.path }}</option>
        </select>
        <div class="modal-actions">
          <button class="outline-button" type="button" @click="movingCard = null">取消</button>
          <button class="dark-button" type="submit" :disabled="!moveTargetPath || actionLoading">{{ actionLoading ? '移动中...' : '确认移动' }}</button>
        </div>
      </form>
    </div>

    <div v-if="showMoveDeck" class="modal-backdrop" @click.self="showMoveDeck = false">
      <form class="modal" @submit.prevent="moveSelectedDeck">
        <div class="modal-heading">
          <div>
            <span class="panel-kicker">MOVE DECK</span>
            <h2>移动牌组</h2>
          </div>
          <button class="close-button" type="button" aria-label="关闭" @click="showMoveDeck = false">×</button>
        </div>
        <label class="field-label" for="move-deck-target">新牌组路径</label>
        <input id="move-deck-target" v-model="moveDeckTargetPath" placeholder="例如：数学 / 已掌握" />
        <p class="modal-hint">当前牌组：{{ selectedDeckPath }}</p>
        <div class="modal-actions">
          <button class="outline-button" type="button" @click="showMoveDeck = false">取消</button>
          <button class="dark-button" type="submit" :disabled="!moveDeckTargetPath.trim() || actionLoading">{{ actionLoading ? '移动中...' : '确认移动' }}</button>
        </div>
      </form>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import DOMPurify from 'dompurify'
import katex from 'katex'
import 'katex/dist/katex.min.css'
import { marked } from 'marked'

import {
  createDeck,
  deleteCard,
  deleteDeck,
  getCard,
  getCards,
  getSubdecks,
  gradeCard,
  moveCard,
  moveDeck,
  resetCard,
  searchCards,
  updateCardContent,
} from '../services/anki'
import type { Card, CardGrade, CardState, Deck } from '../types/anki'

const emit = defineEmits<{
  'edit-card': [card: Card]
}>()

type DeckNode = {
  deck: Deck
  children: DeckNode[]
  expanded: boolean
}

type DeckRow = {
  deck: Deck
  depth: number
  expanded: boolean
  hasChildren: boolean
}

const deckTree = ref<DeckNode[]>([])
const selectedDeckPath = ref('')
const cards = ref<Card[]>([])
const keyword = ref('')
const stateFilter = ref<CardState | ''>('')
const loading = ref(false)
const loadingCards = ref(false)
const actionLoading = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const showCreateDeck = ref(false)
const showMoveDeck = ref(false)
const newDeckPath = ref('')
const moveDeckTargetPath = ref('')
const previewCard = ref<Card | null>(null)
const editingCard = ref<Card | null>(null)
const editFront = ref('')
const editBack = ref('')
const movingCard = ref<Card | null>(null)
const moveTargetPath = ref('')

const allDecks = computed(() => flattenDecks(deckTree.value))
const deckCount = computed(() => allDecks.value.length)
const deckRows = computed(() => flattenVisibleDecks(deckTree.value))
const totalCardCount = computed(() => allDecks.value.reduce((total, deck) => total + deck.cardCount, 0))

function flattenDecks(nodes: DeckNode[]): Deck[] {
  return nodes.flatMap(node => [node.deck, ...flattenDecks(node.children)])
}

function flattenVisibleDecks(nodes: DeckNode[], depth = 0): DeckRow[] {
  return nodes.flatMap(node => [
    {
      deck: node.deck,
      depth,
      expanded: node.expanded,
      hasChildren: node.children.length > 0,
    },
    ...(node.expanded ? flattenVisibleDecks(node.children, depth + 1) : []),
  ])
}

async function fetchDeckTree() {
  const roots = await getSubdecks('')
  return Promise.all(roots.map(deck => buildDeckNode(deck)))
}

async function buildDeckNode(deck: Deck): Promise<DeckNode> {
  const children = await getSubdecks(deck.path)
  return {
    deck,
    children: await Promise.all(children.map(child => buildDeckNode(child))),
    expanded: false,
  }
}

async function reloadDecks() {
  loading.value = true
  errorMessage.value = ''

  try {
    deckTree.value = await fetchDeckTree()

    if (selectedDeckPath.value && allDecks.value.some(deck => deck.path === selectedDeckPath.value)) {
      await loadCards()
    } else {
      selectedDeckPath.value = allDecks.value[0]?.path || ''
      await loadCards()
    }
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组加载失败，请确认 Anki 服务已连接。'
  } finally {
    loading.value = false
  }
}

async function selectDeck(deckPath: string) {
  selectedDeckPath.value = deckPath
  keyword.value = ''
  stateFilter.value = ''
  await loadCards()
}

function toggleDeck(deckPath: string) {
  const node = findDeckNode(deckTree.value, deckPath)
  if (node) {
    node.expanded = !node.expanded
  }
}

function findDeckNode(nodes: DeckNode[], deckPath: string): DeckNode | null {
  for (const node of nodes) {
    if (node.deck.path === deckPath) {
      return node
    }
    const found = findDeckNode(node.children, deckPath)
    if (found) {
      return found
    }
  }
  return null
}

async function loadCards() {
  loadingCards.value = true
  errorMessage.value = ''

  try {
    if (keyword.value.trim()) {
      cards.value = await searchCards(keyword.value.trim(), {
        deckPath: selectedDeckPath.value || undefined,
      })
      if (stateFilter.value) {
        cards.value = cards.value.filter(card => card.state === stateFilter.value)
      }
    } else if (selectedDeckPath.value) {
      cards.value = await getCards(selectedDeckPath.value, {
        state: stateFilter.value || undefined,
      })
    } else {
      cards.value = await searchCards('', {})
      if (stateFilter.value) {
        cards.value = cards.value.filter(card => card.state === stateFilter.value)
      }
    }
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片加载失败，请稍后重试。'
  } finally {
    loadingCards.value = false
  }
}

function clearSearch() {
  keyword.value = ''
  loadCards()
}

function openPreview(card: Card) {
  previewCard.value = card
}

function editPreviewCard() {
  if (!previewCard.value) {
    return
  }

  const card = previewCard.value
  previewCard.value = null
  emit('edit-card', card)
}

function renderCardContent(content: string) {
  const formulas: string[] = []
  const replaceFormula = (formula: string, displayMode: boolean) => {
    const index = formulas.length
    formulas.push(katex.renderToString(formula.trim(), {
      displayMode,
      throwOnError: false,
    }))
    return `ANKI_KATEX_FORMULA_${index}`
  }

  const withPlaceholders = content
    .replace(/\$\$([\s\S]*?)\$\$/g, (_, formula: string) => replaceFormula(formula, true))
    .replace(/\$([^$\n]+?)\$/g, (_, formula: string) => replaceFormula(formula, false))

  let html = marked.parse(withPlaceholders, { async: false })
  formulas.forEach((formula, index) => {
    html = html.replace(`ANKI_KATEX_FORMULA_${index}`, formula)
  })

  return DOMPurify.sanitize(html)
}

async function createDeckItem() {
  const path = newDeckPath.value.trim()
  if (!path || actionLoading.value) {
    return
  }

  actionLoading.value = true
  errorMessage.value = ''

  try {
    const deck = await createDeck(path)
    showCreateDeck.value = false
    newDeckPath.value = ''
    selectedDeckPath.value = deck.path
    successMessage.value = `牌组“${deck.path}”已创建。`
    await reloadDecks()
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组创建失败，请检查牌组路径。'
  } finally {
    actionLoading.value = false
  }
}

async function deleteSelectedDeck() {
  if (!selectedDeckPath.value || !window.confirm(`确定删除牌组“${selectedDeckPath.value}”及其中的卡片吗？`)) {
    return
  }

  actionLoading.value = true
  try {
    await deleteDeck(selectedDeckPath.value)
    successMessage.value = '牌组已删除。'
    selectedDeckPath.value = ''
    await reloadDecks()
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组删除失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

function openEditCard(card: Card) {
  getCard(card.id)
    .then(latestCard => {
      editingCard.value = latestCard
      editFront.value = latestCard.front
      editBack.value = latestCard.back
    })
    .catch(error => {
      console.error(error)
      errorMessage.value = '卡片详情加载失败，请重试。'
    })
}

async function moveSelectedDeck() {
  if (!selectedDeckPath.value || !moveDeckTargetPath.value.trim() || actionLoading.value) {
    return
  }

  actionLoading.value = true
  try {
    const movedDeck = await moveDeck(selectedDeckPath.value, moveDeckTargetPath.value.trim())
    selectedDeckPath.value = movedDeck.path
    moveDeckTargetPath.value = ''
    showMoveDeck.value = false
    successMessage.value = '牌组已移动。'
    await reloadDecks()
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组移动失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

async function saveCardEdit() {
  if (!editingCard.value || !editFront.value.trim() || !editBack.value.trim()) {
    return
  }

  actionLoading.value = true
  try {
    await updateCardContent(editingCard.value.id, {
      front: editFront.value.trim(),
      back: editBack.value.trim(),
    })
    editingCard.value = null
    successMessage.value = '卡片内容已更新。'
    await loadCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片更新失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

function openMoveCard(card: Card) {
  movingCard.value = card
  moveTargetPath.value = card.deckPath
}

async function moveSelectedCard() {
  if (!movingCard.value || !moveTargetPath.value || moveTargetPath.value === movingCard.value.deckPath) {
    return
  }

  actionLoading.value = true
  try {
    await moveCard(movingCard.value.id, moveTargetPath.value)
    movingCard.value = null
    successMessage.value = '卡片已移动。'
    await loadCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片移动失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

async function deleteCardItem(card: Card) {
  if (!window.confirm('确定删除这张卡片吗？')) {
    return
  }

  actionLoading.value = true
  try {
    await deleteCard(card.id)
    successMessage.value = '卡片已删除。'
    await loadCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片删除失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

async function resetSelectedCard(card: Card) {
  actionLoading.value = true
  try {
    await resetCard(card.id)
    successMessage.value = '卡片已重置为新卡状态。'
    await loadCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片重置失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

async function gradeSelectedCard(card: Card, grade: CardGrade) {
  actionLoading.value = true
  try {
    await gradeCard(card.id, grade)
    successMessage.value = '复习结果已记录。'
    await loadCards()
  } catch (error) {
    console.error(error)
    errorMessage.value = '复习结果记录失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

function stateLabel(state: CardState) {
  const labels: Record<CardState, string> = {
    new: '新卡',
    learning: '学习中',
    review: '复习中',
    relearning: '重新学习',
  }
  return labels[state]
}

function formatDueAt(value: string | null) {
  if (!value) {
    return '尚未安排'
  }
  return new Date(value).toLocaleString('zh-CN', {
    month: 'numeric',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  })
}

onMounted(reloadDecks)
</script>

<style scoped>
.manager-page {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
  padding: 42px clamp(20px, 5vw, 70px) 60px;
  background: #f8f8f6;
  color: #292723;
}

.manager-header,
.manager-layout {
  width: min(1220px, 100%);
  margin: 0 auto;
}

.manager-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 28px;
}

.eyebrow,
.panel-kicker {
  color: #8a735a;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  margin-top: 9px;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(30px, 4vw, 46px);
  font-weight: 500;
  line-height: 1.1;
}

.manager-header p {
  margin-top: 10px;
  color: #77736d;
  font-size: 14px;
}

.header-actions,
.modal-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

button,
input,
select,
textarea {
  font: inherit;
}

button {
  cursor: pointer;
}

button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.outline-button,
.dark-button,
.danger-outline,
.text-button {
  border-radius: 6px;
  font-size: 12px;
}

.outline-button {
  padding: 9px 12px;
  border: 1px solid #d8d3ca;
  background: #fff;
  color: #514b43;
}

.dark-button {
  padding: 10px 14px;
  border: 1px solid #25231f;
  background: #25231f;
  color: #fff;
}

.manager-layout {
  display: grid;
  grid-template-columns: minmax(230px, 0.3fr) minmax(0, 0.7fr);
  gap: 18px;
  align-items: start;
}

.deck-panel,
.cards-panel {
  min-width: 0;
  border: 1px solid #e5e1da;
  border-radius: 8px;
  background: #fff;
}

.deck-panel {
  min-height: 520px;
  padding: 20px 10px;
}

.panel-title-row,
.cards-toolbar,
.modal-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.panel-title-row {
  padding: 0 8px 16px;
}

h2 {
  margin-top: 6px;
  font-size: 20px;
  font-weight: 650;
}

.count-label,
.card-meta,
.modal-hint {
  color: #9a948b;
  font-size: 12px;
}

.state-message,
.cards-state {
  display: flex;
  flex-direction: column;
  gap: 5px;
  padding: 30px 12px;
  color: #9a948b;
  font-size: 12px;
  line-height: 1.5;
}

.empty-state strong,
.empty-cards strong {
  color: #625b52;
  font-size: 14px;
}

.text-button {
  width: fit-content;
  margin-top: 8px;
  padding: 0;
  border: 0;
  background: transparent;
  color: #896f52;
}

.deck-tree {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.deck-row {
  width: 100%;
  min-height: 38px;
  display: flex;
  align-items: center;
  gap: 6px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #4f4a43;
  text-align: left;
}

.deck-row:hover,
.deck-row.active {
  background: #f0ede8;
}

.deck-row.active {
  color: #2c2925;
  font-weight: 650;
}

.all-cards-row {
  margin-bottom: 6px;
  border-bottom: 1px solid #eeeae4;
  border-radius: 0;
}

.tree-toggle {
  width: 14px;
  color: #938a7f;
  font-size: 17px;
  text-align: center;
}

.tree-toggle.invisible {
  visibility: hidden;
}

.deck-icon {
  color: #9e8567;
  font-size: 17px;
}

.deck-name {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: 13px;
}

.deck-card-count {
  margin-left: auto;
  padding-right: 10px;
  color: #aba39a;
  font-size: 11px;
}

.cards-panel {
  padding: 22px;
}

.cards-toolbar {
  margin-bottom: 18px;
}

.toolbar-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.danger-outline {
  padding: 8px 10px;
  border: 1px solid #e1c8c0;
  background: #fff;
  color: #a65345;
}

.filters {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.search-box {
  display: flex;
  align-items: center;
  flex: 1;
  gap: 8px;
  min-width: 0;
  height: 38px;
  padding: 0 10px;
  border: 1px solid #dedbd5;
  border-radius: 6px;
  background: #fcfcfb;
  color: #9a948b;
}

.search-box input,
.filters select,
.modal input,
.modal select,
.modal textarea {
  border: 1px solid #dedbd5;
  border-radius: 6px;
  outline: none;
  background: #fcfcfb;
  color: #292723;
  font-size: 13px;
}

.search-box input {
  min-width: 0;
  flex: 1;
  border: 0;
  background: transparent;
}

.search-box button {
  border: 0;
  background: transparent;
  color: #8d857c;
  font-size: 18px;
}

.filters select,
.modal input,
.modal select {
  height: 38px;
  padding: 0 10px;
}

.filter-button {
  height: 38px;
}

.feedback {
  margin-bottom: 14px;
  padding: 9px 11px;
  border-radius: 6px;
  font-size: 12px;
}

.feedback.error {
  background: #fff1ee;
  color: #a34e3f;
}

.feedback.success {
  background: #eef7ef;
  color: #4d7652;
}

.card-list {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.card-item {
  overflow: hidden;
  border: 1px solid #e7e3dc;
  border-radius: 7px;
  background: #fff;
}

.card-content {
  display: grid;
  grid-template-columns: 1fr 1fr;
}

.card-side {
  min-width: 0;
  padding: 16px;
}

.back-side {
  border-left: 1px solid #eeeae4;
  background: #fcfbf9;
}

.side-label {
  color: #a08c75;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
}

.card-side p {
  display: -webkit-box;
  margin-top: 10px;
  overflow: hidden;
  color: #49443d;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 15px;
  line-height: 1.5;
  line-clamp: 4;
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 4;
}

.card-meta,
.review-actions {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 9px;
  padding: 9px 16px;
  border-top: 1px solid #eeeae4;
}

.state-badge {
  padding: 3px 6px;
  border-radius: 4px;
  font-size: 10px;
}

.state-new {
  background: #f0eee9;
  color: #777067;
}

.state-learning,
.state-relearning {
  background: #fff3df;
  color: #9b6d32;
}

.state-review {
  background: #edf5ed;
  color: #5d805f;
}

.card-actions {
  display: inline-flex;
  gap: 9px;
  margin-left: auto;
}

.card-actions button,
.review-actions button {
  padding: 0;
  border: 0;
  background: transparent;
  color: #806a51;
  font-size: 11px;
}

.card-actions button:hover,
.review-actions button:hover {
  color: #332b22;
  text-decoration: underline;
}

.card-actions .delete-text {
  color: #a65345;
}

.review-actions {
  justify-content: flex-end;
  background: #faf9f7;
  color: #a29a91;
  font-size: 11px;
}

.modal-backdrop {
  position: fixed;
  z-index: 30;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: rgba(33, 30, 26, 0.35);
}

.modal {
  width: min(460px, 100%);
  padding: 24px;
  border-radius: 8px;
  background: #fff;
  box-shadow: 0 20px 60px rgba(26, 22, 18, 0.2);
}

.edit-modal {
  width: min(620px, 100%);
}

.preview-modal {
  width: min(700px, 100%);
}

.preview-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  margin-top: 18px;
  color: #8e877e;
  font-size: 12px;
}

.preview-side {
  margin-top: 18px;
  padding: 18px;
  border: 1px solid #e7e3dc;
  border-radius: 6px;
  background: #fcfbf9;
}

.preview-side-back {
  background: #f7f4ef;
}

.rendered-card-content {
  margin-top: 12px;
  color: #403a33;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 17px;
  line-height: 1.6;
}

.rendered-card-content :deep(p) {
  margin: 0 0 10px;
}

.rendered-card-content :deep(p:last-child) {
  margin-bottom: 0;
}

.rendered-card-content :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 10px auto;
  border-radius: 5px;
}

.rendered-card-content :deep(.katex-display) {
  overflow-x: auto;
  margin: 14px 0;
  text-align: center;
}

.rendered-card-content :deep(pre) {
  overflow-x: auto;
  padding: 10px;
  border-radius: 5px;
  background: #ebe7e0;
  font-family: Consolas, monospace;
  font-size: 13px;
}

.close-button {
  width: 30px;
  height: 30px;
  border: 0;
  border-radius: 6px;
  background: #f1efec;
  color: #71685f;
  font-size: 20px;
}

.modal .field-label {
  margin-top: 22px;
}

.modal input,
.modal select,
.modal textarea {
  width: 100%;
}

.modal textarea {
  display: block;
  padding: 10px;
  resize: vertical;
  line-height: 1.55;
}

.modal-hint {
  margin-top: 9px;
}

.modal-actions {
  justify-content: flex-end;
  margin-top: 24px;
}

@media (max-width: 850px) {
  .manager-page {
    padding: 28px 16px 48px;
  }

  .manager-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .manager-layout {
    grid-template-columns: 1fr;
  }

  .deck-panel {
    min-height: 0;
  }
}

@media (max-width: 620px) {
  .cards-panel {
    padding: 16px;
  }

  .filters {
    flex-wrap: wrap;
  }

  .search-box {
    flex-basis: 100%;
  }

  .card-content {
    grid-template-columns: 1fr;
  }

  .back-side {
    border-top: 1px solid #eeeae4;
    border-left: 0;
  }

  .card-actions {
    width: 100%;
    margin-left: 0;
  }
}
</style>
