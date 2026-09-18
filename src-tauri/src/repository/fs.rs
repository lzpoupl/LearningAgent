//! 跨平台文件系统适配层：唯一接触 `std::fs` / `std::path` 的模块。
//!
//! 上层只使用「bucket + 以 `/` 分隔的相对路径」这一层语义；Windows 与 Linux 在
//! 分隔符、盘符、大小写、创建时间可用性、`\\?\` 前缀与符号链接上的差异全部收敛在这里。

use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use chrono::{DateTime, Utc};

use crate::interface::asset::Bucket;
use crate::interface::error::ApiError;

/// 递归扫描的最大深度，防止异常目录结构拖垮界面。
const MAX_DEPTH: usize = 32;
/// 单次扫描收集的最大条目数。
const MAX_ENTRIES: usize = 20_000;

/// bucket 内一个文件在内存中的元数据投影。
#[derive(Clone, Debug)]
pub struct FileEntry {
    /// 统一以 `/` 分隔的相对路径。
    pub relative_path: String,
    pub name: String,
    /// 小写扩展名（不含点），无扩展名时为空串。
    pub extension: String,
    pub size: i64,
    /// RFC3339：优先创建时间，回退修改时间，再兜底当前时间。
    pub added_at: String,
}

/// 校验 bucket 根目录存在且为目录，返回规范化后的根路径。
pub fn validate_root(root: &str) -> Result<PathBuf, ApiError> {
    if root.trim().is_empty() {
        return Err(ApiError::invalid_path("bucket 目录不能为空"));
    }
    let path = PathBuf::from(root);
    let metadata = fs::metadata(&path)
        .map_err(|e| ApiError::invalid_path(format!("bucket 目录不可用: {root} ({e})")))?;
    if !metadata.is_dir() {
        return Err(ApiError::invalid_path(format!("bucket 根不是目录: {root}")));
    }
    let canonical = fs::canonicalize(&path)
        .map_err(|e| ApiError::invalid_path(format!("bucket 目录无法解析: {root} ({e})")))?;
    Ok(strip_verbatim(canonical))
}

/// 规范化相对路径：同时接受 `/` 与 `\`，统一为 `/`，并拒绝绝对路径、`..` 与非法文件名。
pub fn normalize_relative(raw: &str) -> Result<String, ApiError> {
    if raw.starts_with('/') || raw.starts_with('\\') {
        return Err(ApiError::invalid_path(format!(
            "相对路径不能是绝对路径: {raw}"
        )));
    }

    let mut segments: Vec<String> = Vec::new();
    for segment in raw.split(['/', '\\']) {
        if segment.is_empty() || segment == "." {
            continue;
        }
        if segment == ".." {
            return Err(ApiError::invalid_path(format!(
                "相对路径不允许包含 ..: {raw}"
            )));
        }
        validate_segment(segment, raw)?;
        segments.push(segment.to_string());
    }

    if segments.is_empty() {
        return Err(ApiError::invalid_path(format!("相对路径为空: {raw}")));
    }
    Ok(segments.join("/"))
}

/// 把 bucket 内相对路径解析为平台原生绝对路径，并确认仍在 bucket 根目录内。
pub fn resolve(bucket: &Bucket, relative: &str) -> Result<PathBuf, ApiError> {
    let normalized = normalize_relative(relative)?;
    let root = validate_root(&bucket.root_path)?;

    let mut path = root.clone();
    for segment in normalized.split('/') {
        path.push(segment);
    }

    ensure_within(&root, &path)?;
    Ok(path)
}

/// 由绝对路径反推 `/` 分隔的相对路径。
pub fn to_relative(root: &Path, path: &Path) -> Result<String, ApiError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| ApiError::invalid_path(format!("路径不在 bucket 内: {}", path.display())))?;

    let mut parts: Vec<String> = Vec::new();
    for component in relative.components() {
        match component {
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
            _ => {
                return Err(ApiError::invalid_path(format!(
                    "非法路径: {}",
                    path.display()
                )))
            }
        }
    }

    if parts.is_empty() {
        return Err(ApiError::invalid_path("相对路径为空".to_string()));
    }
    Ok(parts.join("/"))
}

/// 递归扫描 bucket，只收集常规文件，忽略隐藏项且不进入符号链接。
pub fn scan(bucket: &Bucket) -> Result<Vec<FileEntry>, ApiError> {
    let root = validate_root(&bucket.root_path)?;
    let mut entries = Vec::new();
    walk(&root, &root, 0, &mut entries);
    Ok(entries)
}

/// 读取单个文件在 bucket 内的元数据。
pub fn stat(bucket: &Bucket, relative: &str) -> Result<FileEntry, ApiError> {
    let root = validate_root(&bucket.root_path)?;
    let path = resolve(bucket, relative)?;
    let metadata = fs::metadata(&path)
        .map_err(|_| ApiError::not_found(format!("资产不存在: {relative}")))?;
    if !metadata.is_file() {
        return Err(ApiError::invalid_path(format!("资产不是文件: {relative}")));
    }

    let name = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .ok_or_else(|| ApiError::invalid_path(format!("非法路径: {relative}")))?;

    Ok(FileEntry {
        relative_path: to_relative(&root, &path)?,
        extension: extension_of(&name),
        name,
        size: metadata.len() as i64,
        added_at: added_at(&metadata),
    })
}

/// 统计 bucket 内常规文件数量。
pub fn count(bucket: &Bucket) -> Result<u32, ApiError> {
    Ok(scan(bucket)?.len() as u32)
}

/// 读取 bucket 内文件内容。
pub fn read(bucket: &Bucket, relative: &str) -> Result<Vec<u8>, ApiError> {
    let path = resolve(bucket, relative)?;
    fs::read(&path).map_err(|e| match e.kind() {
        std::io::ErrorKind::NotFound => ApiError::not_found(format!("资产不存在: {relative}")),
        _ => ApiError::internal(format!("读取资产失败: {relative} ({e})")),
    })
}

/// 写入 bucket 内文件，必要时创建父目录。
pub fn write(bucket: &Bucket, relative: &str, content: &[u8]) -> Result<(), ApiError> {
    let path = resolve(bucket, relative)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            ApiError::invalid_path(format!("无法创建目录: {} ({e})", parent.display()))
        })?;
    }
    fs::write(&path, content).map_err(|e| ApiError::internal(format!("写入资产失败: {relative} ({e})")))
}

/// 删除 bucket 内的单个文件；不会删除目录。
pub fn delete(bucket: &Bucket, relative: &str) -> Result<(), ApiError> {
    let path = resolve(bucket, relative)?;
    let metadata = fs::metadata(&path)
        .map_err(|_| ApiError::not_found(format!("资产不存在: {relative}")))?;
    if metadata.is_dir() {
        return Err(ApiError::invalid_path(format!("资产不是文件: {relative}")));
    }
    fs::remove_file(&path).map_err(|e| ApiError::internal(format!("删除资产失败: {relative} ({e})")))
}

fn walk(root: &Path, dir: &Path, depth: usize, out: &mut Vec<FileEntry>) {
    if depth > MAX_DEPTH || out.len() >= MAX_ENTRIES {
        return;
    }

    let read_dir = match fs::read_dir(dir) {
        Ok(read_dir) => read_dir,
        // 单个目录读取失败时跳过，保证整体扫描不中断。
        Err(_) => return,
    };

    for entry in read_dir.flatten() {
        if out.len() >= MAX_ENTRIES {
            return;
        }

        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }

        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(_) => continue,
        };
        if file_type.is_symlink() {
            // 不进入符号链接，避免目录环与越权访问。
            continue;
        }

        let path = entry.path();
        if file_type.is_dir() {
            walk(root, &path, depth + 1, out);
        } else if file_type.is_file() {
            if let Some(file) = file_entry(root, &path, &name) {
                out.push(file);
            }
        }
    }
}

fn file_entry(root: &Path, path: &Path, name: &str) -> Option<FileEntry> {
    let metadata = fs::metadata(path).ok()?;
    let relative_path = to_relative(root, path).ok()?;
    Some(FileEntry {
        relative_path,
        name: name.to_string(),
        extension: extension_of(name),
        size: metadata.len() as i64,
        added_at: added_at(&metadata),
    })
}

fn extension_of(name: &str) -> String {
    Path::new(name)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or("")
        .to_lowercase()
}

fn added_at(metadata: &fs::Metadata) -> String {
    let timestamp = metadata
        .created()
        .or_else(|_| metadata.modified())
        .unwrap_or_else(|_| SystemTime::now());
    DateTime::<Utc>::from(timestamp).to_rfc3339()
}

/// 找到目标或其最近的已存在祖先，并返回规范化结果。
fn existing_anchor(path: &Path) -> Option<PathBuf> {
    if let Ok(canonical) = fs::canonicalize(path) {
        return Some(strip_verbatim(canonical));
    }
    let mut current = path.parent();
    while let Some(dir) = current {
        if let Ok(canonical) = fs::canonicalize(dir) {
            return Some(strip_verbatim(canonical));
        }
        current = dir.parent();
    }
    None
}

fn ensure_within(root: &Path, path: &Path) -> Result<(), ApiError> {
    let anchor = existing_anchor(path)
        .ok_or_else(|| ApiError::invalid_path(format!("路径不可用: {}", path.display())))?;
    if is_within(root, &anchor) {
        Ok(())
    } else {
        Err(ApiError::invalid_path(format!(
            "路径越出 bucket 根目录: {}",
            path.display()
        )))
    }
}

#[cfg(windows)]
fn is_within(root: &Path, path: &Path) -> bool {
    // Windows 路径大小写不敏感，逐段折叠大小写比较。
    let mut root_components = root.components();
    let mut path_components = path.components();
    loop {
        match (root_components.next(), path_components.next()) {
            (None, _) => return true,
            (Some(left), Some(right)) => {
                let left = left.as_os_str().to_string_lossy().to_lowercase();
                let right = right.as_os_str().to_string_lossy().to_lowercase();
                if left != right {
                    return false;
                }
            }
            (Some(_), None) => return false,
        }
    }
}

#[cfg(not(windows))]
fn is_within(root: &Path, path: &Path) -> bool {
    path.starts_with(root)
}

/// 去掉 `canonicalize` 在 Windows 上返回的 `\\?\` verbatim 前缀。
#[cfg(windows)]
fn strip_verbatim(path: PathBuf) -> PathBuf {
    let raw = path.to_string_lossy();
    if let Some(rest) = raw.strip_prefix(r"\\?\UNC\") {
        PathBuf::from(format!(r"\\{rest}"))
    } else if let Some(rest) = raw.strip_prefix(r"\\?\") {
        PathBuf::from(rest)
    } else {
        path
    }
}

#[cfg(not(windows))]
fn strip_verbatim(path: PathBuf) -> PathBuf {
    path
}

fn validate_segment(segment: &str, raw: &str) -> Result<(), ApiError> {
    if segment.contains(':') {
        return Err(ApiError::invalid_path(format!(
            "路径段包含非法字符 ':': {raw}"
        )));
    }
    if segment.chars().any(|c| c.is_control()) {
        return Err(ApiError::invalid_path(format!("路径段包含控制字符: {raw}")));
    }

    #[cfg(windows)]
    {
        const ILLEGAL: [char; 7] = ['<', '>', '"', '|', '?', '*', '\\'];
        if segment.chars().any(|c| ILLEGAL.contains(&c)) {
            return Err(ApiError::invalid_path(format!(
                "文件名包含 Windows 保留字符: {raw}"
            )));
        }
        if segment.ends_with('.') || segment.ends_with(' ') {
            return Err(ApiError::invalid_path(format!(
                "文件名不能以点或空格结尾: {raw}"
            )));
        }
        const RESERVED: [&str; 22] = [
            "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
            "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
        ];
        let stem = segment
            .split('.')
            .next()
            .unwrap_or(segment)
            .to_ascii_uppercase();
        if RESERVED.contains(&stem.as_str()) {
            return Err(ApiError::invalid_path(format!(
                "文件名是 Windows 保留名: {raw}"
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs as std_fs;

    use tempfile::TempDir;

    use super::*;

    fn bucket_at(dir: &TempDir) -> Bucket {
        Bucket {
            id: 1,
            name: "测试".into(),
            root_path: dir.path().to_string_lossy().into_owned(),
            asset_count: 0,
            created_at: String::new(),
            updated_at: String::new(),
        }
    }

    fn write_file(dir: &TempDir, relative: &str, content: &str) {
        let path = dir.path().join(relative);
        if let Some(parent) = path.parent() {
            std_fs::create_dir_all(parent).unwrap();
        }
        std_fs::write(path, content).unwrap();
    }

    #[test]
    fn normalize_relative_rejects_escape_and_normalizes_separators() {
        assert_eq!(normalize_relative("a/b/c.txt").unwrap(), "a/b/c.txt");
        assert_eq!(normalize_relative(r"a\b\c.txt").unwrap(), "a/b/c.txt");
        assert_eq!(normalize_relative("./a//b/").unwrap(), "a/b");
        assert_eq!(normalize_relative("中文/笔记.md").unwrap(), "中文/笔记.md");

        for raw in ["/abs/path", r"\abs\path", "C:/tmp", r"C:\tmp", "a/../b", "..", "", "/", "  "]
        {
            let code = normalize_relative(raw).unwrap_err().code;
            assert_eq!(code, "invalid_path", "输入应被拒绝: {raw}");
        }
        // 空白的路径段会被忽略，最终没有有效段时同样拒绝。
        assert_eq!(normalize_relative(" . / . ").unwrap_err().code, "invalid_path");
    }

    #[cfg(windows)]
    #[test]
    fn normalize_relative_rejects_windows_reserved_names() {
        for raw in ["CON", "nul.txt", "a/COM1", "bad?.txt", "trailing."] {
            assert_eq!(
                normalize_relative(raw).unwrap_err().code,
                "invalid_path",
                "输入应被拒绝: {raw}"
            );
        }
    }

    #[test]
    fn validate_root_requires_existing_directory() {
        let dir = TempDir::new().unwrap();
        let bucket = bucket_at(&dir);
        assert!(validate_root(&bucket.root_path).is_ok());

        write_file(&dir, "file.txt", "hello");
        let file_path = dir.path().join("file.txt");
        assert_eq!(
            validate_root(&file_path.to_string_lossy()).unwrap_err().code,
            "invalid_path"
        );

        let missing = dir.path().join("missing");
        assert_eq!(
            validate_root(&missing.to_string_lossy()).unwrap_err().code,
            "invalid_path"
        );
        assert_eq!(validate_root("  ").unwrap_err().code, "invalid_path");
    }

    #[test]
    fn resolve_and_to_relative_round_trip() {
        let dir = TempDir::new().unwrap();
        let bucket = bucket_at(&dir);
        write_file(&dir, "notes/a.md", "hi");

        let path = resolve(&bucket, "notes/a.md").unwrap();
        assert!(path.is_file());
        let root = validate_root(&bucket.root_path).unwrap();
        assert_eq!(to_relative(&root, &path).unwrap(), "notes/a.md");

        // 不存在的目标也能解析，方便写入新文件。
        assert!(resolve(&bucket, "new/b.txt").is_ok());
        assert_eq!(resolve(&bucket, "../escape").unwrap_err().code, "invalid_path");
    }

    #[test]
    fn scan_collects_files_with_metadata_and_skips_hidden() {
        let dir = TempDir::new().unwrap();
        let bucket = bucket_at(&dir);
        write_file(&dir, "a.pdf", "12345");
        write_file(&dir, "sub/note.MD", "hello");
        write_file(&dir, ".hidden", "secret");

        let mut entries = scan(&bucket).unwrap();
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].relative_path, "a.pdf");
        assert_eq!(entries[0].extension, "pdf");
        assert_eq!(entries[0].size, 5);
        assert!(!entries[0].added_at.is_empty());
        assert_eq!(entries[1].relative_path, "sub/note.MD");
        assert_eq!(entries[1].extension, "md");
        assert_eq!(entries[1].name, "note.MD");

        assert_eq!(count(&bucket).unwrap(), 2);
    }

    #[test]
    fn read_write_and_delete_round_trip() {
        let dir = TempDir::new().unwrap();
        let bucket = bucket_at(&dir);

        write(&bucket, "deep/nested/file.txt", b"content").unwrap();
        assert_eq!(read(&bucket, "deep/nested/file.txt").unwrap(), b"content");
        assert_eq!(stat(&bucket, "deep/nested/file.txt").unwrap().size, 7);

        delete(&bucket, "deep/nested/file.txt").unwrap();
        assert_eq!(
            read(&bucket, "deep/nested/file.txt").unwrap_err().code,
            "not_found"
        );
        assert_eq!(
            delete(&bucket, "deep/nested/file.txt").unwrap_err().code,
            "not_found"
        );
    }
}
