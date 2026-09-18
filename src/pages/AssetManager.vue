<template>
  <main class="materials-page">
    <header class="materials-header">
      <div>
        <div class="eyebrow">学习资产 / MATERIALS</div>
        <h1>学习资料</h1>
        <p>PDF、PPT、笔记等非结构化学习资产</p>
      </div>
      <el-button type="primary" :loading="addingDirectory" @click="addDirectory">
        <el-icon>
          <FolderAdd />
        </el-icon>
        添加目录
      </el-button>
    </header>

    <section class="materials-toolbar">
      <el-popover v-model:visible="bucketMenuOpen" class="bucket-picker-popover" placement="bottom-start" :width="270"
        trigger="click">
        <template #reference>
          <el-button class="bucket-picker" plain>
            <span class="bucket-picker-label">{{ selectedBucketLabel }}</span>
            <el-icon>
              <ArrowDown />
            </el-icon>
          </el-button>
        </template>
        <el-tree class="bucket-tree" :data="bucketTreeData" node-key="value" :props="bucketTreeProps"
          highlight-current :current-node-key="selectedBucket" @node-click="handleBucketNodeClick">
          <template #default="{ data }">
            <span class="bucket-tree-node" :title="data.folder ? `物理存储：${data.folder}` : '全部资料目录'">
              <span class="bucket-tree-label">{{ data.label }}</span>
              <span class="bucket-tree-count">{{ data.count }}</span>
            </span>
          </template>
        </el-tree>
      </el-popover>

      <!-- 搜索框：位于排序左侧 -->
      <el-input v-model="searchKeyword" class="material-search" clearable placeholder="搜索资料名称" aria-label="搜索资料">
        <template #prefix>
          <el-icon>
            <Search />
          </el-icon>
        </template>
      </el-input>

      <div class="sort-control">
        <span>排序</span>
        <el-select v-model="sortBy" class="sort-select" aria-label="资料排序" size="small">
          <el-option label="最近添加" value="updated" />
          <el-option label="名称" value="name" />
          <el-option label="文件大小" value="size" />
        </el-select>
      </div>
    </section>

    <section v-if="filteredMaterials.length" class="materials-grid" aria-label="资料列表">
      <el-card v-for="material in filteredMaterials" :key="material.id" class="material-card" shadow="hover"
        :body-style="{ padding: '0' }">
        <div class="material-cover" :class="`cover-${material.kind}`">
          <span class="file-mark">{{ material.extension }}</span>
          <span class="file-type">{{ material.typeLabel }}</span>
        </div>
        <div class="material-body">
          <div class="material-bucket" :class="`format-accent-${material.kind}`" :title="`物理存储：${material.folder}`">
            {{ material.bucket }}
          </div>
          <h2 :title="material.name">{{ material.name }}</h2>
          <div class="material-meta">
            <span>{{ formatSize(material.size) }}</span>
            <span>添加于 {{ formatAddedAt(material.addedAt) }}</span>
          </div>
          <div class="material-actions">
            <el-button class="material-open-button" :class="`format-accent-${material.kind}`" type="primary" text
              size="small" @click="openMaterial(material)">
              <el-icon>
                <FolderOpened />
              </el-icon>
              打开
            </el-button>
            <el-button class="material-remove-button" text type="danger" size="small" title="移除资料"
              @click="removeMaterial(material.id)">
              <el-icon>
                <Delete />
              </el-icon>
              删除
            </el-button>
          </div>
        </div>
      </el-card>
    </section>

    <section v-else class="empty-materials">
      <el-empty :description="emptyDescription">
        <p>{{ emptyHint }}</p>
        <el-button type="primary" :loading="addingDirectory" @click="addDirectory">选择目录</el-button>
      </el-empty>
    </section>
  </main>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { ArrowDown, FolderAdd, Search } from '@element-plus/icons-vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

import type { Bucket, LearningAsset } from '../types/assets'
import { createBucket, deleteAsset, getAssetUrl, listAssets, listBuckets } from '../services/assets'

type MaterialKind = 'pdf' | 'slides' | 'note' | 'image' | 'word' | 'document'

type Material = {
  id: string
  name: string
  extension: string
  typeLabel: string
  kind: MaterialKind
  /** 资料所属的 bucket（即资料目录）名称。 */
  bucket: string
  /** 该资料目录在文件系统中的绝对路径。 */
  folder: string
  size: number
  /** RFC3339 时间戳，展示时格式化，排序时直接比较。 */
  addedAt: string
  /** mock 或已解析出的可访问地址；缺省时按需向服务端解析。 */
  url?: string
}

/** 表示“全部资料目录”的哨兵值。 */
const ALL_BUCKETS = 'ALL_BUCKETS'

const buckets = ref<Bucket[]>([])
const materials = ref<Material[]>([])

const selectedBucket = ref(ALL_BUCKETS)
const bucketMenuOpen = ref(false)
const addingDirectory = ref(false)
const sortBy = ref<'updated' | 'name' | 'size'>('updated')
const searchKeyword = ref('')

const bucketNames = computed(() => [ALL_BUCKETS, ...buckets.value.map(item => item.name)])

const selectedBucketLabel = computed(() =>
  selectedBucket.value === ALL_BUCKETS ? '全部资料' : `${selectedBucket.value}资料`)

const bucketTreeData = computed(() => bucketNames.value.map(name => ({
  label: name === ALL_BUCKETS ? '全部资料' : name,
  value: name,
  count: bucketCount(name),
  folder: name === ALL_BUCKETS ? '' : folderOf(name),
})))

const bucketTreeProps = { label: 'label', children: 'children' }

const emptyDescription = computed(() => {
  if (!materials.value.length) return '还没有资料目录'
  if (searchKeyword.value.trim()) return '没有匹配的资料'
  return '这个资料目录还没有内容'
})

const emptyHint = computed(() => {
  if (!materials.value.length) return '选择一个本地目录，目录内的文件会作为学习资料展示。'
  if (searchKeyword.value.trim()) return '调整搜索关键词，再试一次。'
  return '切换其他资料目录查看。'
})

const filteredMaterials = computed(() => {
  const bucketFiltered = selectedBucket.value === ALL_BUCKETS
    ? materials.value
    : materials.value.filter(material => material.bucket === selectedBucket.value)
  const normalizedKeyword = searchKeyword.value.trim().toLocaleLowerCase()
  const filtered = bucketFiltered.filter(material => {
    const searchableText = [material.name, material.extension, material.typeLabel, material.bucket]
      .join(' ')
      .toLocaleLowerCase()
    return !normalizedKeyword || searchableText.includes(normalizedKeyword)
  })

  return [...filtered].sort((left, right) => {
    if (sortBy.value === 'name') {
      return left.name.localeCompare(right.name, 'zh-CN')
    }
    if (sortBy.value === 'size') {
      return right.size - left.size
    }
    return right.addedAt.localeCompare(left.addedAt)
  })
})

function bucketCount(name: string) {
  return name === ALL_BUCKETS
    ? materials.value.length
    : materials.value.filter(material => material.bucket === name).length
}

function folderOf(name: string) {
  return buckets.value.find(item => item.name === name)?.rootPath ?? ''
}

/** 添加目录：选择本地目录后登记为 bucket，目录内文件即成为资料。 */
async function addDirectory() {
  addingDirectory.value = true
  try {
    const selected = await open({ directory: true, multiple: false, title: '选择资料目录' })
    if (typeof selected !== 'string' || !selected) {
      return
    }

    const name = await resolveBucketName(defaultNameOf(selected))
    if (!name) {
      return
    }

    await createBucket(name, selected)
    await refresh()
    selectedBucket.value = name
    bucketMenuOpen.value = false
    ElMessage.success(`已添加资料目录“${name}”`)
  } catch (error) {
    console.error('添加资料目录失败：', error)
    ElMessage.error(errorMessage(error, '添加资料目录失败，请稍后重试。'))
  } finally {
    addingDirectory.value = false
  }
}

/** 默认用目录名作为资料目录名称，重名时再让用户改一个。 */
async function resolveBucketName(baseName: string): Promise<string | null> {
  if (!buckets.value.some(item => item.name === baseName)) {
    return baseName
  }
  return askBucketName(`已存在名为“${baseName}”的资料目录，请换一个名称`, baseName)
}

async function askBucketName(message: string, defaultName: string): Promise<string | null> {
  try {
    const { value } = await ElMessageBox.prompt(message, '添加资料目录', {
      confirmButtonText: '添加',
      cancelButtonText: '取消',
      inputValue: defaultName,
      inputPlaceholder: '例如：操作系统',
      inputValidator: (input: string) => (input && input.trim() ? true : '名称不能为空'),
    })
    return value.trim()
  } catch {
    return null
  }
}

function defaultNameOf(path: string) {
  const parts = path.split(/[\\/]/).filter(Boolean)
  return parts[parts.length - 1] ?? '资料'
}

async function refresh() {
  buckets.value = await listBuckets()
  materials.value = (await listAssets()).map(toMaterial)
}

/** 把服务端资产映射为页面资料；所属目录与物理路径由资产 id 的首段还原。 */
function toMaterial(asset: LearningAsset): Material {
  const { bucket } = parseAssetId(asset.id)
  return {
    id: asset.id,
    name: asset.name,
    extension: asset.extension ? asset.extension.toUpperCase() : 'FILE',
    typeLabel: asset.typeLabel,
    kind: getMaterialKind(asset.name),
    bucket: bucket || '未分组',
    folder: folderOf(bucket),
    size: asset.size,
    addedAt: asset.addedAt,
    url: asset.url,
  }
}

/** 资产 id 形如 `/<bucket>/<relative-path>`。 */
function parseAssetId(id: string) {
  const trimmed = id.replace(/^\/+/, '')
  const separator = trimmed.indexOf('/')
  return separator === -1
    ? { bucket: trimmed, relativePath: '' }
    : { bucket: trimmed.slice(0, separator), relativePath: trimmed.slice(separator + 1) }
}

function errorMessage(error: unknown, fallback: string) {
  if (error && typeof error === 'object' && 'message' in error) {
    const message = (error as { message?: unknown }).message
    if (typeof message === 'string' && message) return message
  }
  if (typeof error === 'string' && error) return error
  if (error instanceof Error && error.message) return error.message
  return fallback
}

function handleBucketNodeClick(data: { value: string }) {
  selectedBucket.value = data.value
  bucketMenuOpen.value = false
}

async function openMaterial(material: Material) {
  try {
    const path = await getAssetUrl(material.id)
    const url = toAccessibleUrl(path || material.url || '')
    if (!url) {
      ElMessage.error('该资料暂时无法打开。')
      return
    }
    window.open(url, '_blank', 'noopener,noreferrer')
  } catch (error) {
    console.error('打开资料失败：', error)
    ElMessage.error('打开资料失败，请稍后重试。')
  }
}

async function removeMaterial(id: string) {
  try {
    await deleteAsset(id)
    materials.value = materials.value.filter(item => item.id !== id)
  } catch (error) {
    console.error('删除资料失败：', error)
    ElMessage.error('删除资料失败，请稍后重试。')
  }
}

/** 后端返回绝对路径，用 convertFileSrc 转成 WebView 可访问地址；mock 已给出 data URL 时直接使用。 */
function toAccessibleUrl(raw: string) {
  if (!raw) {
    return ''
  }
  if (/^(data:|blob:|https?:|file:)/i.test(raw)) {
    return raw
  }
  try {
    return convertFileSrc(raw)
  } catch {
    return raw
  }
}

function getExtension(name: string) {
  return name.split('.').pop()?.toUpperCase() || 'FILE'
}

function getMaterialKind(name: string): MaterialKind {
  const extension = getExtension(name)
  if (extension === 'PDF') return 'pdf'
  if (extension === 'PPT' || extension === 'PPTX') return 'slides'
  if (['MD', 'MARKDOWN', 'TXT'].includes(extension)) return 'note'
  if (['PNG', 'JPG', 'JPEG', 'WEBP'].includes(extension)) return 'image'
  if (extension === 'DOC' || extension === 'DOCX') return 'word'
  return 'document'
}

function formatSize(size: number) {
  if (size < 1024 * 1024) {
    return `${Math.max(1, Math.round(size / 1024))} KB`
  }
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

function formatAddedAt(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) {
    return value
  }
  return date.toLocaleDateString('zh-CN', { year: 'numeric', month: '2-digit', day: '2-digit' })
}

onMounted(async () => {
  try {
    await refresh()
  } catch (error) {
    console.error('加载学习资料失败：', error)
    ElMessage.error('加载学习资料失败，请稍后重试。')
  }
})
</script>

<style scoped>
.materials-page {
  flex: 1;
  min-width: 0;
  height: 100vh;
  overflow-y: auto;
  padding: 42px clamp(20px, 5vw, 70px) 60px;
  background: #f8f8f6;
  color: #292723;
}

.materials-header,
.materials-toolbar,
.materials-grid,
.empty-materials {
  width: min(1220px, 100%);
  margin: 0 auto;
}

.materials-header {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 24px;
  margin-bottom: 26px;
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

.materials-header p {
  margin-top: 10px;
  color: #77736d;
  font-size: 14px;
}

button,
input,
select {
  font: inherit;
}

button {
  cursor: pointer;
}

.materials-toolbar {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e1da;
}

.bucket-picker {
  flex: 0 0 220px;
  width: 220px;
  justify-content: space-between;
  border-color: var(--learning-border);
  color: var(--learning-text-secondary);
  text-align: left;
}

.bucket-picker:hover,
.bucket-picker:focus {
  border-color: #b7d0f8;
  background: #eef5ff;
  color: var(--learning-primary);
}

.bucket-picker-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bucket-tree {
  margin: -6px;
  color: var(--learning-text);
}

.bucket-tree :deep(.el-tree-node__content) {
  height: 38px;
  border-radius: 6px;
}

.bucket-tree :deep(.el-tree-node__content:hover),
.bucket-tree :deep(.is-current > .el-tree-node__content) {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.bucket-tree-node {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-right: 8px;
  font-size: 13px;
}

.bucket-tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.bucket-tree-count {
  flex-shrink: 0;
  color: var(--learning-text-muted);
  font-size: 11px;
}

/* 搜索框：紧跟资料目录选择器，位于排序左侧 */
.material-search {
  flex: 0 0 260px;
  width: 260px;
}

.sort-control {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 8px;
  margin-left: auto;
  color: #9a9289;
  font-size: 12px;
}

.sort-select {
  width: 150px;
}

.materials-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 14px;
}

.material-card {
  overflow: hidden;
  border: 1px solid #e5e1da;
  border-radius: 8px;
  background: #fff;
  transition: transform 160ms ease, box-shadow 160ms ease;
}

.material-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 24px rgba(61, 49, 36, 0.08);
}

.material-cover {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  height: 116px;
  padding: 16px;
  background: #f1ebe3;
  color: #765d45;
}

.cover-pdf {
  background: #f3e7df;
  color: #9d604b;
}

.cover-slides {
  background: #f1eadc;
  color: #967345;
}

.cover-note {
  background: #e9eee8;
  color: #647b67;
}

.cover-image {
  background: #e7ecee;
  color: #5f7680;
}

.cover-document {
  background: #ece9e4;
  color: #73695e;
}

.file-mark {
  font-family: Georgia, 'Times New Roman', serif;
  font-size: 30px;
  font-weight: 700;
}

.file-type {
  font-size: 11px;
}

.material-body {
  padding: 15px 16px 14px;
}

.material-bucket {
  color: #a08c75;
  font-size: 10px;
  font-weight: 700;
  letter-spacing: 0.08em;
}

.material-body h2 {
  overflow: hidden;
  margin-top: 8px;
  color: #4d463e;
  font-size: 15px;
  font-weight: 650;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.material-meta {
  display: flex;
  justify-content: space-between;
  gap: 8px;
  margin-top: 12px;
  color: #a29a91;
  font-size: 11px;
}

.material-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 15px;
}

.empty-materials {
  display: flex;
  align-items: center;
  flex-direction: column;
  justify-content: center;
  min-height: 390px;
  border: 1px dashed #d8d0c5;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.42);
  text-align: center;
}

@media (max-width: 1024px) {
  .sort-control {
    margin-left: 0;
  }
}

@media (max-width: 900px) {
  .materials-grid {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 620px) {
  .materials-page {
    padding: 28px 16px 48px;
  }

  .materials-header {
    align-items: flex-start;
    flex-direction: column;
  }

  .materials-toolbar {
    align-items: stretch;
    flex-direction: column;
  }

  .bucket-picker,
  .material-search {
    flex: 1 1 auto;
    width: 100%;
    max-width: none;
  }

  .sort-control {
    width: 100%;
    margin-left: 0;
  }

  .sort-select {
    flex: 1 1 auto;
    width: auto;
  }

  .materials-grid {
    grid-template-columns: 1fr;
  }
}

.materials-page {
  background: var(--learning-bg);
  color: var(--learning-text);
}

.materials-page .eyebrow {
  color: var(--learning-primary);
}

.materials-page .materials-header p,
.materials-page .sort-control,
.materials-page .material-meta,
.materials-page .empty-materials p {
  color: var(--learning-text-secondary);
}

.materials-page .materials-toolbar {
  border-bottom-color: var(--learning-border);
}

.materials-page .bucket-picker:hover,
.materials-page .bucket-picker:focus {
  border-color: var(--el-color-primary-light-5);
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.materials-page .bucket-tree :deep(.el-tree-node__content:hover),
.materials-page .bucket-tree :deep(.is-current > .el-tree-node__content) {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.materials-page .material-card {
  border-color: var(--learning-border);
  background: var(--learning-surface);
  box-shadow: var(--learning-shadow);
}

.materials-page .material-card:hover {
  box-shadow: 0 10px 24px rgba(40, 125, 245, 0.14);
}

.materials-page .material-cover {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.materials-page .cover-pdf {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.materials-page .cover-slides {
  background: var(--el-color-warning-light-9);
  color: var(--el-color-warning);
}

.materials-page .cover-word {
  background: var(--el-color-primary-light-9);
  color: var(--el-color-primary);
}

.materials-page .cover-note {
  background: var(--el-color-success-light-9);
  color: var(--el-color-success);
}

.materials-page .cover-image {
  background: var(--el-color-info-light-9);
  color: var(--el-color-info);
}

.materials-page .cover-document {
  background: var(--el-color-info-light-9);
  color: var(--el-color-info);
}

.materials-page .material-bucket {
  color: var(--learning-primary);
}

.materials-page .material-body h2 {
  color: var(--learning-text);
}

.materials-page .empty-materials {
  border-color: var(--learning-border);
  background: var(--learning-surface-soft);
}

.materials-page .material-actions {
  gap: 10px;
  margin-top: 16px;
}

.materials-page .material-open-button {
  min-width: 74px;
  border-color: transparent;
  background: transparent;
  color: var(--learning-primary);
  font-weight: 600;
}

.materials-page .material-open-button:hover {
  background: var(--el-color-primary-light-9);
  color: var(--learning-primary);
}

.materials-page .material-remove-button {
  padding-right: 4px;
  padding-left: 4px;
  font-weight: 500;
}

.materials-page .material-remove-button:hover {
  background: var(--el-color-danger-light-9);
}

.materials-page .material-open-button:focus-visible,
.materials-page .material-remove-button:focus-visible {
  outline: 3px solid rgba(40, 125, 245, 0.2);
  outline-offset: 2px;
}

.materials-page .material-bucket.format-accent-pdf,
.materials-page .material-open-button.format-accent-pdf {
  color: var(--el-color-danger);
}

.materials-page .material-bucket.format-accent-slides,
.materials-page .material-open-button.format-accent-slides {
  color: var(--el-color-warning);
}

.materials-page .material-bucket.format-accent-word,
.materials-page .material-open-button.format-accent-word {
  color: var(--el-color-primary);
}

.materials-page .material-bucket.format-accent-note,
.materials-page .material-open-button.format-accent-note {
  color: var(--el-color-success);
}

.materials-page .material-bucket.format-accent-image,
.materials-page .material-open-button.format-accent-image {
  color: var(--el-color-info);
}

.materials-page .material-bucket.format-accent-document,
.materials-page .material-open-button.format-accent-document {
  color: var(--el-color-info);
}

.materials-page .material-open-button.format-accent-pdf:hover {
  background: var(--el-color-danger-light-9);
}

.materials-page .material-open-button.format-accent-slides:hover {
  background: var(--el-color-warning-light-9);
}

.materials-page .material-open-button.format-accent-word:hover {
  background: var(--el-color-primary-light-9);
}

.materials-page .material-open-button.format-accent-note:hover {
  background: var(--el-color-success-light-9);
}

.materials-page .material-open-button.format-accent-image:hover,
.materials-page .material-open-button.format-accent-document:hover {
  background: var(--el-color-info-light-9);
}
</style>