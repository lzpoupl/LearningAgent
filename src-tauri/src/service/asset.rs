//! bucket 与资产业务：把 `FileEntry` 映射为 `LearningAsset`。
//!
//! 本层不感知文件系统差异，只使用 `/<bucket>/<relative-path>` 形式的资产 id。

use std::sync::{Arc, Mutex, MutexGuard};

use base64::engine::general_purpose::STANDARD;
use base64::Engine as _;

use crate::interface::asset::{
    AssetKind, AssetQuery, AssetSort, Bucket, LearningAsset, UploadAssetRequest,
    UploadImageRequest, UploadedImage,
};
use crate::interface::error::ApiError;
use crate::repository::fs::{self, FileEntry};
use crate::repository::bucket as bucket_repo;

/// bucket 与资产服务。
pub struct AssetService {
    db: Arc<Mutex<rusqlite::Connection>>,
}

impl AssetService {
    pub fn new(db: Arc<Mutex<rusqlite::Connection>>) -> Self {
        Self { db }
    }

    fn conn(&self) -> Result<MutexGuard<'_, rusqlite::Connection>, ApiError> {
        self.db
            .lock()
            .map_err(|_| ApiError::internal("数据库连接不可用"))
    }

    // ---------- bucket ----------

    pub fn list_buckets(&self) -> Result<Vec<Bucket>, ApiError> {
        let conn = self.conn()?;
        let mut buckets = bucket_repo::list_buckets(&conn)?;
        for bucket in &mut buckets {
            // 目录失效不应阻断整份列表，计数回退为 0。
            bucket.asset_count = fs::count(bucket).unwrap_or(0);
        }
        Ok(buckets)
    }

    pub fn create_bucket(&self, name: String, root_path: String) -> Result<Bucket, ApiError> {
        fs::validate_root(&root_path)?;
        let conn = self.conn()?;
        let mut bucket = bucket_repo::create_bucket(&conn, &name, &root_path)?;
        bucket.asset_count = fs::count(&bucket).unwrap_or(0);
        Ok(bucket)
    }

    pub fn update_bucket(
        &self,
        id: i64,
        name: Option<String>,
        root_path: Option<String>,
    ) -> Result<Bucket, ApiError> {
        if let Some(ref root) = root_path {
            fs::validate_root(root)?;
        }
        let conn = self.conn()?;
        let mut bucket = bucket_repo::update_bucket(&conn, id, name, root_path)?;
        bucket.asset_count = fs::count(&bucket).unwrap_or(0);
        Ok(bucket)
    }

    pub fn delete_bucket(&self, id: i64) -> Result<(), ApiError> {
        let conn = self.conn()?;
        bucket_repo::delete_bucket(&conn, id)
    }

    // ---------- 资产 ----------

    pub fn list_assets(&self, query: AssetQuery) -> Result<Vec<LearningAsset>, ApiError> {
        let conn = self.conn()?;
        let buckets = match &query.bucket {
            Some(name) => vec![Self::bucket_by_name(&conn, name)?],
            None => bucket_repo::list_buckets(&conn)?,
        };

        let mut assets = Vec::new();
        for bucket in &buckets {
            // 单个 bucket 目录不可用时跳过，不影响其它 bucket。
            let entries = fs::scan(bucket).unwrap_or_default();
            assets.extend(
                entries
                    .into_iter()
                    .map(|entry| asset_from_entry(bucket, entry)),
            );
        }

        sort_assets(&mut assets, query.sort_by);
        Ok(assets)
    }

    pub fn upload_asset(&self, input: UploadAssetRequest) -> Result<LearningAsset, ApiError> {
        let conn = self.conn()?;
        let bucket = Self::bucket_by_name(&conn, &input.bucket)?;

        let name = input.name.trim();
        if name.is_empty() {
            return Err(ApiError::invalid_input("资产名称不能为空"));
        }
        let relative = fs::normalize_relative(name)?;
        let content = decode_base64(&input.content_base64)?;

        fs::write(&bucket, &relative, &content)?;
        let entry = fs::stat(&bucket, &relative)?;
        Ok(asset_from_entry(&bucket, entry))
    }

    /// 返回资产的绝对路径，前端用 `convertFileSrc` 转为可访问 URL。
    pub fn get_asset_url(&self, asset_id: &str) -> Result<String, ApiError> {
        let conn = self.conn()?;
        let (bucket_name, relative) = parse_asset_id(asset_id)?;
        let bucket = Self::bucket_by_name(&conn, &bucket_name)?;
        let path = fs::resolve(&bucket, &relative)?;
        if !path.is_file() {
            return Err(ApiError::not_found(format!("资产不存在: {asset_id}")));
        }
        Ok(path.to_string_lossy().into_owned())
    }

    pub fn delete_asset(&self, asset_id: &str) -> Result<(), ApiError> {
        let conn = self.conn()?;
        let (bucket_name, relative) = parse_asset_id(asset_id)?;
        let bucket = Self::bucket_by_name(&conn, &bucket_name)?;
        fs::delete(&bucket, &relative)
    }

    /// 把图片写入默认 bucket 的 `images/` 目录，返回绝对路径。
    pub fn upload_image(&self, input: UploadImageRequest) -> Result<UploadedImage, ApiError> {
        let conn = self.conn()?;
        let bucket = bucket_repo::list_buckets(&conn)?
            .into_iter()
            .next()
            .ok_or_else(|| ApiError::not_found("尚未配置任何 bucket，无法上传图片"))?;

        let name = input.name.trim();
        if name.is_empty() {
            return Err(ApiError::invalid_input("图片名称不能为空"));
        }
        let relative = fs::normalize_relative(&format!("images/{name}"))?;
        let content = decode_base64(&input.content_base64)?;

        fs::write(&bucket, &relative, &content)?;
        let path = fs::resolve(&bucket, &relative)?;
        Ok(UploadedImage {
            name: name.to_string(),
            url: path.to_string_lossy().into_owned(),
        })
    }

    fn bucket_by_name(conn: &rusqlite::Connection, name: &str) -> Result<Bucket, ApiError> {
        bucket_repo::find_bucket_by_name(conn, name)?
            .ok_or_else(|| ApiError::not_found(format!("bucket 不存在: {name}")))
    }
}

/// 把 bucket 内文件映射为资产；`id` 为 `/<bucket>/<relative-path>`。
fn asset_from_entry(bucket: &Bucket, entry: FileEntry) -> LearningAsset {
    let kind = kind_of(&entry.extension);
    LearningAsset {
        id: format!("/{}/{}", bucket.name, entry.relative_path),
        extension: entry.extension.clone(),
        type_label: type_label_of(kind).to_string(),
        kind,
        size: entry.size,
        mime_type: mime_type_of(&entry.extension).to_string(),
        added_at: entry.added_at,
        name: entry.name,
        url: None,
    }
}

/// 解析 `/<bucket>/<relative-path>` 形式的资产 id。
fn parse_asset_id(asset_id: &str) -> Result<(String, String), ApiError> {
    let trimmed = asset_id.strip_prefix('/').unwrap_or(asset_id);
    match trimmed.split_once('/') {
        Some((bucket, rest)) if !bucket.is_empty() && !rest.is_empty() => {
            Ok((bucket.to_string(), rest.to_string()))
        }
        _ => Err(ApiError::invalid_input(format!(
            "资产 id 格式错误，应为 /<bucket>/<path>: {asset_id}"
        ))),
    }
}

fn decode_base64(raw: &str) -> Result<Vec<u8>, ApiError> {
    let trimmed = raw.trim();
    // 兼容前端直接传入 data URL 的情况。
    let payload = trimmed
        .split_once(',')
        .filter(|(head, _)| head.starts_with("data:"))
        .map(|(_, payload)| payload)
        .unwrap_or(trimmed);
    STANDARD
        .decode(payload)
        .map_err(|e| ApiError::invalid_input(format!("base64 解码失败: {e}")))
}

fn sort_assets(assets: &mut [LearningAsset], sort_by: Option<AssetSort>) {
    match sort_by.unwrap_or(AssetSort::Updated) {
        AssetSort::Name => assets.sort_by(|left, right| {
            left.name
                .to_lowercase()
                .cmp(&right.name.to_lowercase())
                .then_with(|| left.id.cmp(&right.id))
        }),
        AssetSort::Size => assets.sort_by(|left, right| {
            right
                .size
                .cmp(&left.size)
                .then_with(|| left.id.cmp(&right.id))
        }),
        AssetSort::Updated => assets.sort_by(|left, right| {
            right
                .added_at
                .cmp(&left.added_at)
                .then_with(|| left.id.cmp(&right.id))
        }),
    }
}

fn kind_of(extension: &str) -> AssetKind {
    match extension {
        "pdf" => AssetKind::Pdf,
        "ppt" | "pptx" => AssetKind::Slides,
        "md" | "markdown" | "txt" => AssetKind::Note,
        "png" | "jpg" | "jpeg" | "webp" | "gif" => AssetKind::Image,
        "doc" | "docx" => AssetKind::Document,
        _ => AssetKind::Other,
    }
}

fn type_label_of(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Pdf => "PDF 文档",
        AssetKind::Slides => "演示文稿",
        AssetKind::Note => "笔记",
        AssetKind::Image => "图片",
        AssetKind::Document => "文档",
        AssetKind::Other => "其他",
    }
}

fn mime_type_of(extension: &str) -> &'static str {
    match extension {
        "pdf" => "application/pdf",
        "ppt" => "application/vnd.ms-powerpoint",
        "pptx" => {
            "application/vnd.openxmlformats-officedocument.presentationml.presentation"
        }
        "md" | "markdown" => "text/markdown",
        "txt" => "text/plain",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "doc" => "application/msword",
        "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        _ => "application/octet-stream",
    }
}

#[cfg(test)]
mod tests {
    use std::fs as std_fs;
    use std::sync::Arc;

    use base64::engine::general_purpose::STANDARD;
    use tempfile::TempDir;

    use super::*;
    use crate::repository::db;

    fn service(dir: &TempDir) -> AssetService {
        let conn = db::open_in_memory().unwrap();
        let service = AssetService::new(Arc::new(Mutex::new(conn)));
        service
            .create_bucket("资料".into(), dir.path().to_string_lossy().into_owned())
            .unwrap();
        service
    }

    fn encode(content: &str) -> String {
        STANDARD.encode(content.as_bytes())
    }

    #[test]
    fn bucket_lifecycle_and_conflicts() {
        let dir = TempDir::new().unwrap();
        let service = service(&dir);

        let buckets = service.list_buckets().unwrap();
        assert_eq!(buckets.len(), 1);
        assert_eq!(buckets[0].name, "资料");
        assert_eq!(buckets[0].asset_count, 0);

        assert_eq!(
            service
                .create_bucket("资料".into(), dir.path().to_string_lossy().into_owned())
                .unwrap_err()
                .code,
            "conflict"
        );
        assert_eq!(
            service
                .create_bucket("坏的".into(), dir.path().join("missing").to_string_lossy().into_owned())
                .unwrap_err()
                .code,
            "invalid_path"
        );

        let renamed = service.update_bucket(buckets[0].id, Some("课程".into()), None).unwrap();
        assert_eq!(renamed.name, "课程");

        service.delete_bucket(buckets[0].id).unwrap();
        assert!(service.list_buckets().unwrap().is_empty());
    }

    #[test]
    fn list_assets_maps_extensions_and_sorts() {
        let dir = TempDir::new().unwrap();
        let service = service(&dir);

        std_fs::write(dir.path().join("课本.pdf"), "pdf").unwrap();
        std_fs::write(dir.path().join("讲义.pptx"), "ppt").unwrap();
        std_fs::write(dir.path().join("笔记.md"), "note").unwrap();
        std_fs::write(dir.path().join("图.png"), "img").unwrap();
        std_fs::write(dir.path().join("未知.xyz"), "other").unwrap();

        let mut assets = service.list_assets(AssetQuery::default()).unwrap();
        assert_eq!(assets.len(), 5);
        assets.sort_by(|left, right| left.name.cmp(&right.name));

        let note = assets.iter().find(|asset| asset.name == "笔记.md").unwrap();
        assert_eq!(note.kind, AssetKind::Note);
        assert_eq!(note.type_label, "笔记");
        assert_eq!(note.mime_type, "text/markdown");
        assert_eq!(note.id, "/资料/笔记.md");

        let unknown = assets.iter().find(|asset| asset.name == "未知.xyz").unwrap();
        assert_eq!(unknown.kind, AssetKind::Other);
        assert_eq!(unknown.type_label, "其他");
        assert_eq!(unknown.mime_type, "application/octet-stream");

        let pdf = assets.iter().find(|asset| asset.name == "课本.pdf").unwrap();
        assert_eq!(pdf.kind, AssetKind::Pdf);
        assert_eq!(pdf.mime_type, "application/pdf");

        // 按名称排序。
        let by_name = service
            .list_assets(AssetQuery {
                bucket: Some("资料".into()),
                sort_by: Some(AssetSort::Name),
            })
            .unwrap();
        assert_eq!(by_name.len(), 5);
        let names: Vec<&str> = by_name.iter().map(|asset| asset.name.as_str()).collect();
        let mut sorted = names.clone();
        sorted.sort_by_key(|name| name.to_lowercase());
        assert_eq!(names, sorted);

        assert_eq!(
            service
                .list_assets(AssetQuery {
                    bucket: Some("不存在".into()),
                    sort_by: None,
                })
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn upload_read_url_and_delete_assets() {
        let dir = TempDir::new().unwrap();
        let service = service(&dir);

        let asset = service
            .upload_asset(UploadAssetRequest {
                bucket: "资料".into(),
                name: "sub/新建笔记.md".into(),
                size: 0,
                mime_type: "text/markdown".into(),
                content_base64: encode("内容"),
            })
            .unwrap();
        assert_eq!(asset.id, "/资料/sub/新建笔记.md");
        assert_eq!(asset.kind, AssetKind::Note);

        let url = service.get_asset_url(&asset.id).unwrap();
        assert!(std_fs::metadata(&url).unwrap().is_file());

        service.delete_asset(&asset.id).unwrap();
        assert_eq!(
            service.get_asset_url(&asset.id).unwrap_err().code,
            "not_found"
        );

        assert_eq!(
            service
                .upload_asset(UploadAssetRequest {
                    bucket: "资料".into(),
                    name: "../逃逸.md".into(),
                    size: 0,
                    mime_type: String::new(),
                    content_base64: encode("x"),
                })
                .unwrap_err()
                .code,
            "invalid_path"
        );
        assert_eq!(
            service
                .upload_asset(UploadAssetRequest {
                    bucket: "资料".into(),
                    name: "坏.md".into(),
                    size: 0,
                    mime_type: String::new(),
                    content_base64: "not base64!!".into(),
                })
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(parse_asset_id("bad").unwrap_err().code, "invalid_input");
        assert!(parse_asset_id("/资料/a.md").is_ok());
    }

    #[test]
    fn upload_image_uses_default_bucket_images_dir() {
        let dir = TempDir::new().unwrap();
        let service = service(&dir);

        let image = service
            .upload_image(UploadImageRequest {
                name: "公式.png".into(),
                mime_type: "image/png".into(),
                content_base64: encode("png"),
            })
            .unwrap();
        assert_eq!(image.name, "公式.png");
        assert!(std_fs::metadata(&image.url).unwrap().is_file());
        assert!(image.url.contains("images"));

        let empty = AssetService::new(Arc::new(Mutex::new(db::open_in_memory().unwrap())));
        assert_eq!(
            empty
                .upload_image(UploadImageRequest {
                    name: "x.png".into(),
                    mime_type: "image/png".into(),
                    content_base64: encode("x"),
                })
                .unwrap_err()
                .code,
            "not_found"
        );
    }
}
