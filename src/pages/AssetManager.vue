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
      <el-tabs v-model="selectedSubject" class="subject-tabs" type="card">
        <el-tab-pane v-for="subject in subjects" :key="subject" :name="subject">
          <template #label>
            {{ subject }}
            <span>{{ subjectCount(subject) }}</span>
          </template>
        </el-tab-pane>
      </el-tabs>
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
          <div class="material-subject">{{ material.subject }}</div>
          <h2 :title="material.name">{{ material.name }}</h2>
          <div class="material-meta">
            <span>{{ formatSize(material.size) }}</span>
            <span>添加于 {{ material.addedAt }}</span>
          </div>
          <div class="material-actions">
            <el-button plain size="small" @click="openMaterial(material)">打开</el-button>
            <el-button text type="danger" size="small" title="移除资料" @click="removeMaterial(material.id)">移除</el-button>
          </div>
        </div>
      </el-card>
    </section>

    <section v-else class="empty-materials">
      <el-empty :description="materials.length ? '这个学科还没有资料' : '添加第一份学习资料'">
        <p>{{ materials.length ? '切换其他学科，或添加一份新的资料。' : '选择 PDF、PPT 或笔记文件，让学习资料集中在这里。' }}</p>
        <el-button type="primary" @click="openUploadPicker">选择文件</el-button>
      </el-empty>
    </section>

    <el-dialog v-model="showSubjectDialog" title="添加学习资料" width="min(460px, 92vw)" destroy-on-close @closed="resetPendingUpload">
      <p class="selected-files">已选择 {{ pendingFiles.length }} 个文件，每个文件会生成一个独立资料模块。</p>
      <el-form label-position="top" @submit.prevent="confirmUpload">
        <el-form-item label="所属学科">
          <el-select v-model="pendingSubject" class="dialog-control">
            <el-option v-for="subject in subjectOptions" :key="subject" :label="subject" :value="subject" />
          </el-select>
        </el-form-item>
        <el-form-item label="或新建学科">
          <el-input v-model="newSubject" placeholder="例如：操作系统" />
        </el-form-item>
      </el-form>
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
import { computed, onBeforeUnmount, ref } from 'vue'
import type { UploadFile, UploadInstance } from 'element-plus'
import { ElMessage } from 'element-plus'
import { Plus } from '@element-plus/icons-vue'

type MaterialKind = 'pdf' | 'slides' | 'note' | 'image' | 'document'

type Material = {
  id: string
  name: string
  extension: string
  typeLabel: string
  kind: MaterialKind
  subject: string
  size: number
  addedAt: string
  url: string
}

const uploadRef = ref<UploadInstance>()
const materials = ref<Material[]>([])
const selectedSubject = ref('全部')
const pendingFiles = ref<File[]>([])
const pendingSubject = ref('数学')
const newSubject = ref('')
const showSubjectDialog = ref(false)
const uploadError = ref('')
const sortBy = ref<'updated' | 'name' | 'size'>('updated')

const subjectOptions = ['数学', '英语', '操作系统']
const subjects = computed(() => ['全部', ...subjectOptions.filter(subject => materials.value.some(item => item.subject === subject))])

const filteredMaterials = computed(() => {
  const filtered = selectedSubject.value === '全部'
    ? materials.value
    : materials.value.filter(material => material.subject === selectedSubject.value)

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

function confirmUpload() {
  const subject = newSubject.value.trim() || pendingSubject.value
  if (!subject || !pendingFiles.value.length) {
    return
  }

  pendingFiles.value.forEach(file => {
    materials.value.push({
      id: `${Date.now()}-${file.name}-${Math.random()}`,
      name: file.name,
      extension: getExtension(file.name),
      typeLabel: getTypeLabel(file.name),
      kind: getMaterialKind(file.name),
      subject,
      size: file.size,
      addedAt: '刚刚',
      url: URL.createObjectURL(file),
    })
  })

  if (!subjectOptions.includes(subject)) {
    subjectOptions.push(subject)
  }
  selectedSubject.value = subject
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

function removeMaterial(id: string) {
  const material = materials.value.find(item => item.id === id)
  if (!material) {
    return
  }
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
  return 'document'
}

function formatSize(size: number) {
  if (size < 1024 * 1024) {
    return `${Math.max(1, Math.round(size / 1024))} KB`
  }
  return `${(size / 1024 / 1024).toFixed(1)} MB`
}

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

.dark-button,
.outline-button,
.remove-button {
  border-radius: 6px;
  font-size: 12px;
}

.dark-button {
  padding: 10px 14px;
  border: 1px solid #25231f;
  background: #25231f;
  color: #fff;
}

.outline-button {
  padding: 8px 11px;
  border: 1px solid #d8d3ca;
  background: #fff;
  color: #514b43;
}

.hidden-file-input {
  display: none;
}

.materials-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 18px;
  margin-bottom: 18px;
  padding-bottom: 12px;
  border-bottom: 1px solid #e5e1da;
}

.subject-tabs {
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.subject-tab {
  padding: 8px 10px;
  border: 0;
  border-radius: 6px;
  background: transparent;
  color: #8b8278;
  font-size: 12px;
}

.subject-tab span {
  margin-left: 4px;
  color: #b3aaa0;
  font-size: 11px;
}

.subject-tab:hover,
.subject-tab.active {
  background: #f0ede8;
  color: #715e4b;
}

.sort-control {
  display: flex;
  align-items: center;
  flex-shrink: 0;
  gap: 8px;
  color: #9a9289;
  font-size: 12px;
}

.sort-select {
  width: 160px;
}

.sort-control select,
.modal select,
.modal input {
  height: 36px;
  padding: 0 10px;
  border: 1px solid #dedbd5;
  border-radius: 6px;
  outline: none;
  background: #fff;
  color: #514b43;
  font-size: 12px;
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

.remove-button {
  padding: 0;
  border: 0;
  background: transparent;
  color: #a65345;
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

.empty-icon {
  width: 44px;
  height: 44px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  background: #eee5da;
  color: #896e50;
  font-size: 25px;
}

.empty-materials h2 {
  margin-top: 17px;
  color: #5e554b;
  font-size: 18px;
}

.empty-materials p {
  margin: 8px 0 18px;
  color: #9a9289;
  font-size: 13px;
}

.feedback {
  width: min(1220px, 100%);
  margin: 0 auto 16px;
  padding: 9px 11px;
  border-radius: 6px;
  background: #fff1ee;
  color: #a34e3f;
  font-size: 12px;
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

.modal-heading,
.modal-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
}

.modal h2 {
  margin-top: 7px;
  font-size: 20px;
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

.selected-files {
  margin-top: 20px;
  color: #8f867d;
  font-size: 12px;
  line-height: 1.5;
}

.field-label {
  display: block;
  margin: 20px 0 8px;
  color: #4c4944;
  font-size: 12px;
  font-weight: 650;
}

.modal select,
.modal input {
  width: 100%;
}

.modal-actions {
  justify-content: flex-end;
  margin-top: 24px;
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
    align-items: flex-start;
    flex-direction: column;
  }

  .sort-control,
  .sort-select {
    width: 100%;
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

.materials-page .subject-tab {
  color: var(--learning-text-secondary);
}

.materials-page .subject-tab span {
  color: var(--learning-text-muted);
}

.materials-page .subject-tab:hover,
.materials-page .subject-tab.active {
  background: #eaf2ff;
  color: var(--learning-primary);
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

.materials-page .cover-pdf,
.materials-page .cover-slides,
.materials-page .cover-note,
.materials-page .cover-image,
.materials-page .cover-document {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.materials-page .material-subject {
  color: var(--learning-primary);
}

.materials-page .material-body h2 {
  color: var(--learning-text);
}

.materials-page .remove-button {
  color: var(--el-color-danger);
}

.materials-page .empty-materials {
  border-color: #b7d0f8;
  background: rgba(255, 255, 255, 0.72);
}

.materials-page .empty-icon {
  background: #eaf2ff;
  color: var(--learning-primary);
}

.materials-page .empty-materials h2 {
  color: var(--learning-text);
}

.materials-page .feedback {
  background: var(--el-color-danger-light-9);
  color: var(--el-color-danger);
}

.materials-page .modal-backdrop {
  background: rgba(15, 35, 65, 0.38);
}

.materials-page .modal {
  box-shadow: 0 20px 60px rgba(35, 75, 130, 0.2);
}

.materials-page .close-button {
  background: #eaf2ff;
  color: var(--learning-primary);
}
</style>
