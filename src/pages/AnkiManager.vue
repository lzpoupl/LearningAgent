<template>
  <main class="manager-page">
    <header class="manager-header">
      <div>
        <div class="eyebrow">学习资产 / ANKI</div>
        <h1>卡片管理</h1>
        <p>按牌组整理、检索和维护你的记忆卡片。</p>
      </div>
      <div class="header-actions">
        <el-button :icon="Plus" @click="$emit('create-card')">新建卡片</el-button>
        <el-button type="primary" :icon="Plus" @click="showCreateDeck = true">新建牌组</el-button>
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
        <div v-else-if="deckTree.length === 0" class="state-message empty-state">
          <strong>还没有牌组</strong>
          <span>创建一个牌组开始整理卡片。</span>
          <el-button link type="primary" @click="showCreateDeck = true">创建牌组 →</el-button>
        </div>
        <div v-else class="deck-tree">
          <el-button class="all-cards-row" :class="{ active: selectedDeckPath === '' }" text @click="selectDeck('')">
            <span class="deck-icon" aria-hidden="true">▦</span>
            <span class="deck-name">全部卡片</span>
            <span class="deck-card-count">{{ totalCardCount }}</span>
          </el-button>
          <el-tree ref="deckTreeRef" class="deck-tree-inner" :data="deckTreeData" node-key="path"
            :expand-on-click-node="false" highlight-current default-expand-all draggable :allow-drag="allowDeckDrag"
            :allow-drop="allowDeckDrop" @node-click="handleDeckNodeClick" @node-drop="handleDeckDrop">
            <template #default="{ data }">
              <span class="deck-node">
                <span class="deck-icon" aria-hidden="true">▱</span>
                <span class="deck-name">{{ data.label }}</span>
                <span class="deck-card-count">{{ data.cardCount }}</span>
              </span>
            </template>
          </el-tree>
        </div>
      </aside>

      <section class="cards-panel">
        <div class="cards-toolbar">
          <div>
            <span class="panel-kicker">CARD LIBRARY</span>
            <h2>{{ selectedDeckPath || '全部卡片' }}</h2>
          </div>
          <div v-if="selectedDeckPath" class="toolbar-actions">
            <el-button type="danger" plain @click="deleteSelectedDeck">删除牌组</el-button>
          </div>
        </div>

        <div class="filters">
          <el-input v-model="keyword" class="search-input" :prefix-icon="Search" placeholder="搜索正面或背面" clearable
            @keydown.enter="loadCards" @clear="loadCards" />
          <el-select v-model="stateFilter" class="state-select" aria-label="卡片状态" @change="loadCards">
            <el-option label="全部状态" value="" />
            <el-option label="新卡" value="new" />
            <el-option label="学习中" value="learning" />
            <el-option label="复习中" value="review" />
            <el-option label="重新学习" value="relearning" />
          </el-select>
          <el-button :loading="loadingCards" @click="loadCards">查询</el-button>
        </div>

        <div v-if="loadingCards" class="cards-state">正在加载卡片...</div>
        <div v-else-if="cards.length === 0" class="cards-state empty-cards">
          <strong>没有找到卡片</strong>
          <span>{{ keyword ? '试试换一个搜索词。' : '这个牌组还没有卡片。' }}</span>
        </div>
        <div v-else class="card-list">
          <article v-for="card in cards" :key="card.id" class="card-item" tabindex="0" @click="openPreview(card)"
            @keydown.enter="openPreview(card)">
            <CardShow :front="card.front" :back="card.back" />
            <div class="card-meta">
              <span class="card-deck-path">{{ card.deckPath }}</span>
              <span class="state-badge" :class="`state-${card.state}`">{{ stateLabel(card.state) }}</span>
              <span>下次复习：{{ formatDueAt(card.dueAt) }}</span>
              <span class="card-actions">
                <el-button link type="primary" size="small" @click.stop="openEditCard(card)">编辑</el-button>
                <el-button link type="primary" size="small" @click.stop="openMoveCard(card)">移动</el-button>
                <el-button link type="info" size="small" @click.stop="resetSelectedCard(card)">忘记</el-button>
                <el-button link type="danger" size="small" @click.stop="deleteCardItem(card)">删除</el-button>
              </span>
            </div>
            <div class="review-actions">
              <span>复习评分</span>
              <el-button link type="danger" size="small" @click.stop="gradeSelectedCard(card, 'again')">重来</el-button>
              <el-button link type="warning" size="small" @click.stop="gradeSelectedCard(card, 'hard')">困难</el-button>
              <el-button link type="primary" size="small" @click.stop="gradeSelectedCard(card, 'good')">良好</el-button>
              <el-button link type="success" size="small" @click.stop="gradeSelectedCard(card, 'easy')">简单</el-button>
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
          <el-button :icon="Close" circle aria-label="关闭" @click="previewCard = null" />
        </div>
        <div class="preview-meta">
          <span class="state-badge" :class="`state-${previewCard.state}`">{{ stateLabel(previewCard.state) }}</span>
          <span>{{ previewCard.deckPath }}</span>
          <span>下次复习：{{ formatDueAt(previewCard.dueAt) }}</span>
        </div>
        <CardShow variant="preview" :front="previewCard.front" :back="previewCard.back" />
        <div class="modal-actions">
          <el-button @click="previewCard = null">关闭</el-button>
          <el-button type="primary" @click="editPreviewCard">编辑卡片</el-button>
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
          <el-button :icon="Close" circle aria-label="关闭" @click="showCreateDeck = false" />
        </div>
        <label class="field-label" for="new-deck-parent">所属牌组</label>
        <el-tree-select id="new-deck-parent" v-model="newDeckParent" class="deck-select" :data="deckOptions"
          node-key="value" check-strictly clearable default-expand-all placeholder="不选择则创建在顶层" />
        <label class="field-label" for="new-deck-name">牌组名称</label>
        <el-input id="new-deck-name" v-model="newDeckName" class="deck-name-input" autofocus clearable
          placeholder="例如：错题" />
        <p class="modal-hint">新牌组将创建在所选牌组的下一级；留空则创建在顶层。</p>
        <div class="modal-actions">
          <el-button @click="showCreateDeck = false">取消</el-button>
          <el-button type="primary" native-type="submit" :disabled="!newDeckName.trim()"
            :loading="actionLoading">创建牌组</el-button>
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
          <el-button :icon="Close" circle aria-label="关闭" @click="movingCard = null" />
        </div>
        <label class="field-label" for="move-target">目标牌组</label>
        <el-tree-select id="move-target" v-model="moveTargetPath" class="deck-select" :data="moveDeckOptions"
          node-key="value" check-strictly default-expand-all placeholder="选择目标牌组" />
        <div class="modal-actions">
          <el-button @click="movingCard = null">取消</el-button>
          <el-button type="primary" native-type="submit" :disabled="!moveTargetPath"
            :loading="actionLoading">确认移动</el-button>
        </div>
      </form>
    </div>

  </main>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import type { AllowDragFunction, AllowDropFunction, TreeInstance, TreeNodeData } from 'element-plus'
import { Close, Plus, Search } from '@element-plus/icons-vue'

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
import CardShow from '../components/anki/CardShow.vue'
import {
  aggregateCardCount,
  flattenDecks,
  getDescendantPaths,
  loadDeckTree,
  type DeckNode,
} from '../composables/useDeckTree'

const emit = defineEmits<{
  'create-card': []
  'edit-card': [card: Card]
}>()

interface DeckOption {
  value: string
  label: string
  children?: DeckOption[]
}

interface DeckTreeNode {
  path: string
  label: string
  cardCount: number
  children: DeckTreeNode[]
}

const deckTree = ref<DeckNode[]>([])
const deckTreeRef = ref<TreeInstance>()
const selectedDeckPath = ref('')
const cards = ref<Card[]>([])
const keyword = ref('')
const stateFilter = ref<CardState | ''>('')
const loading = ref(false)
const loadingCards = ref(false)
const actionLoading = ref(false)
const showCreateDeck = ref(false)
const newDeckParent = ref<string | undefined>('')
const newDeckName = ref('')
const previewCard = ref<Card | null>(null)
const movingCard = ref<Card | null>(null)
const moveTargetPath = ref('')

const allDecks = computed(() => flattenDecks(deckTree.value))
const deckCount = computed(() => allDecks.value.length)
const totalCardCount = computed(() => deckTree.value.reduce((total, node) => total + aggregateCardCount(node), 0))
const deckTreeData = computed<DeckTreeNode[]>(() => deckTree.value.map(toDeckTreeNode))
const deckOptions = computed(() => buildDeckOptions(node => node.deck.name || node.deck.path))
const moveDeckOptions = computed(() => buildDeckOptions(node => node.deck.path))

function toDeckTreeNode(node: DeckNode): DeckTreeNode {
  return {
    path: node.deck.path,
    label: node.deck.name || node.deck.path,
    cardCount: aggregateCardCount(node),
    children: node.children.map(toDeckTreeNode),
  }
}

function buildDeckOptions(labelOf: (node: DeckNode) => string, nodes: DeckNode[] = deckTree.value): DeckOption[] {
  return nodes.map(node => ({
    value: node.deck.path,
    label: labelOf(node),
    children: node.children.length > 0 ? buildDeckOptions(labelOf, node.children) : undefined,
  }))
}

watch(selectedDeckPath, async path => {
  await nextTick()
  deckTreeRef.value?.setCurrentKey(path || null)
})

async function fetchDeckTree() {
  return loadDeckTree()
}

async function reloadDecks() {
  loading.value = true

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
    ElMessage.error('牌组加载失败，请确认 Anki 服务已连接。')
  } finally {
    loading.value = false
  }

  await nextTick()
  deckTreeRef.value?.setCurrentKey(selectedDeckPath.value || null)
}

async function selectDeck(deckPath: string) {
  selectedDeckPath.value = deckPath
  keyword.value = ''
  stateFilter.value = ''
  await loadCards()
}

function handleDeckNodeClick(data: TreeNodeData) {
  void selectDeck(data.path)
}

const allowDeckDrag: AllowDragFunction = node => Boolean(node.data.path)

const allowDeckDrop: AllowDropFunction = (draggingNode, dropNode, type) => {
  const draggingPath: string = draggingNode.data.path
  const dropPath: string = dropNode.data.path

  return canMoveDeckInto(draggingPath, type === 'inner' ? dropPath : getParentPath(dropPath))
}

function handleDeckDrop(
  draggingNode: { data: TreeNodeData },
  dropNode: { data: TreeNodeData },
  dropType: 'before' | 'after' | 'inner',
) {
  void moveDeckTo(draggingNode.data.path, dropNode.data.path, dropType)
}

function getParentPath(path: string) {
  const separatorIndex = path.lastIndexOf('/')

  return separatorIndex === -1 ? '' : path.slice(0, separatorIndex)
}

function canMoveDeckInto(sourcePath: string, targetParentPath: string) {
  return sourcePath !== targetParentPath && !targetParentPath.startsWith(`${sourcePath}/`)
}

async function moveDeckTo(sourcePath: string, dropPath: string, dropType: 'before' | 'after' | 'inner') {
  if (!sourcePath || actionLoading.value) {
    return
  }

  // 情况1：/path/to/a -> /path/to/b (before/after)
  // 情况2：/path/to/a -> /path/to/b (inner)
  // 情况3：/path/to/a -> /
  const targetPath = dropType === 'inner' ? dropPath : getParentPath(dropPath)

  if (!canMoveDeckInto(sourcePath, targetPath)) {
    await reloadDecks()
    return
  }

  if (targetPath === sourcePath) {
    await reloadDecks()
    return
  }

  actionLoading.value = true

  try {
    await moveDeck(sourcePath, targetPath)
    selectedDeckPath.value = targetPath
    ElMessage.success(`牌组已移动到“${targetPath}”。`)
    await reloadDecks()
  } catch (error) {
    console.error(error)
    ElMessage.error('牌组移动失败，请重试。')
  } finally {
    actionLoading.value = false
  }
}

async function loadCards() {
  loadingCards.value = true

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
    ElMessage.error('卡片加载失败，请稍后重试。')
  } finally {
    loadingCards.value = false
  }
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
  const name = newDeckName.value.trim()
  if (!name || actionLoading.value) {
    return
  }

  const path = newDeckParent.value ? `${newDeckParent.value}/${name}` : name

  actionLoading.value = true

  try {
    await createDeck(path)
    showCreateDeck.value = false
    newDeckName.value = ''
    newDeckParent.value = ''
    selectedDeckPath.value = path
    ElMessage.success(`牌组“${path}”已创建。`)
    await reloadDecks()
  } catch (error) {
    console.error(error)
    ElMessage.error('牌组创建失败，请检查牌组路径。')
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
    ElMessage.success('牌组已删除。')
    selectedDeckPath.value = ''
    await reloadDecks()
  } catch (error) {
    console.error(error)
    ElMessage.error('牌组删除失败，请重试。')
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
      ElMessage.error('卡片详情加载失败，请重试。')
    })
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
    ElMessage.success('卡片已移动。')
    await loadCards()
  } catch (error) {
    console.error(error)
    ElMessage.error('卡片移动失败，请重试。')
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
    ElMessage.success('卡片已删除。')
    await loadCards()
  } catch (error) {
    console.error(error)
    ElMessage.error('卡片删除失败，请重试。')
  } finally {
    actionLoading.value = false
  }
}

async function resetSelectedCard(card: Card) {
  actionLoading.value = true
  try {
    await resetCard(card.id)
    ElMessage.success('卡片已重置为新卡状态。')
    await loadCards()
  } catch (error) {
    console.error(error)
    ElMessage.error('卡片重置失败，请重试。')
  } finally {
    actionLoading.value = false
  }
}

async function gradeSelectedCard(card: Card, grade: CardGrade) {
  actionLoading.value = true
  try {
    await gradeCard(card.id, grade)
    ElMessage.success('复习结果已记录。')
    await loadCards()
  } catch (error) {
    console.error(error)
    ElMessage.error('复习结果记录失败，请重试。')
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

.manager-page :deep(.el-button + .el-button) {
  margin-left: 0;
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

.deck-tree {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.deck-tree .all-cards-row {
  width: 100%;
  height: 38px;
  justify-content: flex-start;
  margin-bottom: 6px;
  padding: 0 10px;
  border-bottom: 1px solid #eeeae4;
  border-radius: 0;
  color: #4f4a43;
  font-size: 13px;
}

.deck-tree .all-cards-row :deep(.el-button__text) {
  flex: 1;
  display: flex;
  align-items: center;
  gap: 6px;
}

.deck-tree .all-cards-row.active {
  color: var(--learning-primary);
  font-weight: 650;
}

.deck-tree-inner :deep(.el-tree-node__content) {
  height: 38px;
  border-radius: 6px;
}

.deck-tree-inner :deep(.el-tree-node__content:hover) {
  background: #eaf2ff;
}

.deck-tree-inner :deep(.el-tree-node.is-current > .el-tree-node__content) {
  background: #eaf2ff;
  color: var(--learning-primary);
  font-weight: 650;
}

.deck-node {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 6px;
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

.filters {
  display: flex;
  gap: 8px;
  margin-bottom: 16px;
}

.filters .search-input {
  flex: 1;
  min-width: 0;
}

.filters .state-select {
  width: 130px;
  flex-shrink: 0;
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
  align-items: center;
  gap: 4px;
  margin-left: auto;
}

.card-actions :deep(.el-button),
.review-actions :deep(.el-button) {
  height: auto;
  padding: 0 3px;
  font-size: 12px;
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

.modal .field-label {
  margin-top: 22px;
}

.modal .deck-select,
.modal .deck-name-input {
  width: 100%;
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

  .filters .search-input {
    flex-basis: 100%;
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
.manager-page .card-deck-path {
  color: var(--learning-primary);
}

.manager-page .card-item {
  border-color: var(--learning-border);
}

.manager-page .review-actions {
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

.manager-page .modal-backdrop {
  background: rgba(15, 35, 65, 0.38);
}
</style>
