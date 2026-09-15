//! 工具引用与工具执行扩展点。
//!
//! 本阶段只固定扩展点：工具元数据由迁移播种并落在 `tool` 表，注册表为空，
//! 后续阶段再在 `service/tool` 下实现具体工具并注册。

use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;

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

/// 工具执行契约。
pub trait Tool: Send + Sync {
    fn key(&self) -> ToolKey;
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
}
