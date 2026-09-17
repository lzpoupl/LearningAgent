//! 工具引用与工具执行扩展点。
//!
//! 工具元数据由迁移播种并落在 `tool` 表，具体实现在 `service/tool` 下按工具组注册；
//! 尚未实现的工具（`asset.*` / `user.*`）继续返回 `tool_unavailable`。

pub mod anki;
mod args;

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use rusqlite::Connection;

use crate::config::ConfigHandle;
use crate::interface::agent::split_tool_id;
use crate::interface::error::ApiError;
use crate::repository::agent as agent_repo;

/// `<group>.<id>` 形式的工具引用。
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ToolKey {
    pub group: String,
    pub id: String,
}

impl ToolKey {
    /// 解析工具引用；缺少 `.` 或段为空返回 `invalid_input`。
    pub fn parse(raw: &str) -> Result<Self, ApiError> {
        let (group, id) = split_tool_id(raw)?;
        Ok(Self { group, id })
    }
}

impl fmt::Display for ToolKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.group, self.id)
    }
}

/// 工具执行上下文；同步、事务内调用，因此不跨 `.await` 持有。
#[allow(dead_code)]
pub struct ToolContext<'a> {
    pub conn: &'a Connection,
    pub agent_id: i64,
    pub session_id: i64,
    pub config: &'a ConfigHandle,
}

/// 工具执行结果；`content` 为回填给模型的 JSON 结果。
#[derive(Clone, Debug)]
pub struct ToolOutcome {
    pub content: serde_json::Value,
}

impl ToolOutcome {
    pub fn new(content: serde_json::Value) -> Self {
        Self { content }
    }
}

/// 工具执行契约。
pub trait Tool: Send + Sync {
    fn key(&self) -> ToolKey;

    /// 执行工具；返回 `Ok` 时结果回填给模型，返回 `Err` 时错误以工具结果形式回填。
    fn execute(
        &self,
        ctx: &ToolContext<'_>,
        arguments: serde_json::Value,
    ) -> Result<ToolOutcome, ApiError>;
}

/// 工具实现注册表；按工具引用索引已实现的工具。
#[derive(Default)]
pub struct ToolRegistry {
    tools: HashMap<ToolKey, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    /// 注册工具实现；同一引用后者覆盖前者。
    pub fn register(&mut self, tool: Arc<dyn Tool>) {
        self.tools.insert(tool.key(), tool);
    }

    pub fn get(&self, key: &ToolKey) -> Option<Arc<dyn Tool>> {
        self.tools.get(key).cloned()
    }

    /// 已注册的工具引用；顺序不固定，调用方按集合语义比较。
    pub fn keys(&self) -> impl Iterator<Item = &ToolKey> {
        self.tools.keys()
    }

    /// 执行工具；目录中存在但未注册实现时返回 `tool_unavailable`。
    pub fn execute(
        &self,
        key: &ToolKey,
        ctx: &ToolContext<'_>,
        arguments: serde_json::Value,
    ) -> Result<ToolOutcome, ApiError> {
        match self.tools.get(key) {
            Some(tool) => tool.execute(ctx, arguments),
            None => Err(ApiError::tool_unavailable(format!("工具尚未实现: {key}"))),
        }
    }
}

/// 校验「已注册的工具 ⊆ 工具目录」，防止代码实现与迁移播种漂移。
///
/// 只做单向校验：目录里允许存在尚未实现的工具（`asset.*` / `user.*`）。
pub fn validate_registry(registry: &ToolRegistry, conn: &Connection) -> Result<(), ApiError> {
    for key in registry.keys() {
        if agent_repo::get_tool(conn, &key.group, &key.id)?.is_none() {
            return Err(ApiError::internal(format!(
                "工具目录中不存在已注册的工具: {key}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_splits_on_first_dot() {
        let key = ToolKey::parse("anki.add_card").unwrap();
        assert_eq!(key.group, "anki");
        assert_eq!(key.id, "add_card");
        assert_eq!(key.to_string(), "anki.add_card");

        let nested = ToolKey::parse("group.id.with.dots").unwrap();
        assert_eq!(nested.group, "group");
        assert_eq!(nested.id, "id.with.dots");
    }

    #[test]
    fn parse_rejects_malformed_references() {
        for raw in ["", "nodot", ".leading", "trailing.", "."] {
            assert_eq!(
                ToolKey::parse(raw).unwrap_err().code,
                "invalid_input",
                "引用应被拒绝: {raw}"
            );
        }
    }

    #[test]
    fn empty_registry_has_no_tools() {
        let registry = ToolRegistry::new();
        assert!(registry.get(&ToolKey::parse("anki.add_card").unwrap()).is_none());
    }

    #[test]
    fn execute_reports_unregistered_tools() {
        let registry = ToolRegistry::new();
        let conn = crate::repository::db::open_in_memory().unwrap();
        let config = ConfigHandle::new(crate::config::AppConfig::default());
        let ctx = ToolContext {
            conn: &conn,
            agent_id: 1,
            session_id: 1,
            config: &config,
        };

        let err = registry
            .execute(
                &ToolKey::parse("anki.add_card").unwrap(),
                &ctx,
                serde_json::json!({}),
            )
            .unwrap_err();
        assert_eq!(err.code, "tool_unavailable");
    }

    struct StubTool {
        group: &'static str,
        id: &'static str,
    }

    impl Tool for StubTool {
        fn key(&self) -> ToolKey {
            ToolKey {
                group: self.group.to_string(),
                id: self.id.to_string(),
            }
        }

        fn execute(
            &self,
            _ctx: &ToolContext<'_>,
            _arguments: serde_json::Value,
        ) -> Result<ToolOutcome, ApiError> {
            Ok(ToolOutcome::new(serde_json::json!({})))
        }
    }

    #[test]
    fn keys_lists_registered_tools() {
        let mut registry = ToolRegistry::new();
        registry.register(Arc::new(StubTool {
            group: "anki",
            id: "list_decks",
        }));
        registry.register(Arc::new(StubTool {
            group: "asset",
            id: "read",
        }));

        let mut keys: Vec<String> = registry.keys().map(|key| key.to_string()).collect();
        keys.sort();
        assert_eq!(keys, vec!["anki.list_decks", "asset.read"]);
    }

    #[test]
    fn validate_registry_accepts_catalog_subset_and_rejects_unknown() {
        let conn = crate::repository::db::open_in_memory().unwrap();

        let mut known = ToolRegistry::new();
        known.register(Arc::new(StubTool {
            group: "anki",
            id: "list_decks",
        }));
        validate_registry(&known, &conn).unwrap();

        let mut unknown = ToolRegistry::new();
        unknown.register(Arc::new(StubTool {
            group: "anki",
            id: "nope",
        }));
        let err = validate_registry(&unknown, &conn).unwrap_err();
        assert_eq!(err.code, "internal");
    }
}
