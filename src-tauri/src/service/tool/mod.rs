//! 工具引用与工具执行扩展点。
//!
//! 本阶段只固定扩展点：工具元数据由迁移播种并落在 `tool` 表，注册表为空，
//! 后续阶段再在 `service/tool` 下实现具体工具并注册。

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

use rusqlite::Connection;

use crate::config::ConfigHandle;
use crate::interface::agent::split_tool_id;
use crate::interface::error::ApiError;

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

/// 工具实现注册表；本阶段为空。
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
}
