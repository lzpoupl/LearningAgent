import type {
  AssetKind,
  AssetQuery,
  Bucket,
  LearningAsset,
  UploadAssetRequest,
  UploadImageRequest,
  UploadedImage,
} from '../types/assets'

function nowIso(): string {
  return new Date().toISOString()
}

function dataUrl(mimeType: string, contentBase64: string): string {
  return `data:${mimeType};base64,${contentBase64}`
}

function extensionOf(name: string): string {
  return name.split('.').pop()?.toLowerCase() || ''
}

function kindOf(extension: string): AssetKind {
  if (extension === 'pdf') return 'pdf'
  if (extension === 'ppt' || extension === 'pptx') return 'slides'
  if (extension === 'md' || extension === 'markdown' || extension === 'txt') return 'note'
  if (['png', 'jpg', 'jpeg', 'webp', 'gif'].includes(extension)) return 'image'
  if (extension === 'doc' || extension === 'docx') return 'document'
  return 'other'
}

function typeLabelOf(kind: AssetKind): string {
  if (kind === 'pdf') return 'PDF 文档'
  if (kind === 'slides') return '演示文稿'
  if (kind === 'note') return '笔记'
  if (kind === 'image') return '图片'
  if (kind === 'document') return '文档'
  return '其他'
}

function mimeTypeOf(extension: string): string {
  if (extension === 'pdf') return 'application/pdf'
  if (extension === 'ppt') return 'application/vnd.ms-powerpoint'
  if (extension === 'pptx') {
    return 'application/vnd.openxmlformats-officedocument.presentationml.presentation'
  }
  if (extension === 'md' || extension === 'markdown') return 'text/markdown'
  if (extension === 'txt') return 'text/plain'
  if (extension === 'png') return 'image/png'
  if (extension === 'jpg' || extension === 'jpeg') return 'image/jpeg'
  if (extension === 'webp') return 'image/webp'
  if (extension === 'gif') return 'image/gif'
  if (extension === 'doc') return 'application/msword'
  if (extension === 'docx') {
    return 'application/vnd.openxmlformats-officedocument.wordprocessingml.document'
  }
  return 'application/octet-stream'
}

const seedBuckets: Bucket[] = [
  {
    id: 1,
    name: '资料',
    rootPath: 'C:\\Users\\you\\LearningAgent\\资料',
    assetCount: 4,
    createdAt: new Date(Date.now() - 6 * 24 * 3_600_000).toISOString(),
    updatedAt: nowIso(),
  },
]

const seedAssets: LearningAsset[] = [
  {
    id: '/资料/高等数学基础.pdf',
    name: '高等数学基础.pdf',
    extension: 'pdf',
    typeLabel: 'PDF 文档',
    kind: 'pdf',
    size: 2_400_000,
    mimeType: 'application/pdf',
    addedAt: new Date(Date.now() - 2 * 3_600_000).toISOString(),
  },
  {
    id: '/资料/考研英语词汇.pdf',
    name: '考研英语词汇.pdf',
    extension: 'pdf',
    typeLabel: 'PDF 文档',
    kind: 'pdf',
    size: 1_800_000,
    mimeType: 'application/pdf',
    addedAt: new Date(Date.now() - 24 * 3_600_000).toISOString(),
  },
  {
    id: '/资料/进程管理课件.pptx',
    name: '进程管理课件.pptx',
    extension: 'pptx',
    typeLabel: '演示文稿',
    kind: 'slides',
    size: 4_200_000,
    mimeType: 'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    addedAt: new Date(Date.now() - 3 * 24 * 3_600_000).toISOString(),
  },
  {
    id: '/资料/极限与导数笔记.md',
    name: '极限与导数笔记.md',
    extension: 'md',
    typeLabel: '笔记',
    kind: 'note',
    size: 24_000,
    mimeType: 'text/markdown',
    addedAt: new Date(Date.now() - 5 * 24 * 3_600_000).toISOString(),
  },
]

function cloneBucket(bucket: Bucket): Bucket {
  return { ...bucket }
}

function cloneAsset(asset: LearningAsset): LearningAsset {
  return { ...asset }
}

/** 内存版资产后端：bucket 映射与资产元数据（资产 id 为 `/<bucket>/<path>`）。 */
export class AssetMock {
  private buckets = new Map<number, Bucket>(seedBuckets.map(bucket => [bucket.id, cloneBucket(bucket)]))
  private assets = new Map<string, LearningAsset>(seedAssets.map(asset => [asset.id, cloneAsset(asset)]))
  private nextBucketId = seedBuckets.length + 1

  handle(cmd: string, payload: Record<string, unknown>): unknown {
    switch (cmd) {
      case 'bucket_list':
        return this.listBuckets()
      case 'bucket_create':
        return this.createBucket(String(payload.name ?? ''), String(payload.rootPath ?? ''))
      case 'bucket_update':
        return this.updateBucket(
          Number(payload.id ?? 0),
          payload.name === undefined ? undefined : String(payload.name),
          payload.rootPath === undefined ? undefined : String(payload.rootPath),
        )
      case 'bucket_delete':
        return this.deleteBucket(Number(payload.id ?? 0))
      case 'asset_list':
        return this.list((payload.query ?? {}) as AssetQuery)
      case 'asset_upload':
        return this.upload((payload.input ?? {}) as Partial<UploadAssetRequest>)
      case 'asset_get_url':
        return this.getUrl(String(payload.assetId ?? ''))
      case 'asset_delete':
        return this.delete(String(payload.assetId ?? ''))
      case 'asset_upload_image':
        return this.uploadImage((payload.input ?? {}) as Partial<UploadImageRequest>)
      default:
        return undefined
    }
  }

  // ---- bucket ----

  private listBuckets(): Bucket[] {
    return [...this.buckets.values()]
      .map(bucket => this.withCount(bucket))
      .sort((left, right) => left.id - right.id)
  }

  private withCount(bucket: Bucket): Bucket {
    return { ...bucket, assetCount: this.assetsFor(bucket.name).length }
  }

  private requireBucket(id: number): Bucket {
    const bucket = this.buckets.get(id)
    if (!bucket) {
      throw new Error(`bucket 不存在: ${id}`)
    }
    return bucket
  }

  private findBucketByName(name: string): Bucket | undefined {
    return [...this.buckets.values()].find(bucket => bucket.name === name)
  }

  private createBucket(name: string, rootPath: string): Bucket {
    const trimmedName = name.trim()
    const trimmedRoot = rootPath.trim()
    if (!trimmedName || !trimmedRoot) {
      throw new Error('bucket 名称或目录不能为空')
    }
    if (this.findBucketByName(trimmedName)) {
      throw new Error(`bucket 名称已存在: ${trimmedName}`)
    }

    const bucket: Bucket = {
      id: this.nextBucketId++,
      name: trimmedName,
      rootPath: trimmedRoot,
      assetCount: 0,
      createdAt: nowIso(),
      updatedAt: nowIso(),
    }
    this.buckets.set(bucket.id, bucket)
    return cloneBucket(bucket)
  }

  private updateBucket(id: number, name?: string, rootPath?: string): Bucket {
    const current = this.requireBucket(id)
    const nextName = name === undefined ? current.name : name.trim()
    const nextRoot = rootPath === undefined ? current.rootPath : rootPath.trim()
    if (!nextName || !nextRoot) {
      throw new Error('bucket 名称或目录不能为空')
    }
    const clash = this.findBucketByName(nextName)
    if (clash && clash.id !== id) {
      throw new Error(`bucket 名称已存在: ${nextName}`)
    }

    const updated: Bucket = { ...current, name: nextName, rootPath: nextRoot, updatedAt: nowIso() }
    this.buckets.set(id, updated)
    return this.withCount(updated)
  }

  private deleteBucket(id: number): void {
    if (!this.buckets.delete(id)) {
      throw new Error(`bucket 不存在: ${id}`)
    }
  }

  // ---- 资产 ----

  private assetsFor(bucketName: string): LearningAsset[] {
    const prefix = `/${bucketName}/`
    return [...this.assets.values()].filter(asset => asset.id.startsWith(prefix))
  }

  private list(query: AssetQuery): LearningAsset[] {
    const buckets = query.bucket
      ? [this.requireBucketName(query.bucket)]
      : [...this.buckets.values()]

    const assets = buckets.flatMap(bucket => this.assetsFor(bucket.name))
    const sorted = [...assets]

    if (query.sortBy === 'name') {
      sorted.sort((left, right) => left.name.localeCompare(right.name, 'zh-CN'))
    } else if (query.sortBy === 'size') {
      sorted.sort((left, right) => right.size - left.size)
    } else {
      sorted.sort((left, right) => right.addedAt.localeCompare(left.addedAt))
    }

    return sorted.map(cloneAsset)
  }

  private requireBucketName(name: string): Bucket {
    const bucket = this.findBucketByName(name)
    if (!bucket) {
      throw new Error(`bucket 不存在: ${name}`)
    }
    return bucket
  }

  private upload(input: Partial<UploadAssetRequest>): LearningAsset {
    const name = String(input.name ?? '').trim()
    const contentBase64 = String(input.contentBase64 ?? '')
    if (!name || !contentBase64) {
      throw new Error('资料上传参数不完整')
    }
    const bucket = this.requireBucketName(String(input.bucket ?? ''))

    const extension = extensionOf(name)
    const kind = kindOf(extension)
    const mimeType = String(input.mimeType ?? '') || mimeTypeOf(extension)
    const asset: LearningAsset = {
      id: `/${bucket.name}/${name}`,
      name,
      extension,
      typeLabel: typeLabelOf(kind),
      kind,
      size: Number(input.size ?? 0),
      mimeType,
      addedAt: nowIso(),
      url: dataUrl(mimeType, contentBase64),
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

  private uploadImage(input: Partial<UploadImageRequest>): UploadedImage {
    const name = String(input.name ?? '').trim()
    const mimeType = String(input.mimeType ?? '')
    const contentBase64 = String(input.contentBase64 ?? '')
    if (!name || !mimeType.startsWith('image/') || !contentBase64) {
      throw new Error('图片上传参数不完整')
    }

    const bucket = [...this.buckets.values()].sort((left, right) => left.id - right.id)[0]
    if (!bucket) {
      throw new Error('尚未配置任何 bucket，无法上传图片')
    }

    const extension = extensionOf(name)
    const asset: LearningAsset = {
      id: `/${bucket.name}/images/${name}`,
      name,
      extension,
      typeLabel: '图片',
      kind: 'image',
      size: contentBase64.length,
      mimeType,
      addedAt: nowIso(),
      url: dataUrl(mimeType, contentBase64),
    }
    this.assets.set(asset.id, asset)

    return { name, url: asset.url ?? '' }
  }
}
