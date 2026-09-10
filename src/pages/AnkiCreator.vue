<template>
  <main class="anki-page">
    <header class="anki-header">
      <div>
        <div class="eyebrow">学习资产 / ANKI</div>
        <h1>{{ isEditing ? '编辑卡片' : '制作一张可复习的卡片' }}</h1>
        <p>{{ isEditing ? '修改卡片内容，保留已有的复习进度。' : '把刚刚理解的知识点压缩成一次清晰的记忆提取。' }}</p>
      </div>
      <button class="refresh-button" type="button" @click="$emit('close')">
        <span aria-hidden="true">←</span>
        返回卡片管理
      </button>
    </header>

    <section class="anki-layout">
      <form class="editor-panel" @submit.prevent="saveCard">
        <div class="panel-heading">
          <div>
            <span class="panel-kicker">CARD BUILDER</span>
            <h2>{{ isEditing ? '编辑内容' : '卡片内容' }}</h2>
          </div>
        </div>

        <label class="field-label" for="deck">目标牌组</label>
        <div class="deck-row">
          <select id="deck" v-model="selectedDeckPath"
            :disabled="isEditing || loadingDecks || saving || decks.length === 0">
            <option value="" disabled>{{ loadingDecks ? '正在加载牌组...' : '选择一个牌组' }}</option>
            <option v-for="deck in decks" :key="deck.path" :value="deck.path">
              {{ deck.path }} · {{ deck.cardCount }} 张
            </option>
          </select>
        </div>


        <div class="field-group">
          <div class="field-header">
            <label class="field-label" for="front">正面 · 先回忆，再翻面</label>
            <button class="image-button" type="button" :disabled="saving || imageUploading" title="插入图片"
              @click="openImagePicker('front')">
              ▧ 插入图片
            </button>
          </div>
          <textarea ref="frontTextarea" id="front" v-model="front" rows="6" placeholder="写一个能独立理解的问题，不要提前泄露答案。" />
          <input ref="frontImageInput" class="hidden-file-input" type="file" accept="image/*"
            @change="handleImageSelected('front', $event)" />
        </div>

        <div class="field-group">
          <div class="field-header">
            <label class="field-label" for="back">背面 · 答案与关键解释</label>
            <button class="image-button" type="button" :disabled="saving || imageUploading" title="插入图片"
              @click="openImagePicker('back')">
              ▧ 插入图片
            </button>
          </div>
          <textarea ref="backTextarea" id="back" v-model="back" rows="8" placeholder="先写标准答案，再补充必要的解释、条件或例子。" />
          <input ref="backImageInput" class="hidden-file-input" type="file" accept="image/*"
            @change="handleImageSelected('back', $event)" />
        </div>

        <div v-if="errorMessage" class="feedback error" role="alert">
          {{ errorMessage }}
        </div>
        <div v-if="successMessage" class="feedback success" role="status">
          {{ successMessage }}
        </div>

        <button class="save-button" type="submit" :disabled="!canSave || saving || imageUploading">
          <span>{{ saving ? '正在保存...' : '保存到 Anki' }}</span>
          <span aria-hidden="true">→</span>
        </button>
      </form>

      <aside class="preview-panel">
        <div class="preview-topline">
          <span class="panel-kicker">LIVE PREVIEW</span>
          <span class="preview-chip">{{ selectedDeckPath || '未选择牌组' }}</span>
        </div>
        <CardShow
          variant="creator"
          :front="front"
          :back="back"
          front-placeholder="你的问题会显示在这里"
          back-placeholder="答案与解释会显示在这里"
        />
        <div class="preview-note">
          <span class="note-mark">✦</span>
          <span>一张卡片只测试一个主要记忆点。复杂内容建议拆成多张卡片。</span>
        </div>
      </aside>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from 'vue'

import { createCard, updateCardContent, uploadImage } from '../services/anki'
import type { Card, Deck } from '../types/anki'
import CardShow from '../components/anki/CardShow.vue'
import { flattenDecks, loadDeckTree } from '../composables/useDeckTree'

const props = defineProps<{
  editingCard?: Card | null
}>()

defineEmits<{
  close: []
}>()

const decks = ref<Deck[]>([])
const selectedDeckPath = ref(props.editingCard?.deckPath || '')
const front = ref(props.editingCard?.front || '')
const back = ref(props.editingCard?.back || '')
const loadingDecks = ref(false)
const saving = ref(false)
const imageUploading = ref(false)
const errorMessage = ref('')
const successMessage = ref('')
const frontTextarea = ref<HTMLTextAreaElement | null>(null)
const backTextarea = ref<HTMLTextAreaElement | null>(null)
const frontImageInput = ref<HTMLInputElement | null>(null)
const backImageInput = ref<HTMLInputElement | null>(null)


const isEditing = computed(() => Boolean(props.editingCard))
const canSave = computed(() => Boolean(selectedDeckPath.value && front.value.trim() && back.value.trim()))
function openImagePicker(field: 'front' | 'back') {
  const input = field === 'front' ? frontImageInput.value : backImageInput.value
  input?.click()
}

function handleImageSelected(field: 'front' | 'back', event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]

  if (!file) {
    return
  }

  input.value = ''

  if (!file.type.startsWith('image/')) {
    errorMessage.value = '只能插入图片文件。'
    return
  }

  if (file.size > 5 * 1024 * 1024) {
    errorMessage.value = '图片不能超过 5MB。'
    return
  }

  const reader = new FileReader()

  reader.onload = async () => {
    if (typeof reader.result !== 'string') {
      errorMessage.value = '图片读取失败，请重试。'
      return
    }

    const separatorIndex = reader.result.indexOf(',')
    const contentBase64 = separatorIndex === -1
      ? reader.result
      : reader.result.slice(separatorIndex + 1)
    imageUploading.value = true

    try {
      const uploadedImage = await uploadImage({
        name: file.name,
        mimeType: file.type,
        contentBase64,
      })
      const textarea = field === 'front' ? frontTextarea.value : backTextarea.value
      const imageMarkdown = `![${file.name}](${uploadedImage.url})`
      const currentContent = field === 'front' ? front.value : back.value
      const start = textarea?.selectionStart ?? currentContent.length
      const end = textarea?.selectionEnd ?? currentContent.length
      const prefix = start > 0 && !currentContent[start - 1].match(/\s/) ? '\n' : ''
      const suffix = end < currentContent.length && !currentContent[end].match(/\s/) ? '\n' : ''
      const insertedContent = `${currentContent.slice(0, start)}${prefix}${imageMarkdown}${suffix}${currentContent.slice(end)}`

      if (field === 'front') {
        front.value = insertedContent
      } else {
        back.value = insertedContent
      }

      errorMessage.value = ''
      await nextTick()

      const cursor = start + prefix.length + imageMarkdown.length + suffix.length
      textarea?.focus()
      textarea?.setSelectionRange(cursor, cursor)
    } catch (error) {
      console.error(error)
      errorMessage.value = '图片上传失败，请重试。'
    } finally {
      imageUploading.value = false
    }
  }

  reader.onerror = () => {
    imageUploading.value = false
    errorMessage.value = '图片读取失败，请重试。'
  }

  reader.readAsDataURL(file)
}

async function loadDecks() {
  loadingDecks.value = true
  errorMessage.value = ''

  try {
    const deckTree = await loadDeckTree()
    decks.value = flattenDecks(deckTree)

    if (decks.value.length > 0) {
      if (!selectedDeckPath.value || !decks.value.some(deck => deck.path === selectedDeckPath.value)) {
        selectedDeckPath.value = decks.value[0].path
      }
    }

  } catch (error) {
    console.error(error)
    errorMessage.value = '牌组加载或默认牌组创建失败，请确认 Anki 服务已连接。'
  } finally {
    loadingDecks.value = false
  }
}

async function saveCard() {
  if (!canSave.value || saving.value) {
    return
  }

  saving.value = true
  errorMessage.value = ''
  successMessage.value = ''

  try {
    if (props.editingCard) {
      await updateCardContent(props.editingCard.id, {
        front: front.value.trim(),
        back: back.value.trim(),
      })
      successMessage.value = '卡片内容已更新。'
    } else {
      await createCard({
        deckPath: selectedDeckPath.value,
        front: front.value.trim(),
        back: back.value.trim(),
      })
      front.value = ''
      back.value = ''
      successMessage.value = '卡片已保存，之后可以在复习页开始学习。'
    }
  } catch (error) {
    console.error(error)
    errorMessage.value = '卡片保存失败，请检查牌组和卡片内容。'
  } finally {
    saving.value = false
  }
}

onMounted(loadDecks)
</script>

<style scoped>
.anki-page {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
  padding: 52px clamp(24px, 6vw, 88px) 72px;
  background: #f8f8f6;
  color: #202020;
}

.anki-header,
.anki-layout {
  width: min(1120px, 100%);
  margin: 0 auto;
}

.anki-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 34px;
}

.eyebrow,
.panel-kicker {
  color: #8a735a;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.12em;
}

h1,
h2,
p {
  margin: 0;
}

h1 {
  margin-top: 10px;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(30px, 4vw, 48px);
  font-weight: 500;
  letter-spacing: 0;
  line-height: 1.08;
}

.anki-header p {
  margin-top: 12px;
  color: #77736d;
  font-size: 14px;
}

.refresh-button,
.secondary-button,
.save-button {
  border: 0;
  cursor: pointer;
  font-size: 13px;
}

.refresh-button {
  display: inline-flex;
  align-items: center;
  gap: 7px;
  padding: 9px 12px;
  border: 1px solid #d9d5ce;
  border-radius: 8px;
  background: #fff;
  color: #45423d;
}

.refresh-button:disabled,
.secondary-button:disabled,
.icon-button:disabled,
.save-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.anki-layout {
  display: grid;
  grid-template-columns: minmax(0, 1.08fr) minmax(320px, 0.92fr);
  gap: 22px;
  align-items: start;
}

.editor-panel,
.preview-panel {
  border: 1px solid #e5e1da;
  border-radius: 8px;
  background: #fff;
}

.editor-panel {
  padding: clamp(22px, 4vw, 38px);
}

.panel-heading,
.preview-topline,
.deck-row {
  display: flex;
  align-items: center;
}

.panel-heading,
.preview-topline {
  justify-content: space-between;
  gap: 14px;
}

h2 {
  margin-top: 7px;
  font-size: 21px;
  font-weight: 650;
}

.field-group {
  position: relative;
  margin-top: 25px;
}

.field-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.field-label {
  display: block;
  margin: 0 0 8px;
  color: #4c4944;
  font-size: 12px;
  font-weight: 650;
}

.image-button {
  flex-shrink: 0;
  margin-bottom: 8px;
  padding: 4px 7px;
  border: 1px solid #dedbd5;
  border-radius: 5px;
  background: #fff;
  color: #6b6259;
  cursor: pointer;
  font-size: 11px;
}

.image-button:hover {
  border-color: #b8a38b;
  background: #faf7f2;
  color: #4f4438;
}

.image-button:disabled {
  cursor: not-allowed;
  opacity: 0.55;
}

.hidden-file-input {
  display: none;
}

select,
input,
textarea {
  width: 100%;
  border: 1px solid #dedbd5;
  border-radius: 6px;
  outline: none;
  background: #fcfcfb;
  color: #262522;
  font-size: 14px;
}

select,
input {
  height: 40px;
  padding: 0 12px;
}

textarea {
  display: block;
  min-height: 118px;
  padding: 12px;
  resize: vertical;
  line-height: 1.65;
}

select:focus,
input:focus,
textarea:focus {
  border-color: #a99072;
  box-shadow: 0 0 0 3px rgba(169, 144, 114, 0.12);
}

.deck-row {
  gap: 8px;
}

.icon-button {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border: 1px solid #dedbd5;
  border-radius: 6px;
  background: #fff;
  color: #5f554a;
  font-size: 20px;
}

.create-deck-row {
  gap: 8px;
  margin-top: 10px;
}

.secondary-button {
  height: 40px;
  flex-shrink: 0;
  padding: 0 14px;
  border-radius: 6px;
  background: #eae4dc;
  color: #574a3e;
}

.field-hint {
  position: absolute;
  right: 10px;
  bottom: 10px;
  color: #aaa49b;
  font-size: 11px;
}

.feedback {
  margin-top: 18px;
  padding: 10px 12px;
  border-radius: 6px;
  font-size: 12px;
  line-height: 1.5;
}

.feedback.error {
  background: #fff1ee;
  color: #a34e3f;
}

.feedback.success {
  background: #eef7ef;
  color: #4d7652;
}

.save-button {
  display: flex;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  height: 46px;
  margin-top: 24px;
  padding: 0 16px;
  border-radius: 6px;
  background: #25231f;
  color: #fff;
  font-weight: 600;
}

.preview-panel {
  padding: 22px;
  background: #272521;
  color: #f9f4eb;
  box-shadow: 0 18px 40px rgba(49, 41, 32, 0.12);
}

.preview-chip {
  max-width: 52%;
  overflow: hidden;
  color: #b9a993;
  font-size: 11px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preview-note {
  display: flex;
  gap: 9px;
  margin-top: 20px;
  color: #aaa196;
  font-size: 12px;
  line-height: 1.55;
}

.note-mark {
  color: #d0a879;
}

@media (max-width: 820px) {
  .anki-page {
    padding: 30px 18px 48px;
  }

  .anki-header {
    align-items: flex-start;
    flex-direction: column;
    margin-bottom: 24px;
  }

  .anki-layout {
    grid-template-columns: 1fr;
  }
}

.anki-page {
  background: var(--learning-bg);
  color: var(--learning-text);
}

.anki-page .eyebrow,
.anki-page .panel-kicker {
  color: var(--learning-primary);
}

.anki-page .anki-header p {
  color: var(--learning-text-secondary);
}

.anki-page .refresh-button,
.anki-page .icon-button,
.anki-page .image-button {
  border-color: var(--learning-border);
  background: var(--learning-surface);
  color: var(--learning-text-secondary);
}

.anki-page .refresh-button:hover,
.anki-page .icon-button:hover,
.anki-page .image-button:hover {
  border-color: #b7d0f8;
  background: #eef5ff;
  color: var(--learning-primary);
}

.anki-page .editor-panel,
.anki-page .preview-panel {
  border-color: var(--learning-border);
}

.anki-page .field-label {
  color: var(--learning-text);
}

.anki-page select,
.anki-page input,
.anki-page textarea {
  border-color: var(--learning-border);
  background: var(--learning-surface);
  color: var(--learning-text);
}

.anki-page select:focus,
.anki-page input:focus,
.anki-page textarea:focus {
  border-color: var(--learning-primary);
  box-shadow: 0 0 0 3px rgba(40, 125, 245, 0.14);
}

.anki-page .secondary-button {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.anki-page .field-hint {
  color: var(--learning-text-muted);
}

.anki-page .feedback.error {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.anki-page .feedback.success {
  background: var(--el-color-success-light-9);
  color: var(--el-color-success);
}

.anki-page .save-button {
  background: var(--learning-primary);
}

.anki-page .preview-panel {
  background: var(--learning-surface);
  color: var(--learning-text);
  box-shadow: var(--learning-shadow);
}

.anki-page .preview-chip {
  color: var(--learning-primary);
}

.anki-page .preview-note {
  color: var(--learning-text-secondary);
}

.anki-page .note-mark {
  color: #8db7f4;
}
</style>
