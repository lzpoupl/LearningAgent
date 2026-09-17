//! 上下文构建：系统快照 + 历史消息 + 可用工具 → `ChatRequest`。

use rusqlite::Connection;

use crate::interface::agent::ToolPermission;
use crate::interface::error::ApiError;
use crate::interface::llm::{ChatMessage, ChatRequest, ToolSchema};
use crate::interface::session::MessageRole;
use crate::repository::{agent as agent_repo, session as session_repo};
use crate::service::llm::wire_name;
use crate::service::permission;
use crate::service::tool::ToolKey;

/// 系统快照为空时使用的兜底说明。
const DEFAULT_SYSTEM_PROMPT: &str = "你是学习助手，基于用户的提问与学习资产提供帮助。";

/// 把持久化状态翻译成一次模型请求。
pub fn build_request(
    conn: &Connection,
    agent_id: i64,
    session_id: i64,
) -> Result<ChatRequest, ApiError> {
    let stored = session_repo::list_messages(conn, session_id)?;

    let system_prompt = stored
        .iter()
        .find(|message| message.role == MessageRole::System)
        .map(|message| message.content.clone())
        .filter(|content| !content.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_SYSTEM_PROMPT.to_string());

    let mut messages = Vec::with_capacity(stored.len() + 1);
    messages.push(ChatMessage::system(system_prompt));
    for message in &stored {
        match message.role {
            // 系统快照已在最前，其余 system 记录不进入请求。
            MessageRole::System => continue,
            MessageRole::User => messages.push(ChatMessage::user(&message.content)),
            MessageRole::Assistant => messages.push(ChatMessage::assistant(
                &message.content,
                message.tool_calls.clone(),
            )),
            MessageRole::Tool => messages.push(ChatMessage::tool(
                message.tool_call_id.clone().unwrap_or_default(),
                &message.content,
            )),
        }
    }

    Ok(ChatRequest {
        messages,
        tools: available_tools(conn, agent_id)?,
    })
}

/// 生成该 Agent 可调用的工具集合，过滤掉权限为 `deny` 的项。
fn available_tools(conn: &Connection, agent_id: i64) -> Result<Vec<ToolSchema>, ApiError> {
    let tools = agent_repo::list_tools(conn)?;
    let mut schemas = Vec::new();

    for tool in tools {
        let key = ToolKey {
            group: tool.group.clone(),
            id: tool.id.clone(),
        };
        if permission::resolve(conn, agent_id, &key)? == ToolPermission::Deny {
            continue;
        }
        schemas.push(ToolSchema {
            name: wire_name(&tool.group, &tool.id),
            tool_id: format!("{}.{}", tool.group, tool.id),
            description: tool.description,
            parameters: normalize_parameters(&tool.parameters),
        });
    }

    Ok(schemas)
}

/// 把工具参数 Schema 规范化为 edgee 支持的子集：
/// `type` / `properties` / `required` / `description`，其余关键字忽略。
pub fn normalize_parameters(value: &serde_json::Value) -> serde_json::Value {
    let object = value.as_object();
    let mut normalized = serde_json::Map::new();

    let schema_type = object
        .and_then(|obj| obj.get("type"))
        .and_then(|value| value.as_str())
        .unwrap_or("object");
    normalized.insert("type".to_string(), serde_json::json!(schema_type));

    if let Some(properties) = object
        .and_then(|obj| obj.get("properties"))
        .and_then(|value| value.as_object())
    {
        normalized.insert(
            "properties".to_string(),
            serde_json::Value::Object(properties.clone()),
        );
    }
    if let Some(required) = object
        .and_then(|obj| obj.get("required"))
        .and_then(|value| value.as_array())
    {
        normalized.insert(
            "required".to_string(),
            serde_json::Value::Array(required.clone()),
        );
    }
    if let Some(description) = object
        .and_then(|obj| obj.get("description"))
        .and_then(|value| value.as_str())
    {
        normalized.insert("description".to_string(), serde_json::json!(description));
    }

    serde_json::Value::Object(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interface::session::ToolCall;
    use crate::repository::{db, session as repo};

    #[test]
    fn builds_system_history_and_tools() {
        let conn = db::open_in_memory().unwrap();
        let session = repo::insert_session(&conn, 1, "新会话").unwrap();

        repo::append_message(
            &conn,
            session.id,
            &repo::NewMessage::new(MessageRole::System, "你是数学助手"),
        )
        .unwrap();
        repo::append_message(
            &conn,
            session.id,
            &repo::NewMessage::new(MessageRole::User, "什么是导数？"),
        )
        .unwrap();
        repo::append_message(
            &conn,
            session.id,
            &repo::NewMessage::new(MessageRole::Assistant, "").with_tool_calls(vec![ToolCall {
                id: "call-1".to_string(),
                tool_id: "anki.list_decks".to_string(),
                arguments: serde_json::json!({}),
            }]),
        )
        .unwrap();
        repo::append_message(
            &conn,
            session.id,
            &repo::NewMessage::new(MessageRole::Tool, "{\"ok\":true}")
                .with_tool_result("call-1", "anki.list_decks"),
        )
        .unwrap();

        let request = build_request(&conn, 1, session.id).unwrap();

        assert_eq!(request.messages.len(), 4);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[0].content, "你是数学助手");
        assert_eq!(request.messages[1].role, MessageRole::User);
        assert_eq!(request.messages[2].role, MessageRole::Assistant);
        assert_eq!(request.messages[2].tool_calls.len(), 1);
        assert_eq!(request.messages[3].role, MessageRole::Tool);
        assert_eq!(request.messages[3].tool_call_id.as_deref(), Some("call-1"));

        // 内置 Agent 1 的 list_decks 为 allow，add_card 为 ask，两者都可用。
        assert!(request
            .tools
            .iter()
            .any(|tool| tool.tool_id == "anki.list_decks" && tool.name == "anki__list_decks"));
        assert!(request
            .tools
            .iter()
            .any(|tool| tool.tool_id == "anki.add_card"));
    }

    #[test]
    fn denied_tools_are_excluded() {
        let conn = db::open_in_memory().unwrap();
        let session = repo::insert_session(&conn, 1, "新会话").unwrap();

        conn.execute("DELETE FROM agent_tool WHERE agent_id = 1", [])
            .unwrap();
        let request = build_request(&conn, 1, session.id).unwrap();
        assert!(request.tools.is_empty());

        // 恢复一条 allow 授权后只出现该工具。
        conn.execute(
            "INSERT INTO agent_tool (agent_id, tool_group, tool_id, permission, created_at, updated_at)
             VALUES (1, 'anki', 'list_decks', 'allow', datetime('now'), datetime('now'))",
            [],
        )
        .unwrap();
        let request = build_request(&conn, 1, session.id).unwrap();
        assert_eq!(request.tools.len(), 1);
        assert_eq!(request.tools[0].tool_id, "anki.list_decks");
    }

    #[test]
    fn empty_system_prompt_uses_default() {
        let conn = db::open_in_memory().unwrap();
        let session = repo::insert_session(&conn, 1, "新会话").unwrap();

        let request = build_request(&conn, 1, session.id).unwrap();
        assert_eq!(request.messages.len(), 1);
        assert_eq!(request.messages[0].role, MessageRole::System);
        assert_eq!(request.messages[0].content, DEFAULT_SYSTEM_PROMPT);
    }

    #[test]
    fn schema_normalization_keeps_supported_subset() {
        let normalized = normalize_parameters(&serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": { "q": { "type": "string" } },
            "required": ["q"],
            "description": "搜索"
        }));

        assert_eq!(normalized["type"], "object");
        assert_eq!(normalized["required"], serde_json::json!(["q"]));
        assert_eq!(normalized["description"], "搜索");
        assert!(normalized.get("additionalProperties").is_none());

        let default = normalize_parameters(&serde_json::json!({}));
        assert_eq!(default["type"], "object");
        assert!(default.get("properties").is_none());
    }
}
