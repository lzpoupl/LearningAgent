import { invoke } from '@tauri-apps/api/core'

import type {
  AssetQuery,
  Bucket,
  LearningAsset,
  UploadAssetRequest,
  UploadImageRequest,
  UploadedImage,
} from '../types/assets'

export function listBuckets(): Promise<Bucket[]> {
  return invoke<Bucket[]>('bucket_list')
}

export function createBucket(name: string, rootPath: string): Promise<Bucket> {
  return invoke<Bucket>('bucket_create', { name, rootPath })
}

export function updateBucket(id: number, name?: string, rootPath?: string): Promise<Bucket> {
  return invoke<Bucket>('bucket_update', { id, name, rootPath })
}

export function deleteBucket(id: number): Promise<void> {
  return invoke<void>('bucket_delete', { id })
}

export function listAssets(query: AssetQuery = {}): Promise<LearningAsset[]> {
  return invoke<LearningAsset[]>('asset_list', { query })
}

export function uploadAsset(input: UploadAssetRequest): Promise<LearningAsset> {
  return invoke<LearningAsset>('asset_upload', { input })
}

export function getAssetUrl(assetId: string): Promise<string> {
  return invoke<string>('asset_get_url', { assetId })
}

export function deleteAsset(assetId: string): Promise<void> {
  return invoke<void>('asset_delete', { assetId })
}

export function uploadImage(input: UploadImageRequest): Promise<UploadedImage> {
  return invoke<UploadedImage>('asset_upload_image', { input })
}
