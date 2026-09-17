//! 会话、消息与轮次 DTO。

use serde::{Deserialize, Serialize};

/// 消息角色；`system` 为会话创建时固化的系统提示词快照。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl MessageRole {
    /// 数据库中使用的字符串形式。
    pub fn as_str(self) -> &'static str {
        match self {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
        }
    }

    /// 解析数据库或外部输入中的角色字符串。
    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "system" => Some(MessageRole::System),
            "user" => Some(MessageRole::User),
            "assistant" => Some(MessageRole::Assistant),
            "tool" => Some(MessageRole::Tool),
            _ => None,
        }
    }
}

/// 消息状态：流式中 / 完成 / 出错 / 被中断。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MessageStatus {
    Streaming,
    Complete,
    Error,
    Interrupted,
}

impl MessageStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            MessageStatus::Streaming => "streaming",
            MessageStatus::Complete => "complete",
            MessageStatus::Error => "error",
            MessageStatus::Interrupted => "interrupted",
        }
    }

    pub fn parse(raw: &str) -> Option<Self> {
        match raw {
            "streaming" => Some(MessageStatus::Streaming),
            "complete" => Some(MessageStatus::Complete),
            "error" => Some(MessageStatus::Error),
            "interrupted" => Some(MessageStatus::Interrupted),
            _ => None,
        }
    }
}

/// 助手发起的一次工具调用。
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ToolCall {
    /// 模型侧调用 id，回填 tool 消息时使用。
    pub id: String,
    /// `<group>.<id>` 形式的工具引用。
    pub tool_id: String,
    pub arguments: serde_json::Value,
}

/// 一条持久化消息。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MessageInfo {
    pub id: i64,
    pub session_id: i64,
    /// 同一轮的 user/assistant/tool 共用；system 快照为 `None`。
    pub turn_id: Option<String>,
    pub role: MessageRole,
    /// 助手回答是 Markdown 文本，工具结果是一段 JSON 文本。
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub tool_call_id: Option<String>,
    /// 工具消息对应的 `<group>.<id>`。
    pub tool_name: Option<String>,
    pub status: MessageStatus,
    pub prompt_tokens: Option<i64>,
    pub completion_tokens: Option<i64>,
    pub created_at: String,
}

/// 会话概览。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionInfo {
    pub id: i64,
    pub agent_id: i64,
    pub title: String,
    /// 最近一轮使用的 provider 名；未调用过为 `None`。
    pub last_provider: Option<String>,
    /// 最近一轮使用的模型名；未调用过为 `None`。
    pub last_model: Option<String>,
    pub message_count: u32,
    pub last_message_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 会话 + 全部消息；前端打开会话时一次性读取。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SessionDetail {
    pub session: SessionInfo,
    pub messages: Vec<MessageInfo>,
}

/// 在既有会话上发送一条消息。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageInput {
    pub session_id: i64,
    pub content: String,
}

/// 新建会话并立即开始第一轮。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct StartSessionInput {
    pub agent_id: i64,
    pub content: String,
}

/// 一次用户输入的受理结果；后续内容通过命令传入的 Channel 推送。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TurnHandle {
    pub session_id: i64,
    pub turn_id: String,
}

/// 轮次的最终状态。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum TurnStatus {
    Completed,
    Cancelled,
    Failed,
    StepLimit,
}

/// 工具权限为 `ask` 时的用户裁决。
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    AllowOnce,
    AllowAlways,
    Deny,
}
