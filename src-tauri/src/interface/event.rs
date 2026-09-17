//! 轮次进度事件；一轮对话对应一个 `Channel<AgentEvent>`。

use serde::Serialize;

use super::error::ApiError;
use super::session::{MessageInfo, ToolCall, TurnStatus};

/// 内部标签枚举：序列化后形如 `{ "type": "message-delta", "sessionId": 1, ... }`。
#[derive(Serialize, Clone, Debug)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase"
)]
pub enum AgentEvent {
    /// 轮次开始。
    TurnStarted { session_id: i64, turn_id: String },
    /// 助手文本增量（仅无工具请求的流式路径）。
    MessageDelta {
        session_id: i64,
        turn_id: String,
        message_id: i64,
        delta: String,
    },
    /// 一条消息定稿。
    MessageCompleted {
        session_id: i64,
        turn_id: String,
        message: MessageInfo,
    },
    /// 模型发起工具调用。
    ToolCall {
        session_id: i64,
        turn_id: String,
        call: ToolCall,
    },
    /// 权限为 `ask`，等待用户裁决。
    ToolApprovalRequired {
        session_id: i64,
        turn_id: String,
        call_id: String,
        tool_id: String,
        arguments: serde_json::Value,
    },
    /// 工具需要用户补充输入，等待回答。
    UserInputRequired {
        session_id: i64,
        turn_id: String,
        call_id: String,
        tool_id: String,
        question: String,
        options: Vec<String>,
    },
    /// 工具执行结束。
    ToolResult {
        session_id: i64,
        turn_id: String,
        call_id: String,
        tool_id: String,
        ok: bool,
        result: Option<serde_json::Value>,
        error: Option<ApiError>,
    },
    /// 轮次结束。
    TurnEnded {
        session_id: i64,
        turn_id: String,
        status: TurnStatus,
        error: Option<ApiError>,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn events_serialize_with_kebab_tag_and_camel_fields() {
        let event = AgentEvent::MessageDelta {
            session_id: 7,
            turn_id: "t-1".to_string(),
            message_id: 42,
            delta: "你好".to_string(),
        };

        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["type"], "message-delta");
        assert_eq!(value["sessionId"], 7);
        assert_eq!(value["turnId"], "t-1");
        assert_eq!(value["messageId"], 42);
        assert_eq!(value["delta"], "你好");

        let ended = AgentEvent::TurnEnded {
            session_id: 7,
            turn_id: "t-1".to_string(),
            status: TurnStatus::StepLimit,
            error: None,
        };
        let value = serde_json::to_value(&ended).unwrap();
        assert_eq!(value["type"], "turn-ended");
        assert_eq!(value["status"], "step_limit");
        assert!(value["error"].is_null());
    }

    #[test]
    fn user_input_required_serializes_question_and_options() {
        let event = AgentEvent::UserInputRequired {
            session_id: 7,
            turn_id: "t-1".to_string(),
            call_id: "call-1".to_string(),
            tool_id: "user.ask_question".to_string(),
            question: "你想学哪个？".to_string(),
            options: vec!["导数".to_string(), "积分".to_string()],
        };

        let value = serde_json::to_value(&event).unwrap();
        assert_eq!(value["type"], "user-input-required");
        assert_eq!(value["sessionId"], 7);
        assert_eq!(value["turnId"], "t-1");
        assert_eq!(value["callId"], "call-1");
        assert_eq!(value["toolId"], "user.ask_question");
        assert_eq!(value["question"], "你想学哪个？");
        assert_eq!(value["options"][0], "导数");
        assert_eq!(value["options"][1], "积分");
    }
}
