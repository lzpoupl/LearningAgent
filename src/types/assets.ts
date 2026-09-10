export type AssetKind = 'pdf' | 'slides' | 'note' | 'image' | 'document'
export type AssetSort = 'updated' | 'name' | 'size'

export interface LearningAsset {
  id: string
  name: string
  extension: string
  typeLabel: string
  kind: AssetKind
  subject: string
  size: number
  mimeType: string
  addedAt: string
  url?: string
}

export interface AssetQuery {
  subject?: string
  sortBy?: AssetSort
}

export interface UploadAssetRequest {
  name: string
  subject: string
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
