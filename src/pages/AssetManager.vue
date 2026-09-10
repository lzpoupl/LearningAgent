<template>
  <main class="materials-page">
    <header class="materials-header">
      <div>
        <div class="eyebrow">学习资产 / MATERIALS</div>
        <h1>学习资料</h1>
        <p>PDF、PPT、笔记等非结构化学习资产</p>
      </div>
      <el-upload
        ref="uploadRef"
        action="#"
        :auto-upload="false"
        :show-file-list="false"
        multiple
        accept=".pdf,.ppt,.pptx,.doc,.docx,.txt,.md,.markdown,.png,.jpg,.jpeg,.webp"
        :on-change="handleFilesSelected"
      >
        <el-button type="primary">
          <el-icon><Plus /></el-icon>
          添加资料
        </el-button>
      </el-upload>
    </header>

    <section class="materials-toolbar">
      <el-popover v-model:visible="subjectMenuOpen" class="subject-picker-popover" placement="bottom-start"
        :width="270" trigger="click">
        <template #reference>
          <el-button class="subject-picker" plain>
            <span class="subject-picker-label">{{ selectedSubject }}资料</span>
            <el-icon><ArrowDown /></el-icon>
          </el-button>
        </template>
        <el-tree
          class="subject-tree"
          :data="subjectTreeData"
          node-key="value"
          :props="subjectTreeProps"
          highlight-current
          :current-node-key="selectedSubject"
          @node-click="handleSubjectNodeClick"
        >
          <template #default="{ data }">
            <span class="subject-tree-node" :title="data.folder ? `物理存储：${data.folder}` : '全部学科文件夹'">
              <span class="subject-tree-label">{{ data.label }}</span>
              <span class="subject-tree-count">{{ data.count }}</span>
            </span>
          </template>
        </el-tree>
      </el-popover>

      <!-- 搜索框：位于排序左侧 -->
      <el-input v-model="searchKeyword" class="material-search" clearable placeholder="搜索资料名称" aria-label="搜索资料">
        <template #prefix>
          <el-icon><Search /></el-icon>
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

    <div v-if="uploadError" class="feedback error" role="alert">{{ uploadError }}</div>

    <section v-if="filteredMaterials.length" class="materials-grid" aria-label="资料列表">
      <el-card v-for="material in filteredMaterials" :key="material.id" class="material-card" shadow="hover" :body-style="{ padding: '0' }">
        <div class="material-cover" :class="`cover-${material.kind}`">
          <span class="file-mark">{{ material.extension }}</span>
          <span class="file-type">{{ material.typeLabel }}</span>
        </div>
        <div class="material-body">
          <div class="material-subject" :class="`format-accent-${material.kind}`" :title="`物理存储：${material.folder}`">
            {{ material.subject }}
          </div>
          <h2 :title="material.name">{{ material.name }}</h2>
          <div class="material-meta">
            <span>{{ formatSize(material.size) }}</span>
            <span>添加于 {{ material.addedAt }}</span>
          </div>
          <div class="material-actions">
            <el-button class="material-open-button" :class="`format-accent-${material.kind}`" type="primary" text size="small" @click="openMaterial(material)">
              <el-icon><FolderOpened /></el-icon>
              打开
            </el-button>
            <el-button class="material-remove-button" text type="danger" size="small" title="移除资料" @click="removeMaterial(material.id)">
              <el-icon><Delete /></el-icon>
              删除
            </el-button>
          </div>
        </div>
      </el-card>
    </section>

    <section v-else class="empty-materials">
      <el-empty :description="materials.length ? (searchKeyword ? '没有匹配的资料' : '这个学科还没有资料') : '添加第一份学习资料'">
        <p>{{ materials.length ? (searchKeyword ? '调整搜索关键词，再试一次。' : '切换其他学科，或添加一份新的资料。') : '选择 PDF、PPT 或笔记文件，让学习资料集中在这里。' }}</p>
        <el-button type="primary" @click="openUploadPicker">选择文件</el-button>
      </el-empty>
    </section>

    <el-dialog v-model="showSubjectDialog" title="添加学习资料" width="min(460px, 92vw)" destroy-on-close @closed="resetPendingUpload">
      <p class="selected-files">已选择 {{ pendingFiles.length }} 个文件，每个文件会生成一个独立资料模块。</p>
      <el-form label-position="top" @submit.prevent="confirmUpload">
        <el-form-item label="所属学科">
          <el-select v-model="pendingSubject" class="dialog-control">
            <el-option v-for="item in subjectFolders" :key="item.name" :label="item.name" :value="item.name" />
          </el-select>
        </el-form-item>
        <el-form-item label="或新建学科">
          <el-input v-model="newSubject" placeholder="例如：操作系统" />
        </el-form-item>
      </el-form>
      <p class="target-folder">
        归档到物理文件夹：<code>{{ pendingFolderPath || '—' }}</code>
      </p>
      <template #footer>
        <el-button @click="cancelUpload">取消</el-button>
        <el-button type="primary" :disabled="!pendingFiles.length || (!pendingSubject && !newSubject.trim())" @click="confirmUpload">
          添加资料
        </el-button>
      </template>
    </el-dialog>
  </main>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from 'vue'
import type { UploadFile, UploadInstance } from 'element-plus'
import { ElMessage } from 'element-plus'
import { ArrowDown, Delete, FolderOpened, Plus, Search } from '@element-plus/icons-vue'

type MaterialKind = 'pdf' | 'slides' | 'note' | 'image' | 'word' | 'document'

/** 学科 <-> 物理文件夹 的一一映射 */
type SubjectFolder = {
  name: string
  folder: string
}

type Material = {
  id: string
  name: string
  extension: string
  typeLabel: string
  kind: MaterialKind
  subject: string
  /** 该资料在总仓库中的物理存储文件夹 */
  folder: string
  size: number
  addedAt: string
  url: string
}

type UploadMaterialRequest = {
  file: File
  subject: string
  folder: string
}

const uploadRef = ref<UploadInstance>()
const materials = ref<Material[]>([])

/** 学科与物理文件夹的映射表：来自服务接口 */
const subjectFolders = ref<SubjectFolder[]>([])

const selectedSubject = ref('全部')
const pendingFiles = ref<File[]>([])
const pendingSubject = ref('')
const newSubject = ref('')
const showSubjectDialog = ref(false)
const uploadError = ref('')
const sortBy = ref<'updated' | 'name' | 'size'>('updated')
const searchKeyword = ref('')
const subjectMenuOpen = ref(false)

const subjects = computed(() => ['全部', ...subjectFolders.value.map(item => item.name)])

const subjectTreeData = computed(() => subjects.value.map(subject => ({
  label: subject === '全部' ? '全部资料' : subject,
  value: subject,
  count: subjectCount(subject),
  folder: subject === '全部'
    ? ''
    : subjectFolders.value.find(item => item.name === subject)?.folder ?? '',
})))

const subjectTreeProps = { label: 'label', children: 'children' }

/** 弹窗中展示的目标文件夹（新学科按同一规则推导，保证一一对应） */
const pendingFolderPath = computed(() => {
  const name = newSubject.value.trim()
  if (name) {
    const existing = subjectFolders.value.find(item => item.name === name)
    return existing ? existing.folder : `materials/${sanitizeFolderName(name)}`
  }
  return subjectFolders.value.find(item => item.name === pendingSubject.value)?.folder ?? ''
})

const filteredMaterials = computed(() => {
  const subjectFiltered = selectedSubject.value === '全部'
    ? materials.value
    : materials.value.filter(material => material.subject === selectedSubject.value)
  const normalizedKeyword = searchKeyword.value.trim().toLocaleLowerCase()
  const filtered = subjectFiltered.filter(material => {
    const searchableText = [material.name, material.extension, material.typeLabel, material.subject]
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
    return right.id.localeCompare(left.id)
  })
})

function subjectCount(subject: string) {
  return subject === '全部'
    ? materials.value.length
    : materials.value.filter(material => material.subject === subject).length
}

/** 保证一个学科只会生成一个文件夹，且文件夹名唯一 */
function ensureSubjectFolder(subject: string) {
  const existing = subjectFolders.value.find(item => item.name === subject)
  if (existing) {
    return existing.folder
  }

  const base = `materials/${sanitizeFolderName(subject)}`
  let folder = base
  let suffix = 2
  while (subjectFolders.value.some(item => item.folder === folder)) {
    folder = `${base}-${suffix}`
    suffix += 1
  }

  subjectFolders.value.push({ name: subject, folder })
  return folder
}

/** 把学科名转换成安全的文件夹名 */
function sanitizeFolderName(name: string) {
  const cleaned = name
    .trim()
    .replace(/[\\/:*?"<>|]+/g, '-')
    .replace(/\s+/g, '-')
    .replace(/-+/g, '-')
    .replace(/^[-.]+|[-.]+$/g, '')
  return cleaned || 'untitled'
}

// TODO:
// 后面替换成真实后端 API：GET /api/subject-folders
async function fetchSubjectFolders(): Promise<SubjectFolder[]> {
  console.log('获取学科与物理文件夹映射')

  return [
    { name: '数学', folder: 'materials/math' },
    { name: '英语', folder: 'materials/english' },
    { name: '操作系统', folder: 'materials/os' },
  ]
}

// TODO:
// 后面替换成真实后端 API：GET /api/materials
async function fetchMaterials(): Promise<Material[]> {
  console.log('获取资料列表')

  return []
}

// TODO:
// 后面替换成真实后端 API：POST /api/materials
async function uploadMaterial(request: UploadMaterialRequest): Promise<Material> {
  const { file, subject, folder } = request

  console.log('发送给后端：', request)

  return {
    id: `${Date.now()}-${file.name}-${Math.random()}`,
    name: file.name,
    extension: getExtension(file.name),
    typeLabel: getTypeLabel(file.name),
    kind: getMaterialKind(file.name),
    subject,
    folder,
    size: file.size,
    addedAt: '刚刚',
    url: URL.createObjectURL(file),
  }
}

// TODO:
// 后面替换成真实后端 API：DELETE /api/materials/:id
async function deleteMaterial(id: string): Promise<void> {
  console.log('删除资料：', id)
}

function handleSubjectNodeClick(data: { value: string }) {
  selectedSubject.value = data.value
  subjectMenuOpen.value = false
}

function openUploadPicker() {
  const uploadElement = uploadRef.value?.$el as HTMLElement | undefined
  uploadElement?.querySelector<HTMLInputElement>('input[type="file"]')?.click()
}

function handleFilesSelected(file: UploadFile) {
  const rawFile = file.raw

  if (!rawFile) {
    return
  }

  if (!isSupportedFile(rawFile)) {
    uploadError.value = `暂不支持“${rawFile.name}”，请选择 PDF、PPT 或笔记文件。`
    ElMessage.error(uploadError.value)
    uploadRef.value?.clearFiles()
    return
  }

  uploadError.value = ''
  if (!pendingFiles.value.some(item => item.name === rawFile.name && item.size === rawFile.size)) {
    pendingFiles.value.push(rawFile)
  }
  showSubjectDialog.value = true
}

function isSupportedFile(file: File) {
  return /\.(pdf|ppt|pptx|doc|docx|txt|md|markdown|png|jpg|jpeg|webp)$/i.test(file.name)
}

async function confirmUpload() {
  const subject = newSubject.value.trim() || pendingSubject.value
  if (!subject || !pendingFiles.value.length) {
    return
  }

  // 学科 -> 物理文件夹，一一对应
  const folder = ensureSubjectFolder(subject)

  const uploadedMaterials = await Promise.all(
    pendingFiles.value.map(file => uploadMaterial({ file, subject, folder }))
  )
  materials.value.push(...uploadedMaterials)

  selectedSubject.value = subject
  subjectMenuOpen.value = false
  cancelUpload()
}

function cancelUpload() {
  uploadRef.value?.clearFiles()
  resetPendingUpload()
  showSubjectDialog.value = false
}

function resetPendingUpload() {
  pendingFiles.value = []
  newSubject.value = ''
}

function openMaterial(material: Material) {
  window.open(material.url, '_blank', 'noopener,noreferrer')
}

async function removeMaterial(id: string) {
  const material = materials.value.find(item => item.id === id)
  if (!material) {
    return
  }
  await deleteMaterial(id)
  URL.revokeObjectURL(material.url)
  materials.value = materials.value.filter(item => item.id !== id)
}

function getExtension(name: string) {
  return name.split('.').pop()?.toUpperCase() || 'FILE'
}

function getTypeLabel(name: string) {
  const extension = getExtension(name)
  if (extension === 'PDF') return 'PDF 文档'
  if (extension === 'PPT' || extension === 'PPTX') return '演示文稿'
  if (extension === 'DOC' || extension === 'DOCX') return 'Word 文档'
  if (extension === 'MD' || extension === 'MARKDOWN' || extension === 'TXT') return '笔记'
  if (['PNG', 'JPG', 'JPEG', 'WEBP'].includes(extension)) return '图片'
  return '文档'
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

onMounted(async () => {
  subjectFolders.value = await fetchSubjectFolders()
  materials.value = await fetchMaterials()
  if (subjectFolders.value.length) {
    pendingSubject.value = subjectFolders.value[0].name
  }
})

onBeforeUnmount(() => {
  materials.value.forEach(material => URL.revokeObjectURL(material.url))
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
.empty-materials,
.feedback {
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

.subject-picker {
  flex: 0 0 220px;
  width: 220px;
  justify-content: space-between;
  border-color: var(--learning-border);
  color: var(--learning-text-secondary);
  text-align: left;
}

.subject-picker:hover,
.subject-picker:focus {
  border-color: #b7d0f8;
  background: #eef5ff;
  color: var(--learning-primary);
}

.subject-picker-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subject-tree {
  margin: -6px;
  color: var(--learning-text);
}

.subject-tree :deep(.el-tree-node__content) {
  height: 38px;
  border-radius: 6px;
}

.subject-tree :deep(.el-tree-node__content:hover),
.subject-tree :deep(.is-current > .el-tree-node__content) {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.subject-tree-node {
  width: 100%;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding-right: 8px;
  font-size: 13px;
}

.subject-tree-label {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.subject-tree-count {
  flex-shrink: 0;
  color: var(--learning-text-muted);
  font-size: 11px;
}

/* 搜索框：紧跟学科选择器，位于排序左侧 */
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

.cover-pdf { background: #f3e7df; color: #9d604b; }
.cover-slides { background: #f1eadc; color: #967345; }
.cover-note { background: #e9eee8; color: #647b67; }
.cover-image { background: #e7ecee; color: #5f7680; }
.cover-document { background: #ece9e4; color: #73695e; }

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

.material-subject {
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

.feedback {
  margin-bottom: 16px;
  padding: 9px 11px;
  border-radius: 6px;
  background: #fff1ee;
  color: #a34e3f;
  font-size: 12px;
}

.selected-files {
  margin-top: 20px;
  color: #8f867d;
  font-size: 12px;
  line-height: 1.5;
}

.dialog-control {
  width: 100%;
}

.target-folder {
  margin-top: 4px;
  color: var(--learning-text-muted);
  font-size: 12px;
}

.target-folder code {
  padding: 1px 6px;
  border-radius: 4px;
  background: #eaf2ff;
  color: var(--learning-primary);
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', monospace;
  font-size: 11px;
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

  .subject-picker,
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
.materials-page .empty-materials p,
.materials-page .selected-files {
  color: var(--learning-text-secondary);
}

.materials-page .materials-toolbar {
  border-bottom-color: var(--learning-border);
}

.materials-page .material-card {
  border-color: var(--learning-border);
  box-shadow: var(--learning-shadow);
}

.materials-page .material-card:hover {
  box-shadow: 0 10px 24px rgba(40, 125, 245, 0.14);
}

.materials-page .material-cover {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.materials-page .cover-pdf {
  background: #fef0f0;
  color: #f56c6c;
}

.materials-page .cover-slides {
  background: #fdf6ec;
  color: #e6a23c;
}

.materials-page .cover-word {
  background: #ecf5ff;
  color: #409eff;
}

.materials-page .cover-note {
  background: #f0f9eb;
  color: #67c23a;
}

.materials-page .cover-image {
  background: #f4f4f5;
  color: #909399;
}

.materials-page .cover-document {
  background: #f4f4f5;
  color: #606266;
}

.materials-page .material-subject {
  color: var(--learning-primary);
}

.materials-page .material-body h2 {
  color: var(--learning-text);
}

.materials-page .empty-materials {
  border-color: #b7d0f8;
  background: rgba(255, 255, 255, 0.72);
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
  background: #eaf2ff;
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

.materials-page .material-subject.format-accent-pdf,
.materials-page .material-open-button.format-accent-pdf {
  color: #f56c6c;
}

.materials-page .material-subject.format-accent-slides,
.materials-page .material-open-button.format-accent-slides {
  color: #e6a23c;
}

.materials-page .material-subject.format-accent-word,
.materials-page .material-open-button.format-accent-word {
  color: #409eff;
}

.materials-page .material-subject.format-accent-note,
.materials-page .material-open-button.format-accent-note {
  color: #67c23a;
}

.materials-page .material-subject.format-accent-image,
.materials-page .material-open-button.format-accent-image {
  color: #909399;
}

.materials-page .material-subject.format-accent-document,
.materials-page .material-open-button.format-accent-document {
  color: #606266;
}

.materials-page .material-open-button.format-accent-pdf:hover {
  background: #fef0f0;
}

.materials-page .material-open-button.format-accent-slides:hover {
  background: #fdf6ec;
}

.materials-page .material-open-button.format-accent-word:hover {
  background: #ecf5ff;
}

.materials-page .material-open-button.format-accent-note:hover {
  background: #f0f9eb;
}

.materials-page .material-open-button.format-accent-image:hover,
.materials-page .material-open-button.format-accent-document:hover {
  background: #f4f4f5;
}

.materials-page .feedback {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}
</style>