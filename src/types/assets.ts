export type AssetKind = 'pdf' | 'slides' | 'note' | 'image' | 'document' | 'other'
export type AssetSort = 'updated' | 'name' | 'size'

/** 资产 id 为 `/<bucket>/<relative-path>`。 */
export interface LearningAsset {
  id: string
  name: string
  extension: string
  typeLabel: string
  kind: AssetKind
  size: number
  mimeType: string
  addedAt: string
  url?: string
}

export interface AssetQuery {
  /** 限定 bucket；缺省表示全部。 */
  bucket?: string
  sortBy?: AssetSort
}

export interface UploadAssetRequest {
  bucket: string
  name: string
  size: number
  mimeType: string
  contentBase64: string
}

export interface UploadImageRequest {
  name: string
  mimeType: string
  contentBase64: string
}

export interface UploadedImage {
  name: string
  url: string
}

/** 非结构化资产 bucket：名称 -> 文件系统目录。 */
export interface Bucket {
  id: number
  name: string
  rootPath: string
  assetCount: number
  createdAt: string
  updatedAt: string
}
