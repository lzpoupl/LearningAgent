<template>
  <main class="manager-page">
    <header class="manager-header">
      <div>
        <div class="eyebrow">学习资产 / ANKI</div>
        <h1>卡片管理</h1>
        <p>按牌组整理、检索和维护你的记忆卡片。</p>
      </div>
      <div class="header-actions">
        <button class="outline-button" type="button" @click="$emit('create-card')">＋ 新建卡片</button>
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
          <button class="deck-row all-cards-row" type="button"
            role="treeitem" @click="selectDeck('')">
            <span class="tree-toggle invisible" aria-hidden="true">›</span>
            <span class="deck-icon" aria-hidden="true">▦</span>
            <span class="deck-name">全部卡片</span>
            <span class="deck-card-count">{{ totalCardCount }}</span>
          </button>
          <button v-for="row in deckRows" :key="row.deck.path" class="deck-row"
            :class="{ active: selectedDeckPath === row.deck.path, 'drop-target': dropTargetPath === row.deck.path }"
            :style="{ paddingLeft: `${14 + row.depth * 18}px` }"
            type="button" role="treeitem" draggable="true"
            :aria-expanded="row.hasChildren ? row.expanded : undefined"
            @dragstart="startDeckDrag(row.deck.path, $event)"
            @dragover.prevent="allowDeckDrop(row.deck.path, $event)"
            @dragleave="clearDeckDrop"
            @drop.prevent="dropDeck(row.deck.path, $event)"
            @dragend="clearDeckDrag"
            @click="selectAndToggleDeck(row.deck.path)">
            <span class="tree-toggle" :class="{ invisible: !row.hasChildren }" aria-hidden="true"
              @click.stop="toggleDeck(row.deck.path)">{{ row.expanded ? '⌄' : '›' }}</span>
            <span class="deck-icon" aria-hidden="true">▱</span>
            <span class="deck-name">{{ row.deck.name || row.deck.path }}</span>
            <span class="deck-card-count">{{ row.cardCount }}</span>
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
            <input v-model="keyword" type="text" placeholder="搜索正面或背面" @keydown.enter="loadCards" />
            <button v-if="keyword" type="button" title="清除搜索" @click="clearSearch">×</button>
          </div>
          <select v-model="stateFilter" aria-label="卡片状态" @change="loadCards">
            <option value="">全部状态</option>
            <option value="new">新卡</option>
            <option value="learning">学习中</option>
            <option value="review">复习中</option>
            <option value="relearning">重新学习</option>
          </select>
          <button class="outline-button filter-button" type="button" :disabled="loadingCards"
            @click="loadCards">查询</button>
        </div>

        <div v-if="errorMessage" class="feedback error" role="alert">{{ errorMessage }}</div>
        <div v-if="successMessage" class="feedback success" role="status">{{ successMessage }}</div>

        <div v-if="loadingCards" class="cards-state">正在加载卡片...</div>
        <div v-else-if="cards.length === 0" class="cards-state empty-cards">
          <strong>没有找到卡片</strong>
          <span>{{ keyword ? '试试换一个搜索词。' : '这个牌组还没有卡片。' }}</span>
        </div>
        <div v-else class="card-list">
          <article v-for="card in cards" :key="card.id" class="card-item" tabindex="0" @click="openPreview(card)"
            @keydown.enter="openPreview(card)">
            <div class="card-content">
              <div class="card-side">
                <span class="side-label">正面</span>
               <CardContent class="card-rendered-content" :content="card.front" />
              </div>
              <div class="card-side back-side">
                <span class="side-label">背面</span>
                 <CardContent class="card-rendered-content" :content="card.back" />
              </div>
            </div>
            <div class="card-meta">
              <span class="card-deck-path">{{ card.deckPath }}</span>
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
           <CardContent class="rendered-card-content" :content="previewCard.front" />
        </div>
        <div class="preview-side preview-side-back">
          <span class="side-label">背面</span>
           <CardContent class="rendered-card-content" :content="previewCard.back" />
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
          <button class="dark-button" type="submit" :disabled="!newDeckPath.trim() || actionLoading">{{ actionLoading ?
            '创建中...' : '创建牌组' }}</button>
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
          <button class="dark-button" type="submit" :disabled="!moveTargetPath || actionLoading">{{ actionLoading ?
            '移动中...' : '确认移动' }}</button>
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
          <button class="dark-button" type="submit" :disabled="!moveDeckTargetPath.trim() || actionLoading">{{
            actionLoading ? '移动中...' : '确认移动' }}</button>
        </div>
      </form>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import {
  createDeck,
  deleteCard,
  deleteDeck,
  getCard,
  getCards,
  gradeCard,
  moveCard,
  moveDeck,
  resetCard,
  searchCards,
} from '../services/anki'
import type { Card, CardGrade, CardState } from '../types/anki'
import CardContent from '../components/anki/CardContent.vue'
import {
  aggregateCardCount,
  findDeckNode,
  flattenDecks,
  flattenVisibleDecks,
  getDescendantPaths,
  loadDeckTree,
  type DeckNode,
} from '../composables/useDeckTree'

const emit = defineEmits<{
  'create-card': []
  'edit-card': [card: Card]
}>()

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
const movingCard = ref<Card | null>(null)
const moveTargetPath = ref('')
const draggedDeckPath = ref('')
const dropTargetPath = ref('')

const allDecks = computed(() => flattenDecks(deckTree.value))
const deckCount = computed(() => allDecks.value.length)
const deckRows = computed(() => flattenVisibleDecks(deckTree.value))
const totalCardCount = computed(() => deckTree.value.reduce((total, node) => total + aggregateCardCount(node), 0))

async function fetchDeckTree() {
  return loadDeckTree()
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

async function selectAndToggleDeck(deckPath: string) {
  const node = findDeckNode(deckTree.value, deckPath)
  if (!node?.children.length) {
    await selectDeck(deckPath)
    return
  }

  if (node.expanded) {
    node.expanded = false

    if (selectedDeckPath.value === deckPath || selectedDeckPath.value.startsWith(`${deckPath}/`)) {
      selectedDeckPath.value = ''
      keyword.value = ''
      stateFilter.value = ''
      await loadCards()
    }

    return
  }

  node.expanded = true
  await selectDeck(deckPath)
}

function toggleDeck(deckPath: string) {
  const node = findDeckNode(deckTree.value, deckPath)
  if (node) {
    node.expanded = !node.expanded
  }
}

function startDeckDrag(deckPath: string, event: DragEvent) {
  draggedDeckPath.value = deckPath
  dropTargetPath.value = ''
  event.dataTransfer?.setData('text/plain', deckPath)
  if (event.dataTransfer) {
    event.dataTransfer.effectAllowed = 'move'
  }
}

function allowDeckDrop(deckPath: string, event: DragEvent) {
  if (!draggedDeckPath.value || !canMoveDeckInto(draggedDeckPath.value, deckPath)) {
    return
  }

  dropTargetPath.value = deckPath
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move'
  }
}

function clearDeckDrop() {
  dropTargetPath.value = ''
}

function clearDeckDrag() {
  draggedDeckPath.value = ''
  dropTargetPath.value = ''
}

function canMoveDeckInto(sourcePath: string, targetParentPath: string) {
  return sourcePath !== targetParentPath && !targetParentPath.startsWith(`${sourcePath}/`)
}

async function dropDeck(targetParentPath: string, event: DragEvent) {
  const sourcePath = draggedDeckPath.value || event.dataTransfer?.getData('text/plain') || ''
  clearDeckDrag()

  if (!sourcePath || !canMoveDeckInto(sourcePath, targetParentPath) || actionLoading.value) {
    return
  }

  const sourceName = sourcePath.split('/').pop() || sourcePath
  const targetPath = `${targetParentPath}/${sourceName}`

  if (targetPath === sourcePath) {
    return
  }

  actionLoading.value = true
  errorMessage.value = ''

  try {
    await moveDeck(sourcePath, targetPath)
    selectedDeckPath.value = targetPath
    successMessage.value = `牌组已移动到“${targetPath}”。`
    await reloadDecks()
  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组移动失败，请重试。'
  } finally {
    actionLoading.value = false
  }
}

async function loadCards() {
  loadingCards.value = true
  errorMessage.value = ''

  try {
    const deckPaths = selectedDeckPath.value
      ? getDescendantPaths(selectedDeckPath.value, deckTree.value)
      : allDecks.value.map(deck => deck.path)

    const normalizedKeyword = keyword.value.trim()
    const loadedCards = normalizedKeyword
      ? await Promise.all(deckPaths.map(deckPath => searchCards(normalizedKeyword, {
        deckPath,
        front: true,
        back: true,
      })))
      : await Promise.all(deckPaths.map(deckPath => getCards(deckPath, {
        state: stateFilter.value || undefined,
      })))

    const cardsById = new Map<string, Card>()
    loadedCards.flat().forEach(card => cardsById.set(card.id, card))

    const normalizedKeywordLowerCase = normalizedKeyword.toLocaleLowerCase()
    cards.value = Array.from(cardsById.values())
      .filter(card => {
        if (stateFilter.value && card.state !== stateFilter.value) {
          return false
        }

        if (!normalizedKeywordLowerCase) {
          return true
        }

        return [card.front, card.back].some(content =>
          content.toLocaleLowerCase().includes(normalizedKeywordLowerCase)
        )
      })
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

async function createDeckItem() {
  const path = newDeckPath.value.trim()
  if (!path || actionLoading.value) {
    return
  }

  actionLoading.value = true
  errorMessage.value = ''

  try {
    await createDeck(path)
    showCreateDeck.value = false
    newDeckPath.value = ''
    selectedDeckPath.value = path
    successMessage.value = `牌组“${path}”已创建。`
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
      emit('edit-card', latestCard)
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
    let targetPath = moveDeckTargetPath.value.trim()
    await moveDeck(selectedDeckPath.value, targetPath)
    selectedDeckPath.value = targetPath
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
  background: var(--learning-bg);
  color: var(--learning-text);
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
  color: var(--learning-primary);
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
  color: var(--learning-text-secondary);
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
  border: 1px solid var(--learning-border);
  background: var(--learning-surface);
  color: var(--learning-text-secondary);
}

.dark-button {
  padding: 10px 14px;
  border: 1px solid var(--learning-primary);
  background: var(--learning-primary);
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
  border: 1px solid var(--learning-border);
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
  background: #eaf2ff;
}

.deck-row.drop-target {
  outline: 2px dashed #b08b63;
  outline-offset: -2px;
  background: #f6efe6;
}

.deck-row.active {
  color: var(--learning-primary);
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
  margin: 0;
}

.card-rendered-content {
  max-height: 150px;
  margin-top: 10px;
  overflow: hidden;
  color: #49443d;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 15px;
  line-height: 1.5;
}

.card-rendered-content :deep(p) {
  margin: 0 0 8px;
}

.card-rendered-content :deep(p:last-child) {
  margin-bottom: 0;
}

.card-rendered-content :deep(h1),
.card-rendered-content :deep(h2),
.card-rendered-content :deep(h3) {
  margin: 0 0 8px;
  font-size: 1.05em;
  line-height: 1.3;
}

.card-rendered-content :deep(ul),
.card-rendered-content :deep(ol) {
  margin: 0 0 8px;
  padding-left: 20px;
}

.card-rendered-content :deep(pre) {
  overflow-x: auto;
  margin: 0 0 8px;
  padding: 8px;
  border-radius: 4px;
  background: #eeeae4;
  font-family: Consolas, monospace;
  font-size: 11px;
}

.card-rendered-content :deep(img) {
  display: block;
  max-width: 100%;
  height: auto;
  margin: 8px auto;
  border-radius: 4px;
}

.card-rendered-content :deep(.katex-display) {
  overflow-x: auto;
  margin: 10px 0;
  text-align: center;
}

.card-rendered-content :deep(.katex) {
  font-size: 0.95em;
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

.card-deck-path {
  color: #8a735a;
  font-size: 11px;
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

.manager-page .count-label,
.manager-page .card-meta,
.manager-page .modal-hint,
.manager-page .state-message,
.manager-page .cards-state {
  color: var(--learning-text-muted);
}

.manager-page .deck-icon,
.manager-page .card-deck-path,
.manager-page .side-label,
.manager-page .text-button,
.manager-page .card-actions button,
.manager-page .review-actions button {
  color: var(--learning-primary);
}

.manager-page .deck-row.drop-target {
  outline-color: var(--learning-primary);
  background: #eaf2ff;
}

.manager-page .card-item,
.manager-page .preview-side {
  border-color: var(--learning-border);
}

.manager-page .back-side,
.manager-page .review-actions,
.manager-page .preview-side,
.manager-page .preview-side-back {
  background: var(--learning-surface-muted);
}

.manager-page .state-new {
  background: #eef2f7;
  color: #64748b;
}

.manager-page .state-learning,
.manager-page .state-relearning {
  background: #eaf2ff;
  color: #1d64c8;
}

.manager-page .state-review {
  background: #e6f4ff;
  color: #1677b8;
}

.manager-page .card-actions .delete-text,
.manager-page .danger-outline,
.manager-page .feedback.error {
  color: var(--el-color-danger);
}

.manager-page .feedback.error {
  background: var(--el-color-danger-light-9);
}

.manager-page .feedback.success {
  background: var(--el-color-success-light-9);
  color: var(--el-color-success);
}

.manager-page .modal-backdrop {
  background: rgba(15, 35, 65, 0.38);
}
</style>
