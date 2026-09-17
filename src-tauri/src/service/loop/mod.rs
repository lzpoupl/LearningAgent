//! Agent 循环：装配上下文 → 调用模型 → 执行工具 → 回填结果 → 循环至最终回答。
//!
//! 循环运行在 tokio 任务上，进度经 `EventEmitter`（生产环境为 Tauri Channel）
//! 推送；模型调用经 `LlmBackend` 抽象进入，因此可用桩后端完整测试。

pub mod context;
pub mod turn;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rusqlite::Connection;

use crate::config::ConfigHandle;
use crate::interface::agent::ToolPermission;
use crate::interface::error::ApiError;
use crate::interface::event::AgentEvent;
use crate::interface::llm::{ChatResponse, LlmTarget, StreamEvent};
use crate::interface::session::{
    ApprovalDecision, MessageInfo, MessageRole, MessageStatus, TurnStatus,
};
use crate::repository::{agent as agent_repo, session as session_repo};
use crate::service::event::EventEmitter;
use crate::service::llm::LlmClient;
use crate::service::permission;
use crate::service::tool::{ToolContext, ToolKey, ToolRegistry};
use crate::service::with_conn;

use context::build_request;
use turn::{TurnCommand, TurnControlHandle};

/// 一轮循环所需的共享依赖；由 `SessionService` 构造。
#[derive(Clone)]
pub struct TurnDeps {
    pub db: Arc<Mutex<Connection>>,
    pub tools: Arc<ToolRegistry>,
    pub llm: LlmClient,
    pub config: ConfigHandle,
}

/// 一轮的最终结果。
pub struct TurnOutcome {
    pub status: TurnStatus,
    pub error: Option<ApiError>,
}

impl TurnOutcome {
    fn completed() -> Self {
        Self {
            status: TurnStatus::Completed,
            error: None,
        }
    }

    fn cancelled() -> Self {
        Self {
            status: TurnStatus::Cancelled,
            error: None,
        }
    }

    fn failed(error: ApiError) -> Self {
        Self {
            status: TurnStatus::Failed,
            error: Some(error),
        }
    }

    fn step_limit() -> Self {
        Self {
            status: TurnStatus::StepLimit,
            error: None,
        }
    }
}

/// 执行一轮；结束时发出 `turn-ended`。
pub async fn run_turn(
    deps: TurnDeps,
    session_id: i64,
    agent_id: i64,
    turn_id: String,
    target: LlmTarget,
    mut ctl: TurnControlHandle,
    emitter: Arc<dyn EventEmitter>,
) -> TurnOutcome {
    let outcome = run_turn_inner(
        &deps, session_id, agent_id, &turn_id, &target, &mut ctl, &emitter,
    )
    .await;

    emitter.emit(AgentEvent::TurnEnded {
        session_id,
        turn_id,
        status: outcome.status,
        error: outcome.error.clone(),
    });
    outcome
}

async fn run_turn_inner(
    deps: &TurnDeps,
    session_id: i64,
    agent_id: i64,
    turn_id: &str,
    target: &LlmTarget,
    ctl: &mut TurnControlHandle,
    emitter: &Arc<dyn EventEmitter>,
) -> TurnOutcome {
    emitter.emit(AgentEvent::TurnStarted {
        session_id,
        turn_id: turn_id.to_string(),
    });

    let llm_config = deps.config.llm();
    let max_steps = llm_config.max_steps.max(1);

    for _ in 0..max_steps {
        if ctl.is_cancelled() {
            return TurnOutcome::cancelled();
        }

        let request = match with_conn(&deps.db, |conn| build_request(conn, agent_id, session_id)) {
            Ok(request) => request,
            Err(error) => return TurnOutcome::failed(error),
        };

        let streamed = request.tools.is_empty() && llm_config.allow_streaming;
        let response = if streamed {
            match call_streaming(deps, session_id, turn_id, target, &request, ctl, emitter).await {
                CallOutcome::Response(response) => response,
                CallOutcome::Cancelled => return TurnOutcome::cancelled(),
                CallOutcome::Failed(error) => return TurnOutcome::failed(error),
            }
        } else {
            match call_complete(deps, target, &request, ctl).await {
                CallOutcome::Response(response) => response,
                CallOutcome::Cancelled => return TurnOutcome::cancelled(),
                CallOutcome::Failed(error) => return TurnOutcome::failed(error),
            }
        };

        if !streamed {
            let message = match with_conn(&deps.db, |conn| {
                persist_assistant(conn, session_id, turn_id, &response)
            }) {
                Ok(message) => message,
                Err(error) => return TurnOutcome::failed(error),
            };
            emit_completed(emitter, session_id, turn_id, &message);
        }

        if response.tool_calls.is_empty() {
            return TurnOutcome::completed();
        }

        for call in response.tool_calls {
            if ctl.is_cancelled() {
                return TurnOutcome::cancelled();
            }
            emitter.emit(AgentEvent::ToolCall {
                session_id,
                turn_id: turn_id.to_string(),
                call: call.clone(),
            });

            let run = match dispatch_tool(deps, session_id, agent_id, turn_id, &call, ctl, emitter)
                .await
            {
                Dispatch::Run(run) => run,
                Dispatch::Cancelled => return TurnOutcome::cancelled(),
            };

            if let Err(error) = persist_tool_result(deps, session_id, turn_id, &call, &run) {
                return TurnOutcome::failed(error);
            }
            emitter.emit(AgentEvent::ToolResult {
                session_id,
                turn_id: turn_id.to_string(),
                call_id: call.id.clone(),
                tool_id: call.tool_id.clone(),
                ok: run.ok,
                result: run.result.clone(),
                error: run.error.clone(),
            });
        }
    }

    // 步数耗尽：留下说明消息，避免前端停在等待态。
    match with_conn(&deps.db, |conn| {
        session_repo::append_message(
            conn,
            session_id,
            &session_repo::NewMessage::new(
                MessageRole::Assistant,
                "已达到本轮工具调用上限，请继续追问。",
            )
            .with_turn_id(turn_id),
        )
    }) {
        Ok(message) => emit_completed(emitter, session_id, turn_id, &message),
        Err(error) => return TurnOutcome::failed(error),
    }
    TurnOutcome::step_limit()
}

fn emit_completed(
    emitter: &Arc<dyn EventEmitter>,
    session_id: i64,
    turn_id: &str,
    message: &MessageInfo,
) {
    emitter.emit(AgentEvent::MessageCompleted {
        session_id,
        turn_id: turn_id.to_string(),
        message: message.clone(),
    });
}

enum CallOutcome {
    Response(ChatResponse),
    Cancelled,
    Failed(ApiError),
}

async fn call_complete(
    deps: &TurnDeps,
    target: &LlmTarget,
    request: &crate::interface::llm::ChatRequest,
    ctl: &mut TurnControlHandle,
) -> CallOutcome {
    let result = tokio::select! {
        result = deps.llm.complete(target, request) => Some(result),
        _ = ctl.wait_cancel() => None,
    };

    match result {
        None => CallOutcome::Cancelled,
        Some(Err(error)) => CallOutcome::Failed(error),
        Some(Ok(response)) => CallOutcome::Response(response),
    }
}

async fn call_streaming(
    deps: &TurnDeps,
    session_id: i64,
    turn_id: &str,
    target: &LlmTarget,
    request: &crate::interface::llm::ChatRequest,
    ctl: &mut TurnControlHandle,
    emitter: &Arc<dyn EventEmitter>,
) -> CallOutcome {
    let assistant = match with_conn(&deps.db, |conn| {
        session_repo::begin_assistant_message(conn, session_id, turn_id)
    }) {
        Ok(message) => message,
        Err(error) => return CallOutcome::Failed(error),
    };

    let mut sink = StreamSink::new(
        deps.db.clone(),
        emitter.clone(),
        session_id,
        turn_id.to_string(),
        assistant.id,
    );

    let result = {
        let mut on_event = |event: StreamEvent| sink.handle(event);
        tokio::select! {
            result = deps.llm.stream(target, request, &mut on_event) => Some(result),
            _ = ctl.wait_cancel() => None,
        }
    };
    sink.finish();

    match result {
        None => {
            finish_streamed(
                deps,
                session_id,
                turn_id,
                assistant.id,
                &sink.content,
                MessageStatus::Interrupted,
                emitter,
            );
            CallOutcome::Cancelled
        }
        Some(Err(error)) => {
            finish_streamed(
                deps,
                session_id,
                turn_id,
                assistant.id,
                &sink.content,
                MessageStatus::Error,
                emitter,
            );
            CallOutcome::Failed(error)
        }
        Some(Ok(response)) => {
            let message = finish_streamed(
                deps,
                session_id,
                turn_id,
                assistant.id,
                &sink.content,
                MessageStatus::Complete,
                emitter,
            );
            match message {
                Some(_) => CallOutcome::Response(response),
                None => CallOutcome::Failed(ApiError::internal("流式消息无法定稿")),
            }
        }
    }
}

fn finish_streamed(
    deps: &TurnDeps,
    session_id: i64,
    turn_id: &str,
    message_id: i64,
    content: &str,
    status: MessageStatus,
    emitter: &Arc<dyn EventEmitter>,
) -> Option<MessageInfo> {
    match with_conn(&deps.db, |conn| {
        session_repo::finish_message(
            conn,
            message_id,
            &session_repo::MessagePatch {
                content: Some(content.to_string()),
                status: Some(status),
                ..session_repo::MessagePatch::default()
            },
        )
    }) {
        Ok(message) => {
            emit_completed(emitter, session_id, turn_id, &message);
            Some(message)
        }
        Err(error) => {
            eprintln!("流式消息定稿失败: {}", error.message);
            None
        }
    }
}

/// 流式增量：DB 按 200ms / 512 字符落库，IPC 按 50ms / 32 字符推送。
struct StreamSink {
    db: Arc<Mutex<Connection>>,
    emitter: Arc<dyn EventEmitter>,
    session_id: i64,
    turn_id: String,
    message_id: i64,
    content: String,
    pending_db: String,
    pending_ipc: String,
    last_db_flush: Instant,
    last_ipc_flush: Instant,
}

impl StreamSink {
    fn new(
        db: Arc<Mutex<Connection>>,
        emitter: Arc<dyn EventEmitter>,
        session_id: i64,
        turn_id: String,
        message_id: i64,
    ) -> Self {
        let now = Instant::now();
        Self {
            db,
            emitter,
            session_id,
            turn_id,
            message_id,
            content: String::new(),
            pending_db: String::new(),
            pending_ipc: String::new(),
            last_db_flush: now,
            last_ipc_flush: now,
        }
    }

    fn handle(&mut self, event: StreamEvent) {
        let StreamEvent::ContentDelta(delta) = event;
        self.content.push_str(&delta);
        self.pending_db.push_str(&delta);
        self.pending_ipc.push_str(&delta);

        if self.pending_db.len() >= 512
            || self.last_db_flush.elapsed() >= Duration::from_millis(200)
        {
            self.flush_db();
        }
        if self.pending_ipc.len() >= 32
            || self.last_ipc_flush.elapsed() >= Duration::from_millis(50)
        {
            self.flush_ipc();
        }
    }

    fn flush_db(&mut self) {
        if self.pending_db.is_empty() {
            return;
        }
        let delta = std::mem::take(&mut self.pending_db);
        let message_id = self.message_id;
        let result = with_conn(&self.db, |conn| {
            session_repo::append_content(conn, message_id, &delta)?;
            Ok(())
        });
        if result.is_ok() {
            self.last_db_flush = Instant::now();
        }
    }

    fn flush_ipc(&mut self) {
        if self.pending_ipc.is_empty() {
            return;
        }
        let delta = std::mem::take(&mut self.pending_ipc);
        self.last_ipc_flush = Instant::now();
        self.emitter.emit(AgentEvent::MessageDelta {
            session_id: self.session_id,
            turn_id: self.turn_id.clone(),
            message_id: self.message_id,
            delta,
        });
    }

    fn finish(&mut self) {
        self.flush_db();
        self.flush_ipc();
    }
}

fn persist_assistant(
    conn: &Connection,
    session_id: i64,
    turn_id: &str,
    response: &ChatResponse,
) -> Result<MessageInfo, ApiError> {
    let (prompt_tokens, completion_tokens) = match response.usage {
        Some(usage) => (Some(usage.prompt_tokens), Some(usage.completion_tokens)),
        None => (None, None),
    };
    session_repo::append_message(
        conn,
        session_id,
        &session_repo::NewMessage::new(MessageRole::Assistant, response.content.clone())
            .with_turn_id(turn_id)
            .with_tool_calls(response.tool_calls.clone())
            .with_usage(prompt_tokens, completion_tokens),
    )
}

struct ToolRun {
    ok: bool,
    result: Option<serde_json::Value>,
    error: Option<ApiError>,
    content: String,
}

impl ToolRun {
    fn success(result: serde_json::Value) -> Self {
        let content = serde_json::json!({ "ok": true, "result": result });
        Self {
            ok: true,
            result: Some(result),
            error: None,
            content: content.to_string(),
        }
    }

    fn failure(error: ApiError) -> Self {
        let content = serde_json::json!({
            "ok": false,
            "error": { "code": error.code, "message": error.message }
        });
        Self {
            ok: false,
            result: None,
            error: Some(error),
            content: content.to_string(),
        }
    }
}

enum Dispatch {
    Run(ToolRun),
    Cancelled,
}

async fn dispatch_tool(
    deps: &TurnDeps,
    session_id: i64,
    agent_id: i64,
    turn_id: &str,
    call: &crate::interface::session::ToolCall,
    ctl: &mut TurnControlHandle,
    emitter: &Arc<dyn EventEmitter>,
) -> Dispatch {
    let key = match ToolKey::parse(&call.tool_id) {
        Ok(key) => key,
        Err(error) => return Dispatch::Run(ToolRun::failure(error)),
    };

    if !call.arguments.is_object() {
        return Dispatch::Run(ToolRun::failure(ApiError::invalid_input(
            "工具参数必须是 JSON 对象",
        )));
    }

    let permission = match with_conn(&deps.db, |conn| {
        if agent_repo::get_tool(conn, &key.group, &key.id)?.is_none() {
            return Err(ApiError::tool_unavailable(format!(
                "工具尚未注册: {}",
                call.tool_id
            )));
        }
        permission::resolve(conn, agent_id, &key)
    }) {
        Ok(permission) => permission,
        Err(error) => return Dispatch::Run(ToolRun::failure(error)),
    };

    match permission {
        ToolPermission::Deny => Dispatch::Run(ToolRun::failure(ApiError::tool_denied(format!(
            "Agent {agent_id} 无权调用工具 {}",
            call.tool_id
        )))),
        ToolPermission::Allow => {
            Dispatch::Run(execute_tool(deps, session_id, agent_id, call, &key))
        }
        ToolPermission::Ask => {
            emitter.emit(AgentEvent::ToolApprovalRequired {
                session_id,
                turn_id: turn_id.to_string(),
                call_id: call.id.clone(),
                tool_id: call.tool_id.clone(),
                arguments: call.arguments.clone(),
            });

            let decision = loop {
                if ctl.is_cancelled() {
                    return Dispatch::Cancelled;
                }
                let (cancel, commands) = ctl.split();
                tokio::select! {
                    command = commands.recv() => match command {
                        Some(TurnCommand::Approve { call_id, decision }) if call_id == call.id => {
                            break decision;
                        }
                        Some(_) => continue,
                        None => return Dispatch::Cancelled,
                    },
                    _ = cancel.changed() => return Dispatch::Cancelled,
                }
            };

            match decision {
                ApprovalDecision::AllowOnce => {
                    Dispatch::Run(execute_tool(deps, session_id, agent_id, call, &key))
                }
                ApprovalDecision::AllowAlways => {
                    let updated = with_conn(&deps.db, |conn| {
                        agent_repo::set_tool_permission(
                            conn,
                            agent_id,
                            &key.group,
                            &key.id,
                            ToolPermission::Allow,
                        )?;
                        Ok(())
                    });
                    match updated {
                        Ok(()) => {
                            Dispatch::Run(execute_tool(deps, session_id, agent_id, call, &key))
                        }
                        Err(error) => Dispatch::Run(ToolRun::failure(error)),
                    }
                }
                ApprovalDecision::Deny => Dispatch::Run(ToolRun::failure(ApiError::tool_denied(
                    format!("用户拒绝了工具调用 {}", call.tool_id),
                ))),
            }
        }
    }
}

fn execute_tool(
    deps: &TurnDeps,
    session_id: i64,
    agent_id: i64,
    call: &crate::interface::session::ToolCall,
    key: &ToolKey,
) -> ToolRun {
    let executed = with_conn(&deps.db, |conn| {
        let ctx = ToolContext {
            conn,
            agent_id,
            session_id,
            config: &deps.config,
        };
        deps.tools.execute(key, &ctx, call.arguments.clone())
    });

    match executed {
        Ok(outcome) => ToolRun::success(outcome.content),
        Err(error) => ToolRun::failure(error),
    }
}

fn persist_tool_result(
    deps: &TurnDeps,
    session_id: i64,
    turn_id: &str,
    call: &crate::interface::session::ToolCall,
    run: &ToolRun,
) -> Result<(), ApiError> {
    with_conn(&deps.db, |conn| {
        session_repo::append_message(
            conn,
            session_id,
            &session_repo::NewMessage::new(MessageRole::Tool, run.content.clone())
                .with_turn_id(turn_id)
                .with_tool_result(call.id.clone(), call.tool_id.clone()),
        )?;
        Ok(())
    })
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::time::Duration;

    use super::turn::TurnRegistry;
    use super::*;
    use crate::config::{AppConfig, LlmConfig, ProviderConfig};
    use crate::interface::llm::ChatRequest;
    use crate::interface::session::ToolCall;
    use crate::repository::db;
    use crate::service::event::RecordingEmitter;
    use crate::service::llm::{BoxFut, LlmBackend};
    use crate::service::tool::{Tool, ToolOutcome};

    const TURN_ID: &str = "t-1";

    fn plain(content: &str) -> ChatResponse {
        ChatResponse {
            content: content.to_string(),
            tool_calls: Vec::new(),
            finish_reason: Some("stop".to_string()),
            usage: None,
            compression: None,
        }
    }

    fn tool_response(tool_id: &str, call_id: &str) -> ChatResponse {
        ChatResponse {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: call_id.to_string(),
                tool_id: tool_id.to_string(),
                arguments: serde_json::json!({ "value": 1 }),
            }],
            finish_reason: Some("tool_calls".to_string()),
            usage: None,
            compression: None,
        }
    }

    /// 按脚本返回响应的桩后端；流式固定回放两个增量。
    struct ScriptBackend {
        responses: Mutex<VecDeque<ChatResponse>>,
    }

    impl ScriptBackend {
        fn new(responses: Vec<ChatResponse>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
            }
        }
    }

    impl LlmBackend for ScriptBackend {
        fn complete<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            let next = self
                .responses
                .lock()
                .expect("脚本锁已中毒")
                .pop_front()
                .unwrap_or_else(|| plain("默认回复"));
            Box::pin(async move { Ok(next) })
        }

        fn stream<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
            sink: &'a mut (dyn FnMut(StreamEvent) + Send),
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                sink(StreamEvent::ContentDelta("流式".to_string()));
                sink(StreamEvent::ContentDelta("回答".to_string()));
                Ok(plain("流式回答"))
            })
        }
    }

    /// 永不返回的桩后端：用于验证取消。
    struct HangingBackend;

    impl LlmBackend for HangingBackend {
        fn complete<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                std::future::pending::<()>().await;
                unreachable!()
            })
        }

        fn stream<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
            _sink: &'a mut (dyn FnMut(StreamEvent) + Send),
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                std::future::pending::<()>().await;
                unreachable!()
            })
        }
    }

    struct EchoTool {
        group: &'static str,
        id: &'static str,
    }

    impl Tool for EchoTool {
        fn key(&self) -> ToolKey {
            ToolKey {
                group: self.group.to_string(),
                id: self.id.to_string(),
            }
        }

        fn execute(
            &self,
            _ctx: &ToolContext<'_>,
            arguments: serde_json::Value,
        ) -> Result<ToolOutcome, ApiError> {
            Ok(ToolOutcome::new(serde_json::json!({ "echo": arguments })))
        }
    }

    struct Fixture {
        deps: TurnDeps,
        registry: Arc<TurnRegistry>,
        emitter: Arc<RecordingEmitter>,
        session_id: i64,
    }

    fn fixture_with(
        responses: Vec<ChatResponse>,
        backend: Arc<dyn LlmBackend>,
        register_tools: bool,
        max_steps: u32,
    ) -> Fixture {
        let conn = db::open_in_memory().unwrap();
        let session = session_repo::insert_session(&conn, 1, "新会话").unwrap();
        session_repo::append_message(
            &conn,
            session.id,
            &session_repo::NewMessage::new(MessageRole::System, "你是助手"),
        )
        .unwrap();
        session_repo::append_message(
            &conn,
            session.id,
            &session_repo::NewMessage::new(MessageRole::User, "你好").with_turn_id(TURN_ID),
        )
        .unwrap();

        let mut app = AppConfig::default();
        let mut llm_config = LlmConfig::default();
        llm_config.default_provider = Some("edgee".to_string());
        llm_config.max_steps = max_steps;
        llm_config.providers.insert(
            "edgee".to_string(),
            ProviderConfig {
                base_url: "https://edgee.io".to_string(),
                api_key: "sk-test".to_string(),
                model: "model-a".to_string(),
                compression_model: None,
            },
        );
        app.llm = llm_config;
        let config = ConfigHandle::new(app);

        let backend: Arc<dyn LlmBackend> = if responses.is_empty() {
            backend
        } else {
            Arc::new(ScriptBackend::new(responses))
        };

        let mut tools = ToolRegistry::new();
        if register_tools {
            tools.register(Arc::new(EchoTool {
                group: "anki",
                id: "list_decks",
            }));
            tools.register(Arc::new(EchoTool {
                group: "anki",
                id: "add_card",
            }));
        }

        Fixture {
            deps: TurnDeps {
                db: Arc::new(Mutex::new(conn)),
                tools: Arc::new(tools),
                llm: LlmClient::new(config.clone(), backend),
                config,
            },
            registry: Arc::new(TurnRegistry::new()),
            emitter: Arc::new(RecordingEmitter::new()),
            session_id: session.id,
        }
    }

    fn fixture(responses: Vec<ChatResponse>, register_tools: bool, max_steps: u32) -> Fixture {
        let script: Arc<dyn LlmBackend> = Arc::new(ScriptBackend::new(responses));
        fixture_with(Vec::new(), script, register_tools, max_steps)
    }

    fn target() -> LlmTarget {
        LlmTarget {
            provider: "edgee".to_string(),
            model: "model-a".to_string(),
        }
    }

    async fn run(fixture: &Fixture) -> TurnOutcome {
        let control = fixture.registry.begin(fixture.session_id, TURN_ID).unwrap();
        let emitter: Arc<dyn EventEmitter> = fixture.emitter.clone();
        run_turn(
            fixture.deps.clone(),
            fixture.session_id,
            1,
            TURN_ID.to_string(),
            target(),
            control,
            emitter,
        )
        .await
    }

    fn messages(fixture: &Fixture) -> Vec<MessageInfo> {
        with_conn(&fixture.deps.db, |conn| {
            session_repo::list_messages(conn, fixture.session_id)
        })
        .unwrap()
    }

    fn count_events(fixture: &Fixture, pred: impl Fn(&AgentEvent) -> bool) -> usize {
        fixture
            .emitter
            .events()
            .iter()
            .filter(|event| pred(event))
            .count()
    }

    async fn wait_until(mut condition: impl FnMut() -> bool) {
        for _ in 0..400 {
            if condition() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
        panic!("条件未在预期时间内满足");
    }

    #[tokio::test]
    async fn turn_without_tool_calls_completes() {
        let fixture = fixture(vec![plain("你好，同学")], true, 8);
        let outcome = run(&fixture).await;

        assert_eq!(outcome.status, TurnStatus::Completed);
        let events = fixture.emitter.events();
        assert!(matches!(
            events.first(),
            Some(AgentEvent::TurnStarted { .. })
        ));
        assert!(matches!(
            events.last(),
            Some(AgentEvent::TurnEnded {
                status: TurnStatus::Completed,
                ..
            })
        ));
        assert_eq!(messages(&fixture).last().unwrap().content, "你好，同学");
    }

    #[tokio::test]
    async fn allowed_tool_runs_and_feeds_result_back() {
        let fixture = fixture(
            vec![
                tool_response("anki.list_decks", "call-1"),
                plain("最终回答"),
            ],
            true,
            8,
        );
        let outcome = run(&fixture).await;

        assert_eq!(outcome.status, TurnStatus::Completed);
        assert_eq!(
            count_events(&fixture, |e| matches!(e, AgentEvent::ToolCall { .. })),
            1
        );
        let results: Vec<_> = fixture
            .emitter
            .events()
            .into_iter()
            .filter_map(|event| match event {
                AgentEvent::ToolResult { ok, .. } => Some(ok),
                _ => None,
            })
            .collect();
        assert_eq!(results, vec![true]);

        let stored = messages(&fixture);
        // system 快照 + user + assistant(tool_calls) + tool + assistant(最终回答)。
        assert_eq!(stored.len(), 5);
        assert_eq!(stored[3].role, MessageRole::Tool);
        assert_eq!(stored[3].tool_name.as_deref(), Some("anki.list_decks"));
        assert!(stored[3].content.contains("\"ok\":true"));
        assert_eq!(stored[4].content, "最终回答");
    }

    #[tokio::test]
    async fn ask_allow_always_writes_permission_back() {
        let fixture = fixture(
            vec![tool_response("anki.add_card", "call-1"), plain("完成")],
            true,
            8,
        );
        let control = fixture.registry.begin(fixture.session_id, TURN_ID).unwrap();
        let emitter: Arc<dyn EventEmitter> = fixture.emitter.clone();
        let deps = fixture.deps.clone();
        let registry = fixture.registry.clone();
        let session_id = fixture.session_id;

        let handle = tokio::spawn(async move {
            run_turn(
                deps,
                session_id,
                1,
                TURN_ID.to_string(),
                target(),
                control,
                emitter,
            )
            .await
        });

        {
            let emitter = fixture.emitter.clone();
            wait_until(move || {
                emitter
                    .events()
                    .iter()
                    .any(|event| matches!(event, AgentEvent::ToolApprovalRequired { .. }))
            })
            .await;
        }

        registry
            .approve(session_id, TURN_ID, "call-1", ApprovalDecision::AllowAlways)
            .unwrap();

        let outcome = handle.await.unwrap();
        assert_eq!(outcome.status, TurnStatus::Completed);
        assert_eq!(
            count_events(&fixture, |e| matches!(
                e,
                AgentEvent::ToolApprovalRequired { .. }
            )),
            1
        );

        let permission = with_conn(&fixture.deps.db, |conn| {
            permission::resolve(conn, 1, &ToolKey::parse("anki.add_card").unwrap())
        })
        .unwrap();
        assert_eq!(permission, ToolPermission::Allow);
    }

    #[tokio::test]
    async fn denied_tool_becomes_an_error_result() {
        let fixture = fixture(
            vec![tool_response("anki.delete_card", "call-1"), plain("好的")],
            true,
            8,
        );
        with_conn(&fixture.deps.db, |conn| {
            conn.execute(
                "DELETE FROM agent_tool WHERE agent_id = 1 AND tool_id = 'delete_card'",
                [],
            )?;
            Ok(())
        })
        .unwrap();

        let outcome = run(&fixture).await;
        assert_eq!(outcome.status, TurnStatus::Completed);

        let error = fixture
            .emitter
            .events()
            .into_iter()
            .find_map(|event| match event {
                AgentEvent::ToolResult { ok, error, .. } if !ok => error,
                _ => None,
            })
            .expect("应有失败的工具结果");
        assert_eq!(error.code, "tool_denied");
    }

    #[tokio::test]
    async fn unregistered_tool_reports_unavailable() {
        let fixture = fixture(
            vec![tool_response("anki.list_decks", "call-1"), plain("好的")],
            false,
            8,
        );
        let outcome = run(&fixture).await;
        assert_eq!(outcome.status, TurnStatus::Completed);

        let error = fixture
            .emitter
            .events()
            .into_iter()
            .find_map(|event| match event {
                AgentEvent::ToolResult { ok, error, .. } if !ok => error,
                _ => None,
            })
            .expect("应有失败的工具结果");
        assert_eq!(error.code, "tool_unavailable");
    }

    #[tokio::test]
    async fn step_limit_appends_explanation() {
        let fixture = fixture(
            vec![
                tool_response("anki.list_decks", "call-1"),
                tool_response("anki.list_decks", "call-2"),
            ],
            true,
            2,
        );
        let outcome = run(&fixture).await;

        assert_eq!(outcome.status, TurnStatus::StepLimit);
        assert!(matches!(
            fixture.emitter.events().last(),
            Some(AgentEvent::TurnEnded {
                status: TurnStatus::StepLimit,
                ..
            })
        ));
        let last = messages(&fixture).last().unwrap().clone();
        assert_eq!(last.role, MessageRole::Assistant);
        assert!(last.content.contains("上限"));
    }

    #[tokio::test]
    async fn cancellation_stops_without_extra_messages() {
        let fixture = fixture_with(Vec::new(), Arc::new(HangingBackend), true, 8);
        let control = fixture.registry.begin(fixture.session_id, TURN_ID).unwrap();
        let emitter: Arc<dyn EventEmitter> = fixture.emitter.clone();
        let deps = fixture.deps.clone();
        let registry = fixture.registry.clone();
        let session_id = fixture.session_id;

        let handle = tokio::spawn(async move {
            run_turn(
                deps,
                session_id,
                1,
                TURN_ID.to_string(),
                target(),
                control,
                emitter,
            )
            .await
        });

        {
            let probe = fixture.emitter.clone();
            wait_until(move || !probe.events().is_empty()).await;
        }
        registry.cancel(session_id).unwrap();

        let outcome = handle.await.unwrap();
        assert_eq!(outcome.status, TurnStatus::Cancelled);
        assert_eq!(messages(&fixture).len(), 2);
    }

    #[tokio::test]
    async fn tool_less_requests_stream_but_tool_requests_do_not() {
        let streaming = fixture(vec![plain("流式回答")], false, 8);
        with_conn(&streaming.deps.db, |conn| {
            conn.execute("DELETE FROM agent_tool WHERE agent_id = 1", [])?;
            Ok(())
        })
        .unwrap();
        let outcome = run(&streaming).await;
        assert_eq!(outcome.status, TurnStatus::Completed);
        // 两个增量在 50ms / 32 字符阈值内被合并为一次推送。
        let deltas: Vec<String> = streaming
            .emitter
            .events()
            .into_iter()
            .filter_map(|event| match event {
                AgentEvent::MessageDelta { delta, .. } => Some(delta),
                _ => None,
            })
            .collect();
        assert_eq!(deltas, vec!["流式回答".to_string()]);
        assert_eq!(messages(&streaming).last().unwrap().content, "流式回答");

        let tooled = fixture(vec![plain("普通回答")], true, 8);
        run(&tooled).await;
        assert_eq!(
            count_events(&tooled, |e| matches!(e, AgentEvent::MessageDelta { .. })),
            0
        );
    }
}
