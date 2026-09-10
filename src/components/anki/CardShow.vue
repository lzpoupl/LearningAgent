<template>
  <article
    v-if="variant === 'review'"
    class="review-card"
    :class="{ revealed }"
    @click="$emit('toggle')"
  >
    <div class="card-half card-front">
      <div class="card-label">问题</div>
      <CardContent class="review-content" :content="front" />
    </div>
    <div class="card-divider">
      <span>{{ revealed ? '点击卡片隐藏答案' : '点击卡片查看答案' }}</span>
    </div>
    <div class="card-half card-back" :class="{ hidden: !revealed }">
      <div class="card-label">答案</div>
      <CardContent class="review-content" :content="back" />
    </div>
  </article>

  <div v-else-if="variant === 'preview'" class="preview-sides">
    <div class="preview-side">
      <span class="side-label">正面</span>
      <CardContent class="rendered-card-content" :content="front" />
    </div>
    <div class="preview-side preview-side--back">
      <span class="side-label">背面</span>
      <CardContent class="rendered-card-content" :content="back" />
    </div>
  </div>

  <div v-else-if="variant === 'creator'" class="creator-preview">
    <div class="preview-side">
      <span class="side-label">正面</span>
      <CardContent v-if="front.trim()" class="rendered-card-content" :content="front" />
      <p v-else class="placeholder">{{ frontPlaceholder }}</p>
    </div>
    <div class="preview-divider"><span>翻面后</span></div>
    <div class="preview-side preview-side--back">
      <span class="side-label">背面</span>
      <CardContent v-if="back.trim()" class="rendered-card-content" :content="back" />
      <p v-else class="placeholder">{{ backPlaceholder }}</p>
    </div>
  </div>

  <div v-else class="card-sides">
    <div class="card-side">
      <span class="side-label">正面</span>
      <CardContent class="card-rendered-content" :content="front" />
    </div>
    <div class="card-side card-side--back">
      <span class="side-label">背面</span>
      <CardContent class="card-rendered-content" :content="back" />
    </div>
  </div>
</template>

<script setup lang="ts">
import CardContent from './CardContent.vue'

defineProps<{
  front: string
  back: string
  variant?: 'manager' | 'preview' | 'review' | 'creator'
  revealed?: boolean
  frontPlaceholder?: string
  backPlaceholder?: string
}>()

defineEmits<{
  toggle: []
}>()
</script>

<style scoped>
/* manager（列表项） */
.card-sides {
  display: grid;
  grid-template-columns: 1fr 1fr;
}

.card-side {
  min-width: 0;
  padding: 16px;
}

.card-side--back {
  border-left: 1px solid #eeeae4;
  background: var(--learning-surface-muted);
}

.side-label {
  color: var(--learning-primary);
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.1em;
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

/* preview（预览弹窗） */
.preview-side {
  margin-top: 18px;
  padding: 18px;
  border: 1px solid var(--learning-border);
  border-radius: 6px;
  background: var(--learning-surface-muted);
}

.preview-side--back {
  background: var(--learning-surface-muted);
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

/* creator（编辑页实时预览） */
.creator-preview {
  margin-top: 26px;
}

.creator-preview .preview-side {
  min-height: 180px;
  margin-top: 0;
}

.creator-preview .preview-side--back {
  min-height: 210px;
}

.preview-divider {
  position: relative;
  display: flex;
  justify-content: center;
  height: 34px;
  color: var(--learning-text-muted);
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.preview-divider::before {
  position: absolute;
  top: 16px;
  right: 0;
  left: 0;
  height: 1px;
  background: #d9e7f8;
  content: '';
}

.preview-divider span {
  z-index: 1;
  padding: 8px 10px;
  background: var(--learning-surface);
}

.placeholder {
  margin: 18px 0 0;
  color: var(--learning-text-secondary);
  font-size: 18px;
}

/* review（复习页） */
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
  border: 1px solid var(--learning-border);
  border-radius: 10px;
  background: var(--learning-surface);
  box-shadow: 0 16px 42px rgba(49, 41, 32, 0.1);
  text-align: center;
}

.review-card:focus-visible {
  outline: 3px solid rgba(40, 125, 245, 0.25);
  outline-offset: 2px;
}

.card-label {
  color: var(--learning-primary);
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.14em;
}

.review-content {
  width: 100%;
  margin-top: 18px;
  color: #49443d;
  font-family: Georgia, 'Times New Roman', serif;
  font-size: clamp(18px, 2.6vw, 29px);
  line-height: 1.55;
}

.review-content :deep(p) {
  margin: 0 0 12px;
}

.review-content :deep(ul),
.review-content :deep(ol) {
  text-align: left;
}

.review-content :deep(img) {
  max-width: 100%;
  height: auto;
}

.review-content :deep(.katex-display) {
  overflow-x: auto;
  margin: 20px 0;
}

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
  background: var(--learning-surface);
}

.card-back {
  background: var(--learning-surface-muted);
}

.card-divider {
  position: relative;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--learning-surface);
  color: #9a948b;
  font-size: 10px;
  letter-spacing: 0.1em;
  text-transform: uppercase;
}

.card-divider::before {
  position: absolute;
  right: 32px;
  left: 32px;
  height: 1px;
  background: #eeeae4;
  content: '';
}

.card-divider span {
  z-index: 1;
  padding: 5px 10px;
  background: var(--learning-surface);
}

.review-card.revealed .card-half {
  min-height: 0;
}

.card-back.hidden .card-label,
.card-back.hidden .review-content {
  visibility: hidden;
}

@media (max-width: 620px) {
  .card-sides {
    grid-template-columns: 1fr;
  }

  .card-side--back {
    border-top: 1px solid #eeeae4;
    border-left: 0;
  }
}
</style>
