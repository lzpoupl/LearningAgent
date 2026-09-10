import type {
  AssetKind,
  AssetQuery,
  LearningAsset,
  UploadAssetRequest,
} from '../types/assets'

function nowIso(): string {
  return new Date().toISOString()
}

function dataUrl(content: string): string {
  return `data:text/plain;charset=utf-8,${encodeURIComponent(content)}`
}

const seedAssets: LearningAsset[] = [
  {
    id: 'asset-math-foundation',
    name: '高等数学基础.pdf',
    extension: 'PDF',
    typeLabel: 'PDF 文档',
    kind: 'pdf',
    subject: '数学',
    size: 2_400_000,
    mimeType: 'application/pdf',
    addedAt: new Date(Date.now() - 2 * 3_600_000).toISOString(),
    url: dataUrl('Mock: 高等数学基础'),
  },
  {
    id: 'asset-english-vocabulary',
    name: '考研英语词汇.pdf',
    extension: 'PDF',
    typeLabel: 'PDF 文档',
    kind: 'pdf',
    subject: '英语',
    size: 1_800_000,
    mimeType: 'application/pdf',
    addedAt: new Date(Date.now() - 24 * 3_600_000).toISOString(),
    url: dataUrl('Mock: 考研英语词汇'),
  },
  {
    id: 'asset-os-process',
    name: '进程管理课件.pptx',
    extension: 'PPTX',
    typeLabel: '演示文稿',
    kind: 'slides',
    subject: '操作系统',
    size: 4_200_000,
    mimeType: 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    addedAt: new Date(Date.now() - 3 * 24 * 3_600_000).toISOString(),
    url: dataUrl('Mock: 进程管理课件'),
  },
  {
    id: 'asset-math-notes',
    name: '极限与导数笔记.md',
    extension: 'MD',
    typeLabel: '笔记',
    kind: 'note',
    subject: '数学',
    size: 24_000,
    mimeType: 'text/markdown',
    addedAt: new Date(Date.now() - 5 * 24 * 3_600_000).toISOString(),
    url: dataUrl('Mock: 极限与导数笔记'),
  },
]

function cloneAsset(asset: LearningAsset): LearningAsset {
  return { ...asset }
}

function normalizeKind(extension: string): AssetKind {
  if (extension === 'PDF') return 'pdf'
  if (extension === 'PPT' || extension === 'PPTX') return 'slides'
  if (extension === 'MD' || extension === 'MARKDOWN' || extension === 'TXT') return 'note'
  if (['PNG', 'JPG', 'JPEG', 'WEBP'].includes(extension)) return 'image'
  return 'document'
}

function typeLabel(kind: AssetKind): string {
  if (kind === 'pdf') return 'PDF 文档'
  if (kind === 'slides') return '演示文稿'
  if (kind === 'note') return '笔记'
  if (kind === 'image') return '图片'
  return '文档'
}

function extensionOf(name: string): string {
  return name.split('.').pop()?.toUpperCase() || 'FILE'
}

/** 内存版学习资料后端，负责资料元数据和上传内容的开发期响应。 */
export class AssetMock {
  private assets = new Map(seedAssets.map(asset => [asset.id, cloneAsset(asset)]))
  private nextAssetId = 1

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'asset_list':
        return this.list((payload.query ?? {}) as AssetQuery)
      case 'asset_list_subjects':
        return this.listSubjects()
      case 'asset_upload':
        return this.upload((payload.input ?? {}) as Partial<UploadAssetRequest>)
      case 'asset_get_url':
        return this.getUrl(String(payload.assetId ?? ''))
      case 'asset_delete':
        return this.delete(String(payload.assetId ?? ''))
      default:
        return undefined
    }
  }

  private list(query: AssetQuery): LearningAsset[] {
    let assets = [...this.assets.values()]
    if (query.subject) {
      assets = assets.filter(asset => asset.subject === query.subject)
    }

    if (query.sortBy === 'name') {
      assets.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN'))
    } else if (query.sortBy === 'size') {
      assets.sort((left, right) => right.size - left.size)
    } else {
      assets.sort((left, right) => right.addedAt.localeCompare(left.addedAt))
    }

    return assets.map(cloneAsset)
  }

  private listSubjects(): string[] {
    return [...new Set([...this.assets.values()].map(asset => asset.subject))]
  }

  private upload(input: Partial<UploadAssetRequest>): LearningAsset {
    const name = String(input.name ?? '').trim()
    const subject = String(input.subject ?? '').trim()
    const mimeType = String(input.mimeType ?? 'application/octet-stream')
    const contentBase64 = String(input.contentBase64 ?? '')
    if (!name || !subject || !contentBase64) {
      throw new Error('资料上传参数不完整')
    }

    const extension = extensionOf(name)
    const kind = normalizeKind(extension)
    const asset: LearningAsset = {
      id: `asset-upload-${this.nextAssetId++}`,
      name,
      extension,
      typeLabel: typeLabel(kind),
      kind,
      subject,
      size: Number(input.size ?? 0),
      mimeType,
      addedAt: nowIso(),
      url: `data:${mimeType};base64,${contentBase64}`,
    }
    this.assets.set(asset.id, asset)
    return cloneAsset(asset)
  }

  private getUrl(assetId: string): string {
    const asset = this.assets.get(assetId)
    if (!asset) {
      throw new Error(`学习资料不存在: ${assetId}`)
    }
    return asset.url ?? ''
  }

  private delete(assetId: string): void {
    if (!this.assets.delete(assetId)) {
      throw new Error(`学习资料不存在: ${assetId}`)
    }
  }
}
