//! 对话请求 / 响应、工具 Schema、流式事件与 provider 视图 DTO。

use serde::{Deserialize, Serialize};

use super::session::{MessageRole, ToolCall};

/// 发给模型的单条消息；与 edgee 的 `Message` 一一对应。
#[derive(Clone, Debug)]
pub struct ChatMessage {
    /// 只使用 System / User / Assistant / Tool。
    pub role: MessageRole,
    pub content: String,
    /// 仅 assistant 非空。
    pub tool_calls: Vec<ToolCall>,
    /// 仅 tool 非空。
    pub tool_call_id: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: None,
        }
    }

    pub fn assistant(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
            tool_calls,
            tool_call_id: None,
        }
    }

    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Tool,
            content: content.into(),
            tool_calls: Vec::new(),
            tool_call_id: Some(tool_call_id.into()),
        }
    }
}

/// 暴露给模型的工具定义。
#[derive(Clone, Debug)]
pub struct ToolSchema {
    /// 线名，见 `service::llm` 的线名映射。
    pub name: String,
    /// `<group>.<id>` 形式的工具引用。
    pub tool_id: String,
    pub description: String,
    /// JSON Schema（顶层 object + properties/required 子集）。
    pub parameters: serde_json::Value,
}

/// 一次调用的实际目标；由 `LlmClient::resolve` 依据会话记录与默认配置得出。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LlmTarget {
    pub provider: String,
    pub model: String,
}

/// 一次模型调用的内容部分；provider 与模型由 `LlmTarget` 决定。
#[derive(Clone, Debug)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
    pub tools: Vec<ToolSchema>,
}

/// 一次调用的 token 用量。
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TokenUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
    pub total_tokens: i64,
}

/// Edgee 网关返回的压缩指标；直连端点时为 `None`。
#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CompressionInfo {
    pub saved_tokens: i64,
    pub reduction: f64,
    pub time_ms: i64,
}

/// 一次模型调用的结果。
#[derive(Clone, Debug)]
pub struct ChatResponse {
    pub content: String,
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
    pub usage: Option<TokenUsage>,
    pub compression: Option<CompressionInfo>,
}

/// 流式回调事件；带工具的请求不走流式，因此只有文本增量。
#[derive(Clone, Debug)]
pub enum StreamEvent {
    ContentDelta(String),
}

/// 连通性探测结果。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ProbeResult {
    pub provider: String,
    pub model: String,
    pub latency_ms: u64,
    pub reply: String,
}

/// 面向前端的 provider 视图，密钥脱敏。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderView {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key_configured: bool,
    pub api_key_masked: String,
    pub compression_model: Option<String>,
}

/// 面向前端的 LLM 配置视图。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmConfigView {
    pub default_provider: String,
    pub max_steps: u32,
    pub allow_streaming: bool,
    pub providers: Vec<LlmProviderView>,
}

/// 更新 / 新增 provider；`api_key` 为 `None` 表示保留原密钥。
#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct LlmProviderInput {
    pub name: String,
    pub base_url: String,
    pub model: String,
    pub api_key: Option<String>,
    pub compression_model: Option<String>,
}
