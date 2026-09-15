//! Agent、工具与工具权限的 DTO。

use serde::{Deserialize, Serialize};

use super::error::ApiError;

/// 工具权限三级：允许 / 询问 / 拒绝。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ToolPermission {
    Allow,
    Ask,
    Deny,
}

impl ToolPermission {
    /// 数据库中使用的字符串形式。
    pub fn as_str(self) -> &'static str {
        match self {
            ToolPermission::Allow => "allow",
            ToolPermission::Ask => "ask",
            ToolPermission::Deny => "deny",
        }
    }

    /// 解析数据库或外部输入中的权限字符串。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "allow" => Some(ToolPermission::Allow),
            "ask" => Some(ToolPermission::Ask),
            "deny" => Some(ToolPermission::Deny),
            _ => None,
        }
    }
}

/// Agent 概览。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentInfo {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub builtin: bool,
}

/// 创建 / 更新输入；可选字段缺省表示沿用原值。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentConfigInput {
    pub name: String,
    pub description: String,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub system_prompt: Option<String>,
    /// 覆盖式设置该 Agent 的工具权限；`None` 表示不改动。
    pub tool_permissions: Option<Vec<AgentToolPermissionInput>>,
}

/// `tool_id` 为 `<group>.<id>` 形式的完整引用，如 `anki.add_card`。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolPermissionInput {
    pub tool_id: String,
    pub permission: ToolPermission,
}

/// 工具元数据。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolInfo {
    pub group: String,
    pub id: String,
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value, // JSON Schema
    pub returns: serde_json::Value,    // JSON Schema
    /// 新增授权时的默认级别，内置迁移播种时写入。
    pub default_permission: ToolPermission,
}

/// 某 Agent 对某工具的生效权限（工具元数据 + 级别）。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AgentToolPermission {
    pub tool: ToolInfo,
    pub permission: ToolPermission,
}

/// 工具组概览。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolGroup {
    pub name: String,
    pub tool_count: u32,
}

/// 解析 `<group>.<id>` 形式的工具引用，按第一个 `.` 切分。
///
/// 缺少 `.` 或任一段为空时返回 `invalid_input`；repository 与 service 共用该解析。
pub fn split_tool_id(raw: &str) -> Result<(String, String), ApiError> {
    match raw.split_once('.') {
        Some((group, id)) if !group.is_empty() && !id.is_empty() => {
            Ok((group.to_string(), id.to_string()))
        }
        _ => Err(ApiError::invalid_input(format!(
            "工具引用格式错误，应为 <group>.<id>: {raw}"
        ))),
    }
}
