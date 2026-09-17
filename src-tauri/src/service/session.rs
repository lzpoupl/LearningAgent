//! 会话 CRUD、轮次启动与异步任务编排。

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use crate::config::ConfigHandle;
use crate::interface::error::ApiError;
use crate::interface::llm::LlmTarget;
use crate::interface::session::{
    ApprovalDecision, MessageRole, SendMessageInput, SessionDetail, SessionInfo, StartSessionInput,
    TurnHandle,
};
use crate::repository::{agent as agent_repo, session as session_repo};
use crate::service::agent_loop::turn::{TurnControlHandle, TurnRegistry};
use crate::service::agent_loop::{run_turn, TurnDeps};
use crate::service::event::EventEmitter;
use crate::service::llm::LlmClient;
use crate::service::tool::ToolRegistry;
use crate::service::with_conn;

/// 会话标题占位值；首条用户消息到达后替换。
const PLACEHOLDER_TITLE: &str = "新会话";
/// 自动标题保留的最大字符数。
const TITLE_LIMIT: usize = 20;
/// Agent 未配置系统提示词时的兜底说明。
const DEFAULT_SYSTEM_PROMPT: &str = "你是学习助手，基于用户的提问与学习资产提供帮助。";

/// 会话服务：持有循环依赖、轮次注册表与轮次编号计数器。
#[derive(Clone)]
pub struct SessionService {
    deps: TurnDeps,
    turns: Arc<TurnRegistry>,
    counter: Arc<AtomicU64>,
}

impl SessionService {
    pub fn new(
        db: Arc<Mutex<Connection>>,
        tools: Arc<ToolRegistry>,
        llm: LlmClient,
        config: ConfigHandle,
    ) -> Self {
        Self {
            deps: TurnDeps {
                db,
                tools,
                llm,
                config,
            },
            turns: Arc::new(TurnRegistry::new()),
            counter: Arc::new(AtomicU64::new(0)),
        }
    }

    fn next_turn_id(&self) -> String {
        let millis = chrono::Utc::now().timestamp_millis();
        let seq = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("t-{millis}-{seq}")
    }

    // ---------- 会话 ----------

    pub fn list_sessions(&self, agent_id: Option<i64>) -> Result<Vec<SessionInfo>, ApiError> {
        with_conn(&self.deps.db, |conn| {
            session_repo::list_sessions(conn, agent_id)
        })
    }

    pub fn get_session(&self, session_id: i64) -> Result<SessionDetail, ApiError> {
        with_conn(&self.deps.db, |conn| {
            // 冷启动恢复：残留的 streaming 消息先收敛为 interrupted。
            session_repo::interrupt_streaming(conn, session_id)?;
            let session = session_repo::ensure_session(conn, session_id)?;
            let messages = session_repo::list_messages(conn, session_id)?;
            Ok(SessionDetail { session, messages })
        })
    }

    pub fn rename_session(&self, session_id: i64, title: &str) -> Result<SessionInfo, ApiError> {
        with_conn(&self.deps.db, |conn| {
            session_repo::update_title(conn, session_id, title)
        })
    }

    pub fn delete_session(&self, session_id: i64) -> Result<(), ApiError> {
        with_conn(&self.deps.db, |conn| {
            session_repo::delete_session(conn, session_id)
        })
    }

    // ---------- 轮次 ----------

    /// 新建会话并立即开始第一轮。
    pub fn start_session(
        &self,
        input: StartSessionInput,
        emitter: Arc<dyn EventEmitter>,
    ) -> Result<TurnHandle, ApiError> {
        let content = input.content.trim().to_string();
        if content.is_empty() {
            return Err(ApiError::invalid_input("消息不能为空"));
        }

        // 先解析调用目标：未配置时不落任何消息。
        let target = self.deps.llm.resolve(None, None)?;
        let turn_id = self.next_turn_id();

        let session = with_conn(&self.deps.db, |conn| {
            let system_prompt = agent_repo::get_system_prompt(conn, input.agent_id)?;
            let system_prompt = if system_prompt.trim().is_empty() {
                DEFAULT_SYSTEM_PROMPT.to_string()
            } else {
                system_prompt
            };

            crate::repository::with_tx(conn, |c| {
                let session = session_repo::insert_session(c, input.agent_id, PLACEHOLDER_TITLE)?;
                session_repo::append_message(
                    c,
                    session.id,
                    &session_repo::NewMessage::new(MessageRole::System, system_prompt),
                )?;
                session_repo::append_message(
                    c,
                    session.id,
                    &session_repo::NewMessage::new(MessageRole::User, &content)
                        .with_turn_id(&turn_id),
                )?;
                session_repo::set_session_model(c, session.id, &target.provider, &target.model)?;
                session_repo::update_title(c, session.id, &auto_title(&content))?;
                session_repo::ensure_session(c, session.id)
            })
        })?;

        let control = self.turns.begin(session.id, &turn_id)?;
        self.spawn_turn(
            session.id,
            session.agent_id,
            turn_id.clone(),
            target,
            control,
            emitter,
        );
        Ok(TurnHandle {
            session_id: session.id,
            turn_id,
        })
    }

    /// 在既有会话上开始新一轮。
    pub fn send_message(
        &self,
        input: SendMessageInput,
        emitter: Arc<dyn EventEmitter>,
    ) -> Result<TurnHandle, ApiError> {
        let content = input.content.trim().to_string();
        if content.is_empty() {
            return Err(ApiError::invalid_input("消息不能为空"));
        }

        let session = with_conn(&self.deps.db, |conn| {
            session_repo::ensure_session(conn, input.session_id)
        })?;

        // 后续轮次沿用会话记录的 provider / model。
        let target = self.deps.llm.resolve(
            session.last_provider.as_deref(),
            session.last_model.as_deref(),
        )?;
        let turn_id = self.next_turn_id();

        let session = with_conn(&self.deps.db, |conn| {
            crate::repository::with_tx(conn, |c| {
                session_repo::append_message(
                    c,
                    session.id,
                    &session_repo::NewMessage::new(MessageRole::User, &content)
                        .with_turn_id(&turn_id),
                )?;
                session_repo::set_session_model(c, session.id, &target.provider, &target.model)?;
                if session.title.trim() == PLACEHOLDER_TITLE {
                    session_repo::update_title(c, session.id, &auto_title(&content))?;
                }
                session_repo::ensure_session(c, session.id)
            })
        })?;

        let control = self.turns.begin(session.id, &turn_id)?;
        self.spawn_turn(
            session.id,
            session.agent_id,
            turn_id.clone(),
            target,
            control,
            emitter,
        );
        Ok(TurnHandle {
            session_id: session.id,
            turn_id,
        })
    }

    pub fn cancel_turn(&self, session_id: i64) -> Result<(), ApiError> {
        self.turns.cancel(session_id)
    }

    pub fn approve_tool_call(
        &self,
        session_id: i64,
        turn_id: &str,
        call_id: &str,
        decision: ApprovalDecision,
    ) -> Result<(), ApiError> {
        self.turns.approve(session_id, turn_id, call_id, decision)
    }

    fn spawn_turn(
        &self,
        session_id: i64,
        agent_id: i64,
        turn_id: String,
        target: LlmTarget,
        control: TurnControlHandle,
        emitter: Arc<dyn EventEmitter>,
    ) {
        let deps = self.deps.clone();
        let turns = self.turns.clone();
        let finished_turn = turn_id.clone();

        tauri::async_runtime::spawn(async move {
            run_turn(
                deps, session_id, agent_id, turn_id, target, control, emitter,
            )
            .await;
            turns.finish(session_id, &finished_turn);
        });
    }
}

/// 首条用户消息自动生成标题：前 20 个字符，超出加省略号。
fn auto_title(content: &str) -> String {
    let chars: Vec<char> = content.chars().collect();
    if chars.len() > TITLE_LIMIT {
        let head: String = chars.iter().take(TITLE_LIMIT).collect();
        format!("{head}...")
    } else {
        content.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, LlmConfig, ProviderConfig};
    use crate::interface::event::AgentEvent;
    use crate::interface::llm::{ChatRequest, ChatResponse, StreamEvent};
    use crate::repository::db;
    use crate::service::event::RecordingEmitter;
    use crate::service::llm::{BoxFut, LlmBackend};

    /// 永不返回的桩后端：让轮次保持在「进行中」，避免断言与后台任务竞态。
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

    /// 立即返回纯文本回答的桩后端。
    struct FastBackend;

    impl LlmBackend for FastBackend {
        fn complete<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                Ok(ChatResponse {
                    content: "这是回答。".to_string(),
                    tool_calls: Vec::new(),
                    finish_reason: Some("stop".to_string()),
                    usage: None,
                    compression: None,
                })
            })
        }

        fn stream<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
            sink: &'a mut (dyn FnMut(StreamEvent) + Send),
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                sink(StreamEvent::ContentDelta("这是回答。".to_string()));
                Ok(ChatResponse {
                    content: "这是回答。".to_string(),
                    tool_calls: Vec::new(),
                    finish_reason: Some("stop".to_string()),
                    usage: None,
                    compression: None,
                })
            })
        }
    }

    fn config() -> ConfigHandle {
        let mut app = AppConfig::default();
        let mut llm = LlmConfig::default();
        llm.default_provider = Some("edgee".to_string());
        llm.providers.insert(
            "edgee".to_string(),
            ProviderConfig {
                base_url: "https://edgee.io".to_string(),
                api_key: "sk-test".to_string(),
                model: "model-a".to_string(),
                compression_model: None,
            },
        );
        llm.providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                base_url: "https://api.deepseek.com".to_string(),
                api_key: "sk-test".to_string(),
                model: "deepseek-chat".to_string(),
                compression_model: None,
            },
        );
        app.llm = llm;
        ConfigHandle::new(app)
    }

    fn service_with(config: ConfigHandle, backend: Arc<dyn LlmBackend>) -> SessionService {
        let conn = db::open_in_memory().unwrap();
        SessionService::new(
            Arc::new(Mutex::new(conn)),
            Arc::new(ToolRegistry::new()),
            LlmClient::new(config.clone(), backend),
            config,
        )
    }

    fn hanging_service(config: ConfigHandle) -> SessionService {
        service_with(config, Arc::new(HangingBackend))
    }

    fn fast_service(config: ConfigHandle) -> SessionService {
        service_with(config, Arc::new(FastBackend))
    }

    async fn wait_for_turn_end(emitter: &Arc<RecordingEmitter>) {
        for _ in 0..200 {
            if emitter
                .events()
                .iter()
                .any(|event| matches!(event, AgentEvent::TurnEnded { .. }))
            {
                return;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
        panic!("轮次未在预期时间内结束");
    }

    #[tokio::test]
    async fn start_session_writes_snapshot_title_and_model() {
        let service = hanging_service(config());
        let emitter = RecordingEmitter::new();

        let handle = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "什么是导数？".to_string(),
                },
                emitter.into_arc(),
            )
            .unwrap();

        let detail = service.get_session(handle.session_id).unwrap();
        assert_eq!(detail.session.last_provider.as_deref(), Some("edgee"));
        assert_eq!(detail.session.last_model.as_deref(), Some("model-a"));
        assert_eq!(detail.session.title, "什么是导数？");
        // system 快照 + user 消息；助手消息要等模型返回。
        assert_eq!(detail.messages.len(), 2);
        assert_eq!(detail.messages[0].role, MessageRole::System);
        assert_eq!(
            detail.messages[0].content,
            "你是数学学习助手，负责题目解析、知识点拆解与错题复盘。"
        );
        assert_eq!(detail.messages[1].role, MessageRole::User);
        assert_eq!(
            detail.messages[1].turn_id.as_deref(),
            Some(handle.turn_id.as_str())
        );
    }

    #[tokio::test]
    async fn auto_title_truncates_long_content() {
        let service = hanging_service(config());
        let handle = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "一".repeat(30),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap();

        let detail = service.get_session(handle.session_id).unwrap();
        let title = detail.session.title;
        assert_eq!(title.chars().count(), 23);
        assert!(title.ends_with("..."));
    }

    #[tokio::test]
    async fn send_message_requires_existing_session_and_valid_content() {
        let service = hanging_service(config());

        let error = service
            .send_message(
                SendMessageInput {
                    session_id: 999,
                    content: "hi".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap_err();
        assert_eq!(error.code, "not_found");

        let handle = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "第一轮".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap();

        let error = service
            .send_message(
                SendMessageInput {
                    session_id: handle.session_id,
                    content: "   ".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap_err();
        assert_eq!(error.code, "invalid_input");
    }

    #[tokio::test]
    async fn unconfigured_provider_writes_no_messages() {
        let config = config();
        let mut llm = config.llm();
        llm.providers.get_mut("edgee").unwrap().api_key = String::new();
        config.set_llm(llm).unwrap();
        std::env::remove_var("EDGEE_API_KEY");

        let service = hanging_service(config);
        let error = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "hi".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap_err();
        assert_eq!(error.code, "llm_unconfigured");

        assert!(service.list_sessions(None).unwrap().is_empty());
    }

    #[tokio::test]
    async fn later_turns_keep_the_recorded_model() {
        let config = config();
        let service = fast_service(config.clone());

        let emitter = Arc::new(RecordingEmitter::new());
        let handle = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "第一轮".to_string(),
                },
                emitter.clone(),
            )
            .unwrap();
        wait_for_turn_end(&emitter).await;

        // 修改默认 provider 后，已有会话仍沿用记录。
        let mut llm = config.llm();
        llm.default_provider = Some("deepseek".to_string());
        config.set_llm(llm).unwrap();

        service
            .send_message(
                SendMessageInput {
                    session_id: handle.session_id,
                    content: "第二轮".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap();

        let detail = service.get_session(handle.session_id).unwrap();
        assert_eq!(detail.session.last_provider.as_deref(), Some("edgee"));
        assert_eq!(detail.session.last_model.as_deref(), Some("model-a"));
        // 标题不再被第二条消息覆盖。
        assert_eq!(detail.session.title, "第一轮");
    }

    #[tokio::test]
    async fn concurrent_turn_returns_conflict() {
        let service = hanging_service(config());
        let handle = service
            .start_session(
                StartSessionInput {
                    agent_id: 1,
                    content: "第一轮".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap();

        let error = service
            .send_message(
                SendMessageInput {
                    session_id: handle.session_id,
                    content: "第二轮".to_string(),
                },
                RecordingEmitter::new().into_arc(),
            )
            .unwrap_err();
        assert_eq!(error.code, "turn_conflict");
    }
}
