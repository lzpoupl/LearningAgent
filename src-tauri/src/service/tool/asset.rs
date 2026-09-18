//! `asset.*` 工具实现：列出、检索与读取非结构化学习资产。
//!
//! 工具执行发生在 `with_conn` 已持有连接锁的同步上下文中，因此这里只使用
//! `repository::bucket` 与 `repository::fs`，绝不调用会再次加锁的 `AssetService`。
//! 资产本体在文件系统上，工具不做内容索引，`search` 为实时扫描。
//! `ToolOutcome.content` 只放业务载荷，`ok` / `error` 信封由循环统一包装。

use std::sync::Arc;

use serde_json::{json, Value};

use crate::interface::asset::{AssetKind, Bucket, LearningAsset};
use crate::interface::error::ApiError;
use crate::repository::{bucket as bucket_repo, fs};
use crate::service::asset::{asset_from_entry, parse_asset_id};

use super::args::{limit, offset, optional_str, optional_u32, required_id, required_str};
use super::{Tool, ToolContext, ToolKey, ToolOutcome, ToolRegistry};

const GROUP: &str = "asset";

/// 本阶段实现的 asset 工具 id 清单，与 `000007.sql` 中的契约一一对应。
pub const TOOL_IDS: [&str; 3] = ["list", "search", "read"];

/// 可抽取正文的文本类扩展名白名单（小写，不含点）。
const TEXT_EXTENSIONS: &[&str] = &[
    "md", "markdown", "txt", "csv", "tsv", "json", "yaml", "yml", "toml", "log", "tex", "rst",
    "html", "htm", "xml", "ini", "cfg", "conf", "srt", "vtt",
];

/// 内容检索的单文件大小上限，超出只按文件名匹配。
const SEARCH_TEXT_MAX_BYTES: i64 = 512 * 1024;
/// `read` 整篇读取上限，超出返回 `unsupported_asset`。
const READ_MAX_BYTES: i64 = 4 * 1024 * 1024;
/// `read` 缺省返回字符数。
const READ_DEFAULT_CHARS: u32 = 20_000;
/// `read` 允许的最大返回字符数。
const READ_MAX_CHARS: u32 = 100_000;
/// 内容命中片段的最大字符数。
const SNIPPET_MAX_CHARS: usize = 120;
/// 单次 `search` 最多读取的文件数，控制内容检索成本。
const MAX_CONTENT_SCAN_FILES: usize = 1_000;

/// 一次扫描出的资产及其来源 bucket。
struct ScannedAsset {
    bucket: Bucket,
    asset: LearningAsset,
    size: i64,
}

/// Asset 工具：一个实例对应一个工具 id。
pub struct AssetTool {
    key: ToolKey,
}

impl AssetTool {
    pub fn new(id: &str) -> Self {
        Self {
            key: ToolKey {
                group: GROUP.to_string(),
                id: id.to_string(),
            },
        }
    }

    fn list(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let bucket = optional_str(args, "bucket")?;
        let kind = parse_kind_arg(args)?;
        let limit = limit(args)?;
        let offset = offset(args)?;

        let assets: Vec<LearningAsset> = collect_assets(ctx, bucket.as_deref())?
            .into_iter()
            .filter(|scanned| kind_matches(kind, scanned.asset.kind))
            .map(|scanned| scanned.asset)
            .collect();

        let start = offset as usize;
        let has_more = assets.len() > start.saturating_add(limit as usize);
        let page: Vec<LearningAsset> = assets
            .into_iter()
            .skip(start)
            .take(limit as usize)
            .collect();
        let count = page.len();
        Ok(json!({ "assets": page, "count": count, "hasMore": has_more }))
    }

    fn search(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let keyword = required_str(args, "keyword")?;
        let keyword = keyword.trim().to_string();
        if keyword.is_empty() {
            return Err(ApiError::invalid_input("参数 keyword 不能为空"));
        }
        let keyword_lower = keyword.to_lowercase();
        let bucket = optional_str(args, "bucket")?;
        let kind = parse_kind_arg(args)?;
        let limit = limit(args)?;
        let offset = offset(args)?;

        let candidates: Vec<ScannedAsset> = collect_assets(ctx, bucket.as_deref())?
            .into_iter()
            .filter(|scanned| kind_matches(kind, scanned.asset.kind))
            .collect();

        let start = offset as usize;
        // 多收集一条用于判断 hasMore，分页后再截断。
        let needed = start.saturating_add(limit as usize).saturating_add(1);

        let mut matches: Vec<Value> = Vec::new();
        let mut content_scans = 0usize;
        for item in candidates {
            if matches.len() >= needed {
                break;
            }

            if item.asset.name.to_lowercase().contains(&keyword_lower) {
                matches.push(json!({
                    "asset": item.asset,
                    "matchedIn": "name",
                    "snippet": Value::Null,
                }));
                continue;
            }

            // 内容匹配；达到读取上限后只保留文件名匹配。
            if content_scans >= MAX_CONTENT_SCAN_FILES {
                continue;
            }
            if !is_text_extension(&item.asset.extension) || item.size > SEARCH_TEXT_MAX_BYTES {
                continue;
            }
            let Ok((_, relative)) = parse_asset_id(&item.asset.id) else {
                continue;
            };
            content_scans += 1;
            let Ok(bytes) = fs::read(&item.bucket, &relative) else {
                continue;
            };
            let Ok(text) = String::from_utf8(bytes) else {
                continue;
            };
            if !text.to_lowercase().contains(&keyword_lower) {
                continue;
            }

            let snippet = snippet_of(&text, &keyword_lower);
            matches.push(json!({
                "asset": item.asset,
                "matchedIn": "content",
                "snippet": snippet,
            }));
        }

        let has_more = matches.len() > start.saturating_add(limit as usize);
        let page: Vec<Value> = matches
            .into_iter()
            .skip(start)
            .take(limit as usize)
            .collect();
        let count = page.len();
        Ok(json!({ "matches": page, "count": count, "hasMore": has_more }))
    }

    fn read(&self, ctx: &ToolContext<'_>, args: &Value) -> Result<Value, ApiError> {
        let asset_id = required_id(args, "assetId")?;
        let offset = offset(args)? as usize;
        let limit = optional_u32(args, "limit")?
            .unwrap_or(READ_DEFAULT_CHARS)
            .min(READ_MAX_CHARS) as usize;

        let (bucket_name, relative) = parse_asset_id(&asset_id)?;
        let bucket = bucket_repo::find_bucket_by_name(ctx.conn, &bucket_name)?
            .ok_or_else(|| ApiError::not_found(format!("bucket 不存在: {bucket_name}")))?;
        let entry = fs::stat(&bucket, &relative)?;
        if !is_text_extension(&entry.extension) || entry.size > READ_MAX_BYTES {
            return Err(ApiError::unsupported_asset(format!(
                "资产不支持文本读取: {asset_id}"
            )));
        }

        let bytes = fs::read(&bucket, &relative)?;
        let text = String::from_utf8(bytes).map_err(|_| {
            ApiError::unsupported_asset(format!("资产不是有效 UTF-8 文本: {asset_id}"))
        })?;

        let total_chars = text.chars().count();
        let content: String = text.chars().skip(offset).take(limit).collect();
        let returned_chars = content.chars().count();
        let truncated = offset.saturating_add(returned_chars) < total_chars;

        Ok(json!({
            "asset": asset_from_entry(&bucket, entry),
            "content": content,
            "offset": offset,
            "returnedChars": returned_chars,
            "totalChars": total_chars,
            "truncated": truncated,
        }))
    }
}

impl Tool for AssetTool {
    fn key(&self) -> ToolKey {
        self.key.clone()
    }

    fn execute(&self, ctx: &ToolContext<'_>, arguments: Value) -> Result<ToolOutcome, ApiError> {
        let content = match self.key.id.as_str() {
            "list" => self.list(ctx, &arguments)?,
            "search" => self.search(ctx, &arguments)?,
            "read" => self.read(ctx, &arguments)?,
            other => {
                return Err(ApiError::tool_unavailable(format!(
                    "工具尚未实现: asset.{other}"
                )))
            }
        };
        Ok(ToolOutcome::new(content))
    }
}

/// 注册全部 asset 工具；应用启动时调用一次。
pub fn register(registry: &mut ToolRegistry) {
    for id in TOOL_IDS {
        registry.register(Arc::new(AssetTool::new(id)));
    }
}

/// 收集资产：指定 bucket 时缺失返回 `not_found`，否则跨全部 bucket。
/// 排序与 `service::asset` 默认排序一致：`added_at` 降序、`id` 升序。
fn collect_assets(
    ctx: &ToolContext<'_>,
    bucket_filter: Option<&str>,
) -> Result<Vec<ScannedAsset>, ApiError> {
    let buckets = match bucket_filter {
        Some(name) => vec![bucket_repo::find_bucket_by_name(ctx.conn, name)?
            .ok_or_else(|| ApiError::not_found(format!("bucket 不存在: {name}")))?],
        None => bucket_repo::list_buckets(ctx.conn)?,
    };

    let mut assets = Vec::new();
    for bucket in buckets {
        // 单个 bucket 目录失效时跳过，不影响其它 bucket。
        let entries = fs::scan(&bucket).unwrap_or_default();
        assets.extend(entries.into_iter().map(|entry| {
            let asset = asset_from_entry(&bucket, entry);
            ScannedAsset {
                size: asset.size,
                asset,
                bucket: bucket.clone(),
            }
        }));
    }

    assets.sort_by(|left, right| {
        right
            .asset
            .added_at
            .cmp(&left.asset.added_at)
            .then_with(|| left.asset.id.cmp(&right.asset.id))
    });
    Ok(assets)
}

/// 解析可选 `kind` 参数。
fn parse_kind_arg(args: &Value) -> Result<Option<AssetKind>, ApiError> {
    match optional_str(args, "kind")? {
        Some(raw) => Ok(Some(parse_kind(&raw)?)),
        None => Ok(None),
    }
}

fn parse_kind(raw: &str) -> Result<AssetKind, ApiError> {
    match raw {
        "pdf" => Ok(AssetKind::Pdf),
        "slides" => Ok(AssetKind::Slides),
        "note" => Ok(AssetKind::Note),
        "image" => Ok(AssetKind::Image),
        "document" => Ok(AssetKind::Document),
        "other" => Ok(AssetKind::Other),
        other => Err(ApiError::invalid_input(format!("参数 kind 取值无效: {other}"))),
    }
}

fn kind_matches(filter: Option<AssetKind>, kind: AssetKind) -> bool {
    match filter {
        Some(want) => want == kind,
        None => true,
    }
}

fn is_text_extension(extension: &str) -> bool {
    TEXT_EXTENSIONS.contains(&extension)
}

/// 取首个命中行，`trim` 后截断到 `SNIPPET_MAX_CHARS`；跨行命中时回退整段文本。
fn snippet_of(text: &str, keyword_lower: &str) -> String {
    let line = text
        .lines()
        .find(|line| line.to_lowercase().contains(keyword_lower))
        .unwrap_or(text);
    truncate_chars(line.trim(), SNIPPET_MAX_CHARS)
}

/// 按字符截断，超出时以省略号收尾。
fn truncate_chars(text: &str, max: usize) -> String {
    let mut chars = text.chars();
    let truncated: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{truncated}…")
    } else {
        truncated
    }
}

#[cfg(test)]
mod tests {
    use std::fs as std_fs;

    use tempfile::TempDir;

    use super::*;
    use crate::config::{AppConfig, ConfigHandle};
    use crate::repository::db;

    fn config() -> ConfigHandle {
        ConfigHandle::new(AppConfig::default())
    }

    fn call(
        tool: &AssetTool,
        conn: &rusqlite::Connection,
        config: &ConfigHandle,
        args: Value,
    ) -> Result<Value, ApiError> {
        let ctx = ToolContext {
            conn,
            agent_id: 1,
            session_id: 1,
            config,
        };
        tool.execute(&ctx, args).map(|outcome| outcome.content)
    }

    fn run(
        id: &str,
        conn: &rusqlite::Connection,
        config: &ConfigHandle,
        args: Value,
    ) -> Result<Value, ApiError> {
        let tool = AssetTool::new(id);
        call(&tool, conn, config, args)
    }

    fn add_bucket(conn: &rusqlite::Connection, name: &str, dir: &TempDir) {
        bucket_repo::create_bucket(conn, name, &dir.path().to_string_lossy()).unwrap();
    }

    fn write_file(dir: &TempDir, relative: &str, content: &str) {
        let path = dir.path().join(relative);
        if let Some(parent) = path.parent() {
            std_fs::create_dir_all(parent).unwrap();
        }
        std_fs::write(path, content).unwrap();
    }

    #[test]
    fn register_covers_all_tool_ids() {
        let mut registry = ToolRegistry::new();
        register(&mut registry);

        let mut ids: Vec<&str> = registry
            .keys()
            .filter(|key| key.group == GROUP)
            .map(|key| key.id.as_str())
            .collect();
        ids.sort_unstable();

        let mut expected = TOOL_IDS.to_vec();
        expected.sort_unstable();
        assert_eq!(ids, expected);
    }

    #[test]
    fn unknown_id_reports_unavailable() {
        let conn = db::open_in_memory().unwrap();
        let tool = AssetTool::new("not_a_tool");
        let err = call(&tool, &conn, &config(), json!({})).unwrap_err();
        assert_eq!(err.code, "tool_unavailable");
    }

    #[test]
    fn list_returns_all_and_filters_by_bucket_and_kind() {
        let notes = TempDir::new().unwrap();
        let slides = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &notes);
        add_bucket(&conn, "课件", &slides);

        write_file(&notes, "笔记.md", "笔记");
        write_file(&notes, "课本.pdf", "pdf");
        write_file(&notes, "未知.xyz", "other");
        write_file(&slides, "讲义.pptx", "ppt");

        let all = run("list", &conn, &config, json!({})).unwrap();
        assert_eq!(all["count"], 4);
        assert_eq!(all["hasMore"], false);

        let scoped = run("list", &conn, &config, json!({ "bucket": "资料" })).unwrap();
        assert_eq!(scoped["count"], 3);

        let only_notes = run("list", &conn, &config, json!({ "kind": "note" })).unwrap();
        assert_eq!(only_notes["count"], 1);
        assert_eq!(only_notes["assets"][0]["id"], "/资料/笔记.md");
        assert_eq!(only_notes["assets"][0]["url"], Value::Null);

        let only_other = run("list", &conn, &config, json!({ "kind": "other" })).unwrap();
        assert_eq!(only_other["count"], 1);
        assert_eq!(only_other["assets"][0]["name"], "未知.xyz");

        assert_eq!(
            run("list", &conn, &config, json!({ "kind": "nope" }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            run("list", &conn, &config, json!({ "bucket": "缺失" }))
                .unwrap_err()
                .code,
            "not_found"
        );
    }

    #[test]
    fn list_pages_and_clamps_limit() {
        let dir = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &dir);
        for index in 0..205 {
            write_file(&dir, &format!("file{index}.txt"), "x");
        }

        let clamped = run("list", &conn, &config, json!({ "limit": 1000 })).unwrap();
        assert_eq!(clamped["count"], 200);
        assert_eq!(clamped["hasMore"], true);

        let page = run("list", &conn, &config, json!({ "limit": 10, "offset": 200 })).unwrap();
        assert_eq!(page["count"], 5);
        assert_eq!(page["hasMore"], false);
    }

    #[test]
    fn search_matches_name_and_content() {
        let dir = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &dir);

        write_file(&dir, "关键词笔记.md", "无关正文");
        write_file(&dir, "其他.md", "第一行\n这里有关键词\n第三行");
        write_file(&dir, "说明.pdf", "这里也有关键词");

        let by_name = run("search", &conn, &config, json!({ "keyword": "关键词笔记" })).unwrap();
        assert_eq!(by_name["count"], 1);
        assert_eq!(by_name["matches"][0]["matchedIn"], "name");
        assert_eq!(by_name["matches"][0]["snippet"], Value::Null);

        // 两个文件正文命中：.md 抽取片段，.pdf 不读正文。
        let by_content = run("search", &conn, &config, json!({ "keyword": "关键词" })).unwrap();
        assert_eq!(by_content["count"], 2);
        let md = by_content["matches"]
            .as_array()
            .unwrap()
            .iter()
            .find(|item| item["asset"]["extension"] == "md")
            .unwrap();
        assert_eq!(md["matchedIn"], "content");
        assert_eq!(md["snippet"], "这里有关键词");
    }

    #[test]
    fn search_is_case_insensitive_and_scoped() {
        let notes = TempDir::new().unwrap();
        let slides = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &notes);
        add_bucket(&conn, "课件", &slides);

        write_file(&notes, "Key.txt", "no hit");
        write_file(&notes, "Key.pdf", "no hit");
        write_file(&slides, "key.txt", "no hit");

        let all = run("search", &conn, &config, json!({ "keyword": "KEY" })).unwrap();
        assert_eq!(all["count"], 3);

        let scoped = run(
            "search",
            &conn,
            &config,
            json!({ "keyword": "key", "bucket": "资料" }),
        )
        .unwrap();
        assert_eq!(scoped["count"], 2);

        let kind_scoped = run(
            "search",
            &conn,
            &config,
            json!({ "keyword": "key", "kind": "pdf" }),
        )
        .unwrap();
        assert_eq!(kind_scoped["count"], 1);
        assert_eq!(kind_scoped["matches"][0]["asset"]["id"], "/资料/Key.pdf");

        assert_eq!(
            run("search", &conn, &config, json!({}))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            run("search", &conn, &config, json!({ "keyword": "   " }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
    }

    #[test]
    fn search_pages_with_offset_and_has_more() {
        let dir = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &dir);
        for index in 0..5 {
            write_file(&dir, &format!("kw{index}.txt"), "x");
        }

        let first = run("search", &conn, &config, json!({ "keyword": "kw", "limit": 2 })).unwrap();
        assert_eq!(first["count"], 2);
        assert_eq!(first["hasMore"], true);

        let last = run(
            "search",
            &conn,
            &config,
            json!({ "keyword": "kw", "limit": 2, "offset": 4 }),
        )
        .unwrap();
        assert_eq!(last["count"], 1);
        assert_eq!(last["hasMore"], false);
    }

    #[test]
    fn read_returns_full_text_and_pages_by_chars() {
        let dir = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &dir);
        write_file(&dir, "note.md", "0123456789");

        let full = run("read", &conn, &config, json!({ "assetId": "/资料/note.md" })).unwrap();
        assert_eq!(full["content"], "0123456789");
        assert_eq!(full["totalChars"], 10);
        assert_eq!(full["returnedChars"], 10);
        assert_eq!(full["truncated"], false);
        assert_eq!(full["asset"]["id"], "/资料/note.md");

        let page = run(
            "read",
            &conn,
            &config,
            json!({ "assetId": "/资料/note.md", "offset": 2, "limit": 3 }),
        )
        .unwrap();
        assert_eq!(page["content"], "234");
        assert_eq!(page["returnedChars"], 3);
        assert_eq!(page["truncated"], true);

        // offset 超出总长返回空内容，不报错。
        let past = run(
            "read",
            &conn,
            &config,
            json!({ "assetId": "/资料/note.md", "offset": 99 }),
        )
        .unwrap();
        assert_eq!(past["content"], "");
        assert_eq!(past["truncated"], false);
    }

    #[test]
    fn read_rejects_non_text_and_reports_missing() {
        let dir = TempDir::new().unwrap();
        let conn = db::open_in_memory().unwrap();
        let config = config();
        add_bucket(&conn, "资料", &dir);
        write_file(&dir, "课本.pdf", "pdf");
        std_fs::write(dir.path().join("坏.md"), [0xff, 0xfe, 0x00]).unwrap();

        assert_eq!(
            run("read", &conn, &config, json!({ "assetId": "/资料/课本.pdf" }))
                .unwrap_err()
                .code,
            "unsupported_asset"
        );
        assert_eq!(
            run("read", &conn, &config, json!({ "assetId": "/资料/坏.md" }))
                .unwrap_err()
                .code,
            "unsupported_asset"
        );
        assert_eq!(
            run("read", &conn, &config, json!({ "assetId": "/资料/不存在.md" }))
                .unwrap_err()
                .code,
            "not_found"
        );
        assert_eq!(
            run("read", &conn, &config, json!({ "assetId": "bad" }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            run("read", &conn, &config, json!({}))
                .unwrap_err()
                .code,
            "invalid_input"
        );
    }
}
