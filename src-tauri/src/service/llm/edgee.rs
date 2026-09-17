//! `EdgeeBackend`：DTO → edgee 类型映射、线名还原、重试与错误转换。
//!
//! edgee 只适合「无工具的流式」与「非流式」两条路径：带工具的请求一律走
//! `complete`，因为流式工具调用分片无法可靠组装。

use std::collections::HashMap;
use std::sync::RwLock;
use std::time::Duration;

use edgee::{
    Edgee, EdgeeConfig, FunctionCall, FunctionDefinition, InputObject, JsonSchema, Message, Role,
    Tool,
};
use tokio_stream::StreamExt;

use crate::config::{ConfigHandle, ProviderConfig};
use crate::interface::error::ApiError;
use crate::interface::llm::{
    ChatMessage, ChatRequest, ChatResponse, CompressionInfo, StreamEvent, TokenUsage,
};
use crate::interface::session::{MessageRole, ToolCall};

use super::{tool_id_to_wire, BoxFut, LlmBackend};

/// edgee 后端；按 `(base_url, api_key)` 指纹缓存客户端以复用连接池。
pub struct EdgeeBackend {
    config: ConfigHandle,
    clients: RwLock<HashMap<String, Edgee>>,
}

impl EdgeeBackend {
    pub fn new(config: ConfigHandle) -> Self {
        Self {
            config,
            clients: RwLock::new(HashMap::new()),
        }
    }

    fn client_for(&self, cfg: &ProviderConfig) -> Edgee {
        let fingerprint = format!("{}|{}", cfg.base_url, cfg.api_key);
        if let Ok(cache) = self.clients.read() {
            if let Some(client) = cache.get(&fingerprint) {
                return client.clone();
            }
        }

        let client =
            Edgee::new(EdgeeConfig::new(cfg.api_key.clone()).with_base_url(cfg.base_url.clone()));
        if let Ok(mut cache) = self.clients.write() {
            cache.insert(fingerprint, client.clone());
        }
        client
    }

    fn retries(&self) -> u32 {
        self.config.llm().max_retries
    }
}

impl LlmBackend for EdgeeBackend {
    fn complete<'a>(
        &'a self,
        cfg: &'a ProviderConfig,
        model: &'a str,
        request: &'a ChatRequest,
    ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
        Box::pin(async move {
            let client = self.client_for(cfg);
            let input = build_input(request, cfg);
            let max_retries = self.retries();
            let mut attempt = 0u32;

            loop {
                match client.send(model.to_string(), input.clone()).await {
                    Ok(response) => return Ok(convert_send_response(response, request)),
                    Err(error) => {
                        if attempt < max_retries && retryable(&error) {
                            tokio::time::sleep(backoff(attempt)).await;
                            attempt += 1;
                            continue;
                        }
                        return Err(map_edgee_error(error));
                    }
                }
            }
        })
    }

    fn stream<'a>(
        &'a self,
        cfg: &'a ProviderConfig,
        model: &'a str,
        request: &'a ChatRequest,
        sink: &'a mut (dyn FnMut(StreamEvent) + Send),
    ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
        Box::pin(async move {
            let client = self.client_for(cfg);
            let input = build_input(request, cfg);
            let max_retries = self.retries();
            let mut attempt = 0u32;

            'attempt: loop {
                let mut stream = match client.stream(model.to_string(), input.clone()).await {
                    Ok(stream) => stream,
                    Err(error) => {
                        if attempt < max_retries && retryable(&error) {
                            tokio::time::sleep(backoff(attempt)).await;
                            attempt += 1;
                            continue 'attempt;
                        }
                        return Err(map_edgee_error(error));
                    }
                };

                let mut content = String::new();
                let mut finish_reason = None;

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(chunk) => {
                            if let Some(text) = chunk.text() {
                                if !text.is_empty() {
                                    content.push_str(text);
                                    sink(StreamEvent::ContentDelta(text.to_string()));
                                }
                            }
                            if let Some(reason) = chunk.finish_reason() {
                                finish_reason = Some(reason.to_string());
                            }
                        }
                        // 已产生输出后不再重试，避免把重复文本推给前端。
                        Err(error) => {
                            if content.is_empty() && attempt < max_retries && retryable(&error) {
                                tokio::time::sleep(backoff(attempt)).await;
                                attempt += 1;
                                continue 'attempt;
                            }
                            return Err(map_edgee_error(error));
                        }
                    }
                }

                return Ok(ChatResponse {
                    content,
                    tool_calls: Vec::new(),
                    finish_reason,
                    usage: None,
                    compression: None,
                });
            }
        })
    }
}

fn backoff(attempt: u32) -> Duration {
    match attempt {
        0 => Duration::from_millis(500),
        _ => Duration::from_millis(1500),
    }
}

/// 可重试：传输层失败、408 / 429 / 5xx。
fn retryable(error: &edgee::Error) -> bool {
    match error {
        edgee::Error::Http(_) => true,
        edgee::Error::Api { status, .. } => *status == 408 || *status == 429 || *status >= 500,
        _ => false,
    }
}

fn map_edgee_error(error: edgee::Error) -> ApiError {
    match error {
        edgee::Error::MissingApiKey | edgee::Error::InvalidConfig(_) => {
            ApiError::llm_unconfigured(error.to_string())
        }
        _ => ApiError::llm_error(error.to_string()),
    }
}

fn build_input(request: &ChatRequest, cfg: &ProviderConfig) -> InputObject {
    let messages = request.messages.iter().map(to_edgee_message).collect();
    let mut input = InputObject::new(messages);

    if !request.tools.is_empty() {
        input = input.with_tools(request.tools.iter().map(to_edgee_tool).collect());
    }
    if let Some(model) = cfg
        .compression_model
        .as_deref()
        .map(str::trim)
        .filter(|model| !model.is_empty())
    {
        input = input.with_compression_model(model.to_string());
    }
    input
}

fn to_edgee_message(message: &ChatMessage) -> Message {
    match message.role {
        MessageRole::System => Message::system(message.content.clone()),
        MessageRole::User => Message::user(message.content.clone()),
        MessageRole::Assistant => Message {
            role: Role::Assistant,
            content: if message.content.is_empty() {
                None
            } else {
                Some(message.content.clone())
            },
            tool_calls: if message.tool_calls.is_empty() {
                None
            } else {
                Some(message.tool_calls.iter().map(to_edgee_tool_call).collect())
            },
            tool_call_id: None,
        },
        MessageRole::Tool => Message::tool(
            message.tool_call_id.clone().unwrap_or_default(),
            message.content.clone(),
        ),
    }
}

fn to_edgee_tool_call(call: &ToolCall) -> edgee::ToolCall {
    edgee::ToolCall {
        id: call.id.clone(),
        call_type: "function".to_string(),
        function: FunctionCall {
            name: tool_id_to_wire(&call.tool_id),
            arguments: call.arguments.to_string(),
        },
    }
}

fn to_edgee_tool(schema: &crate::interface::llm::ToolSchema) -> Tool {
    Tool::function(FunctionDefinition {
        name: schema.name.clone(),
        description: if schema.description.trim().is_empty() {
            None
        } else {
            Some(schema.description.clone())
        },
        parameters: to_json_schema(&schema.parameters),
    })
}

/// 只保留 edgee 支持的关键字：`type` / `properties` / `required` / `description`。
fn to_json_schema(value: &serde_json::Value) -> JsonSchema {
    let object = value.as_object();

    let schema_type = object
        .and_then(|obj| obj.get("type"))
        .and_then(|value| value.as_str())
        .unwrap_or("object")
        .to_string();
    let properties = object
        .and_then(|obj| obj.get("properties"))
        .and_then(|value| value.as_object())
        .map(|props| {
            props
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        });
    let required = object
        .and_then(|obj| obj.get("required"))
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        });
    let description = object
        .and_then(|obj| obj.get("description"))
        .and_then(|value| value.as_str())
        .map(String::from);

    JsonSchema {
        schema_type,
        properties,
        required,
        description,
    }
}

fn convert_send_response(response: edgee::SendResponse, request: &ChatRequest) -> ChatResponse {
    let wire_to_tool: HashMap<&str, &str> = request
        .tools
        .iter()
        .map(|schema| (schema.name.as_str(), schema.tool_id.as_str()))
        .collect();

    let message = response.message();
    let content = message
        .and_then(|message| message.content.clone())
        .unwrap_or_default();
    let tool_calls = message
        .and_then(|message| message.tool_calls.as_ref())
        .map(|calls| {
            calls
                .iter()
                .map(|call| convert_tool_call(call, &wire_to_tool))
                .collect()
        })
        .unwrap_or_default();

    ChatResponse {
        content,
        tool_calls,
        finish_reason: response.finish_reason().map(String::from),
        usage: response.usage.as_ref().map(|usage| TokenUsage {
            prompt_tokens: usage.prompt_tokens as i64,
            completion_tokens: usage.completion_tokens as i64,
            total_tokens: usage.total_tokens as i64,
        }),
        compression: response
            .compression
            .as_ref()
            .map(|compression| CompressionInfo {
                saved_tokens: compression.saved_tokens as i64,
                reduction: compression.reduction,
                time_ms: compression.time_ms as i64,
            }),
    }
}

fn convert_tool_call(call: &edgee::ToolCall, wire_to_tool: &HashMap<&str, &str>) -> ToolCall {
    let tool_id = wire_to_tool
        .get(call.function.name.as_str())
        .map(|tool_id| (*tool_id).to_string())
        .unwrap_or_else(|| call.function.name.replacen("__", ".", 1));
    let arguments = serde_json::from_str(&call.function.arguments)
        .unwrap_or_else(|_| serde_json::Value::String(call.function.arguments.clone()));

    ToolCall {
        id: call.id.clone(),
        tool_id,
        arguments,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, LlmConfig};
    use crate::interface::llm::ToolSchema;
    use crate::service::llm::wire_name;

    fn backend() -> EdgeeBackend {
        let mut app = AppConfig::default();
        app.llm = LlmConfig::default();
        EdgeeBackend::new(ConfigHandle::new(app))
    }

    fn provider_config(base_url: &str) -> ProviderConfig {
        ProviderConfig {
            base_url: base_url.to_string(),
            api_key: "sk-test".to_string(),
            model: "test-model".to_string(),
            compression_model: None,
        }
    }

    fn tool_schema() -> ToolSchema {
        ToolSchema {
            name: wire_name("anki", "list_decks"),
            tool_id: "anki.list_decks".to_string(),
            description: "列出牌组".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "additionalProperties": false,
                "properties": { "parent": { "type": "string" } },
                "required": ["parent"]
            }),
        }
    }

    #[tokio::test]
    async fn non_streaming_maps_request_and_response() {
        let mut server = mockito::Server::new_async().await;
        let body = serde_json::json!({
            "id": "1",
            "object": "chat.completion",
            "created": 1,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "答案是 1/3",
                    "tool_calls": [{
                        "id": "call-1",
                        "type": "function",
                        "function": {
                            "name": "anki__list_decks",
                            "arguments": "{\"parent\":null}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }],
            "usage": { "prompt_tokens": 10, "completion_tokens": 5, "total_tokens": 15 },
            "compression": { "saved_tokens": 3, "cost_savings": 1, "reduction": 20.0, "time_ms": 12 }
        })
        .to_string();

        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "model": "test-model",
                "stream": false,
                "messages": [{ "role": "user", "content": "hi" }],
                "tools": [{
                    "type": "function",
                    "function": {
                        "name": "anki__list_decks",
                        "parameters": {
                            "type": "object",
                            "properties": { "parent": { "type": "string" } },
                            "required": ["parent"]
                        }
                    }
                }]
            })))
            .with_status(200)
            .with_body(&body)
            .create_async()
            .await;

        let request = ChatRequest {
            messages: vec![ChatMessage::user("hi")],
            tools: vec![tool_schema()],
        };
        let response = backend()
            .complete(&provider_config(&server.url()), "test-model", &request)
            .await
            .unwrap();

        assert_eq!(response.content, "答案是 1/3");
        assert_eq!(response.finish_reason.as_deref(), Some("tool_calls"));
        assert_eq!(response.tool_calls.len(), 1);
        assert_eq!(response.tool_calls[0].tool_id, "anki.list_decks");
        assert_eq!(
            response.tool_calls[0].arguments["parent"],
            serde_json::Value::Null
        );
        assert_eq!(response.usage.unwrap().total_tokens, 15);
        assert_eq!(response.compression.unwrap().saved_tokens, 3);
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn streaming_accumulates_sse_chunks() {
        let mut server = mockito::Server::new_async().await;
        let body = concat!(
            "data: {\"id\":\"1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",",
            "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"你好\"},\"finish_reason\":null}]}\n\n",
            "data: {\"id\":\"1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"m\",",
            "\"choices\":[{\"index\":0,\"delta\":{\"content\":\"，世界\"},\"finish_reason\":\"stop\"}]}\n\n",
            "data: [DONE]\n\n"
        );

        let mock = server
            .mock("POST", "/v1/chat/completions")
            .match_body(mockito::Matcher::PartialJson(serde_json::json!({
                "model": "test-model",
                "stream": true
            })))
            .with_status(200)
            .with_body(body)
            .create_async()
            .await;

        let request = ChatRequest {
            messages: vec![ChatMessage::user("hi")],
            tools: Vec::new(),
        };
        let cfg = provider_config(&server.url());
        let backend = backend();

        let mut deltas = String::new();
        let response = {
            let mut sink = |event: StreamEvent| {
                let StreamEvent::ContentDelta(delta) = event;
                deltas.push_str(&delta);
            };
            backend
                .stream(&cfg, "test-model", &request, &mut sink)
                .await
                .unwrap()
        };

        assert_eq!(deltas, "你好，世界");
        assert_eq!(response.content, "你好，世界");
        assert_eq!(response.finish_reason.as_deref(), Some("stop"));
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn rate_limited_requests_are_retried() {
        let mut server = mockito::Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(429)
            .with_body("{}")
            .expect(3)
            .create_async()
            .await;

        let request = ChatRequest {
            messages: vec![ChatMessage::user("hi")],
            tools: Vec::new(),
        };
        let error = backend()
            .complete(&provider_config(&server.url()), "test-model", &request)
            .await
            .unwrap_err();

        assert_eq!(error.code, "llm_error");
        mock.assert_async().await;
    }

    #[test]
    fn wire_names_and_schema_subset_are_normalized() {
        assert_eq!(wire_name("anki", "list_decks"), "anki__list_decks");
        assert_eq!(tool_id_to_wire("anki.add_card"), "anki__add_card");
        assert_eq!(tool_id_to_wire("no-dot"), "no-dot");

        let schema = to_json_schema(&serde_json::json!({
            "type": "object",
            "additionalProperties": false,
            "properties": { "q": { "type": "string" } },
            "required": ["q"],
            "description": "搜索"
        }));
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.as_ref().unwrap().contains_key("q"));
        assert_eq!(schema.required.as_deref(), Some(&["q".to_string()][..]));
        assert_eq!(schema.description.as_deref(), Some("搜索"));

        // 缺省 type 补 object；不支持的字段被忽略。
        let schema = to_json_schema(&serde_json::json!({ "additionalProperties": false }));
        assert_eq!(schema.schema_type, "object");
        assert!(schema.properties.is_none());

        // id 内出现 `__` 时以请求内的映射表为准，不会歧义。
        let wire_to_tool: HashMap<&str, &str> =
            [("group__id__x", "group.id__x")].into_iter().collect();
        let call = edgee::ToolCall {
            id: "c".to_string(),
            call_type: "function".to_string(),
            function: FunctionCall {
                name: "group__id__x".to_string(),
                arguments: "not json".to_string(),
            },
        };
        let converted = convert_tool_call(&call, &wire_to_tool);
        assert_eq!(converted.tool_id, "group.id__x");
        assert_eq!(
            converted.arguments,
            serde_json::Value::String("not json".to_string())
        );
    }

    #[test]
    fn error_mapping_covers_edgee_variants() {
        assert_eq!(
            map_edgee_error(edgee::Error::MissingApiKey).code,
            "llm_unconfigured"
        );
        assert_eq!(
            map_edgee_error(edgee::Error::InvalidConfig("bad".to_string())).code,
            "llm_unconfigured"
        );
        assert_eq!(
            map_edgee_error(edgee::Error::Api {
                status: 401,
                message: "no".to_string()
            })
            .code,
            "llm_error"
        );
        assert_eq!(
            map_edgee_error(edgee::Error::Stream("boom".to_string())).code,
            "llm_error"
        );
    }
}
