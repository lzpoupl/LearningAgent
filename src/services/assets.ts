import { invoke } from '@tauri-apps/api/core'

import type {
  AssetQuery,
  LearningAsset,
  UploadAssetRequest,
} from '../types/assets'

export function listAssets(query: AssetQuery = {}): Promise<LearningAsset[]> {
  return invoke<LearningAsset[]>('asset_list', { query })
}

export function listAssetSubjects(): Promise<string[]> {
  return invoke<string[]>('asset_list_subjects')
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
