//! bucket 与资产命令，仅做参数透传。

use crate::interface::asset::{
    AssetQuery, Bucket, LearningAsset, UploadAssetRequest, UploadImageRequest, UploadedImage,
};
use crate::interface::error::ApiError;
use crate::AppState;

#[tauri::command]
pub fn bucket_list(state: tauri::State<'_, AppState>) -> Result<Vec<Bucket>, ApiError> {
    state.asset.list_buckets()
}

#[tauri::command]
pub fn bucket_create(
    state: tauri::State<'_, AppState>,
    name: String,
    root_path: String,
) -> Result<Bucket, ApiError> {
    state.asset.create_bucket(name, root_path)
}

#[tauri::command]
pub fn bucket_update(
    state: tauri::State<'_, AppState>,
    id: i64,
    name: Option<String>,
    root_path: Option<String>,
) -> Result<Bucket, ApiError> {
    state.asset.update_bucket(id, name, root_path)
}

#[tauri::command]
pub fn bucket_delete(state: tauri::State<'_, AppState>, id: i64) -> Result<(), ApiError> {
    state.asset.delete_bucket(id)
}

#[tauri::command]
pub fn asset_list(
    state: tauri::State<'_, AppState>,
    query: AssetQuery,
) -> Result<Vec<LearningAsset>, ApiError> {
    state.asset.list_assets(query)
}

#[tauri::command]
pub fn asset_upload(
    state: tauri::State<'_, AppState>,
    input: UploadAssetRequest,
) -> Result<LearningAsset, ApiError> {
    state.asset.upload_asset(input)
}

#[tauri::command]
pub fn asset_get_url(
    state: tauri::State<'_, AppState>,
    asset_id: String,
) -> Result<String, ApiError> {
    state.asset.get_asset_url(&asset_id)
}

#[tauri::command]
pub fn asset_delete(state: tauri::State<'_, AppState>, asset_id: String) -> Result<(), ApiError> {
    state.asset.delete_asset(&asset_id)
}

#[tauri::command]
pub fn asset_upload_image(
    state: tauri::State<'_, AppState>,
    input: UploadImageRequest,
) -> Result<UploadedImage, ApiError> {
    state.asset.upload_image(input)
}
