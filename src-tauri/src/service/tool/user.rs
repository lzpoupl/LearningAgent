//! `user.*` 工具实现：当前只有 `ask_question`，由 Agent 循环在权限通过后暂停等待回答。
//!
//! 该工具不直接执行：`user_prompt` 解析出提问内容与候选答案后交回循环，
//! 循环发出 `user-input-required` 并等待用户作答；`execute` 仅为防御性实现。

use std::sync::Arc;

use serde_json::Value;

use crate::interface::error::ApiError;

use super::args::required_str;
use super::{Tool, ToolContext, ToolKey, ToolOutcome, ToolRegistry, UserPrompt};

const GROUP: &str = "user";

/// 候选答案数量上限，超出视为参数非法。
const MAX_OPTIONS: usize = 6;

/// 本阶段实现的 user 工具 id 清单，与 `000006.sql` 中的契约一一对应。
pub const TOOL_IDS: [&str; 1] = ["ask_question"];

/// User 工具：一个实例对应一个工具 id。
pub struct UserTool {
    key: ToolKey,
}

impl UserTool {
    pub fn new(id: &str) -> Self {
        Self {
            key: ToolKey {
                group: GROUP.to_string(),
                id: id.to_string(),
            },
        }
    }
}

/// 解析候选答案：缺省为空，存在时必须是字符串数组，元素非空、去重、最多 6 项。
fn parse_options(args: &Value) -> Result<Vec<String>, ApiError> {
    let Some(value) = args.get("options").filter(|value| !value.is_null()) else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| ApiError::invalid_input("参数 options 应为字符串数组"))?;

    let mut options: Vec<String> = Vec::new();
    for item in array {
        let text = item
            .as_str()
            .ok_or_else(|| ApiError::invalid_input("参数 options 的元素应为字符串"))?
            .trim();
        if text.is_empty() {
            return Err(ApiError::invalid_input("参数 options 的元素不能为空"));
        }
        if !options.iter().any(|existing| existing == text) {
            options.push(text.to_string());
        }
    }
    if options.len() > MAX_OPTIONS {
        return Err(ApiError::invalid_input(format!(
            "参数 options 最多 {MAX_OPTIONS} 项"
        )));
    }
    Ok(options)
}

impl Tool for UserTool {
    fn key(&self) -> ToolKey {
        self.key.clone()
    }

    fn execute(
        &self,
        _ctx: &ToolContext<'_>,
        _arguments: Value,
    ) -> Result<ToolOutcome, ApiError> {
        Err(ApiError::internal(
            "user.ask_question 由 Agent 循环处理，不应直接执行",
        ))
    }

    fn user_prompt(&self, arguments: &Value) -> Result<Option<UserPrompt>, ApiError> {
        let question = required_str(arguments, "question")?.trim().to_string();
        if question.is_empty() {
            return Err(ApiError::invalid_input("参数 question 不能为空"));
        }
        Ok(Some(UserPrompt {
            question,
            options: parse_options(arguments)?,
        }))
    }
}

/// 注册全部 user 工具；应用启动时调用一次。
pub fn register(registry: &mut ToolRegistry) {
    for id in TOOL_IDS {
        registry.register(Arc::new(UserTool::new(id)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, ConfigHandle};
    use crate::repository::db;
    use serde_json::json;

    fn prompt(args: Value) -> Result<Option<UserPrompt>, ApiError> {
        UserTool::new("ask_question").user_prompt(&args)
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
    fn user_prompt_parses_question_and_options() {
        let parsed = prompt(json!({
            "question": " 你想学哪个？ ",
            "options": [" 导数 ", "积分", "导数"]
        }))
        .unwrap()
        .expect("应返回提问内容");
        assert_eq!(parsed.question, "你想学哪个？");
        assert_eq!(parsed.options, vec!["导数", "积分"]);

        let without_options = prompt(json!({ "question": "还有别的问题吗？" }))
            .unwrap()
            .unwrap();
        assert!(without_options.options.is_empty());

        // 显式 null 等同缺省。
        let null_options = prompt(json!({ "question": "q", "options": null }))
            .unwrap()
            .unwrap();
        assert!(null_options.options.is_empty());
        let empty_options = prompt(json!({ "question": "q", "options": [] }))
            .unwrap()
            .unwrap();
        assert!(empty_options.options.is_empty());
    }

    #[test]
    fn user_prompt_rejects_invalid_arguments() {
        assert_eq!(
            prompt(json!({})).unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": "   " })).unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": 1 })).unwrap_err().code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": "q", "options": "导数" }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": "q", "options": ["导数", 1] }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": "q", "options": ["导数", " "] }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        assert_eq!(
            prompt(json!({ "question": "q", "options": ["1", "2", "3", "4", "5", "6", "7"] }))
                .unwrap_err()
                .code,
            "invalid_input"
        );
        // 去重后恰好 6 项仍然合法。
        assert!(prompt(json!({
            "question": "q",
            "options": ["1", "2", "3", "4", "5", "6", "6"]
        }))
        .is_ok());
    }

    #[test]
    fn execute_is_defensive() {
        let conn = db::open_in_memory().unwrap();
        let config = ConfigHandle::new(AppConfig::default());
        let ctx = ToolContext {
            conn: &conn,
            agent_id: 1,
            session_id: 1,
            config: &config,
        };
        let err = UserTool::new("ask_question")
            .execute(&ctx, json!({ "question": "q" }))
            .unwrap_err();
        assert_eq!(err.code, "internal");
    }
}
