//! bucket 与学习资产的 DTO。

use serde::{Deserialize, Serialize};

/// 资产种类。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Pdf,
    Slides,
    Note,
    Image,
    Document,
    Other,
}

/// 资产排序方式。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AssetSort {
    Updated,
    Name,
    Size,
}

/// bucket 目录内文件在内存中的投影，不落库。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LearningAsset {
    pub id: String, // /<bucket>/<relative-path>
    pub name: String,
    pub extension: String,
    pub type_label: String,
    pub kind: AssetKind,
    pub size: i64,
    pub mime_type: String,
    pub added_at: String, // RFC3339
    pub url: Option<String>,
}

/// 资产查询条件。
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct AssetQuery {
    pub bucket: Option<String>, // 限定 bucket；None 表示全部
    pub sort_by: Option<AssetSort>,
}

/// 上传文件资产。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadAssetRequest {
    pub bucket: String,
    pub name: String,
    pub size: i64,
    pub mime_type: String,
    pub content_base64: String,
}

/// 上传图片。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadImageRequest {
    pub name: String,
    pub mime_type: String,
    pub content_base64: String,
}

/// 上传图片的返回结果。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UploadedImage {
    pub name: String,
    pub url: String,
}

/// 非结构化资产 bucket。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Bucket {
    pub id: i64,
    pub name: String,
    pub root_path: String,
    pub asset_count: u32,
    pub created_at: String,
    pub updated_at: String,
}
