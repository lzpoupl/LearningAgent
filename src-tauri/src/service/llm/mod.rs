//! edgee 后端抽象、provider 解析与调用封装。
//!
//! `LlmBackend` 是唯一的模型调用出口：生产环境注入 `EdgeeBackend`，测试注入桩实现。
//! `LlmClient` 负责把「会话记录 → 默认配置」解析成一次调用的目标，并统一施加超时。

pub mod edgee;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::{ConfigHandle, ProviderConfig};
use crate::interface::error::ApiError;
use crate::interface::llm::{
    ChatMessage, ChatRequest, ChatResponse, LlmConfigView, LlmProviderInput, LlmProviderView,
    LlmTarget, ProbeResult, StreamEvent,
};

/// 后端调用的装箱 future；trait 对象需要显式生命周期。
pub type BoxFut<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// 模型后端抽象；`complete` 对应 edgee 的 `send`，`stream` 对应 `stream`。
pub trait LlmBackend: Send + Sync {
    /// 非流式调用；工具调用与用量都在这里获得。
    fn complete<'a>(
        &'a self,
        cfg: &'a ProviderConfig,
        model: &'a str,
        request: &'a ChatRequest,
    ) -> BoxFut<'a, Result<ChatResponse, ApiError>>;

    /// 流式调用；只用于无工具的请求，文本增量经 `sink` 回调。
    fn stream<'a>(
        &'a self,
        cfg: &'a ProviderConfig,
        model: &'a str,
        request: &'a ChatRequest,
        sink: &'a mut (dyn FnMut(StreamEvent) + Send),
    ) -> BoxFut<'a, Result<ChatResponse, ApiError>>;
}

/// 线名映射：OpenAI 的 function name 不允许包含 `.`，因此 `<group>.<id>` 写成
/// `<group>__<id>`，并在一次请求内维护还原表，避免 id 内出现 `__` 时歧义。
pub fn wire_name(group: &str, id: &str) -> String {
    format!("{group}__{id}")
}

/// 把 `<group>.<id>` 还原为线名；无 `.` 时原样返回。
pub fn tool_id_to_wire(tool_id: &str) -> String {
    match tool_id.split_once('.') {
        Some((group, id)) => wire_name(group, id),
        None => tool_id.to_string(),
    }
}

/// 模型客户端：解析调用目标、施加上限超时，并托管 provider 配置视图。
#[derive(Clone)]
pub struct LlmClient {
    config: ConfigHandle,
    backend: Arc<dyn LlmBackend>,
}

impl LlmClient {
    pub fn new(config: ConfigHandle, backend: Arc<dyn LlmBackend>) -> Self {
        Self { config, backend }
    }

    /// 解析一次调用的目标：provider 取会话记录的 `last_provider`，缺省用
    /// `default_provider`；模型取会话记录的 `last_model`，缺省用该 provider 的模型。
    pub fn resolve(
        &self,
        last_provider: Option<&str>,
        last_model: Option<&str>,
    ) -> Result<LlmTarget, ApiError> {
        let llm = self.config.llm();
        let provider = last_provider
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(String::from)
            .unwrap_or_else(|| llm.default_provider.trim().to_string());
        if provider.is_empty() {
            return Err(ApiError::llm_unconfigured("未配置默认 provider"));
        }

        let provider_cfg = llm
            .providers
            .get(&provider)
            .ok_or_else(|| ApiError::llm_unconfigured(format!("provider 不存在: {provider}")))?;

        let model = last_model
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(String::from)
            .unwrap_or_else(|| provider_cfg.model.trim().to_string());
        if model.is_empty() {
            return Err(ApiError::llm_unconfigured(format!(
                "provider {provider} 未配置模型"
            )));
        }

        if self.effective_api_key(provider_cfg).is_empty() {
            return Err(ApiError::llm_unconfigured(format!(
                "provider {provider} 未配置 API 密钥"
            )));
        }

        Ok(LlmTarget { provider, model })
    }

    /// 取实际使用的密钥：provider 配置优先，为空时回退环境变量。
    fn effective_api_key(&self, cfg: &ProviderConfig) -> String {
        if !cfg.api_key.trim().is_empty() {
            return cfg.api_key.trim().to_string();
        }
        std::env::var("EDGEE_API_KEY").unwrap_or_default()
    }

    fn provider_config(&self, name: &str) -> Result<ProviderConfig, ApiError> {
        let llm = self.config.llm();
        let mut cfg = llm
            .providers
            .get(name)
            .cloned()
            .ok_or_else(|| ApiError::llm_unconfigured(format!("provider 不存在: {name}")))?;
        if cfg.api_key.trim().is_empty() {
            cfg.api_key = self.effective_api_key(&cfg);
        }
        if cfg.api_key.trim().is_empty() {
            return Err(ApiError::llm_unconfigured(format!(
                "provider {name} 未配置 API 密钥"
            )));
        }
        Ok(cfg)
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.config.llm().request_timeout_seconds.max(1))
    }

    /// 非流式调用。
    pub async fn complete(
        &self,
        target: &LlmTarget,
        request: &ChatRequest,
    ) -> Result<ChatResponse, ApiError> {
        let cfg = self.provider_config(&target.provider)?;
        let timeout = self.timeout();
        let fut = self.backend.complete(&cfg, &target.model, request);
        tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| ApiError::llm_error(format!("模型调用超时（{} 秒）", timeout.as_secs())))?
    }

    /// 流式调用；文本增量经 `sink` 回调。
    pub async fn stream(
        &self,
        target: &LlmTarget,
        request: &ChatRequest,
        sink: &mut (dyn FnMut(StreamEvent) + Send),
    ) -> Result<ChatResponse, ApiError> {
        let cfg = self.provider_config(&target.provider)?;
        let timeout = self.timeout();
        let fut = self.backend.stream(&cfg, &target.model, request, sink);
        tokio::time::timeout(timeout, fut)
            .await
            .map_err(|_| ApiError::llm_error(format!("模型调用超时（{} 秒）", timeout.as_secs())))?
    }

    /// 连通性探测：发送一次极小的非流式请求。
    pub async fn probe(&self, name: Option<&str>) -> Result<ProbeResult, ApiError> {
        let explicit = name.map(str::trim).filter(|name| !name.is_empty());
        let target = self.resolve(explicit, None)?;
        let request = ChatRequest {
            messages: vec![ChatMessage::user("ping")],
            tools: Vec::new(),
        };

        let started = Instant::now();
        let response = self.complete(&target, &request).await?;

        Ok(ProbeResult {
            provider: target.provider,
            model: target.model,
            latency_ms: started.elapsed().as_millis() as u64,
            reply: response.content,
        })
    }

    /// 面向前端的配置视图，密钥脱敏。
    pub fn config_view(&self) -> LlmConfigView {
        let llm = self.config.llm();
        let mut names: Vec<String> = llm.providers.keys().cloned().collect();
        names.sort();

        let providers = names
            .into_iter()
            .filter_map(|name| {
                let cfg = llm.providers.get(&name)?;
                let key = self.effective_api_key(cfg);
                Some(LlmProviderView {
                    name,
                    base_url: cfg.base_url.clone(),
                    model: cfg.model.clone(),
                    api_key_configured: !key.is_empty(),
                    api_key_masked: mask_api_key(&key),
                    compression_model: cfg.compression_model.clone(),
                })
            })
            .collect();

        LlmConfigView {
            default_provider: llm.default_provider,
            max_steps: llm.max_steps,
            allow_streaming: llm.allow_streaming,
            providers,
        }
    }

    /// 新增或更新 provider；`api_key` 为 `None` / 空串时保留原密钥。
    pub fn upsert_provider(&self, input: LlmProviderInput) -> Result<LlmConfigView, ApiError> {
        let name = input.name.trim().to_string();
        if name.is_empty() {
            return Err(ApiError::invalid_input("provider 名称不能为空"));
        }
        if input.base_url.trim().is_empty() {
            return Err(ApiError::invalid_input("base_url 不能为空"));
        }
        if input.model.trim().is_empty() {
            return Err(ApiError::invalid_input("模型名不能为空"));
        }

        let mut llm = self.config.llm();
        let existing_key = llm
            .providers
            .get(&name)
            .map(|cfg| cfg.api_key.clone())
            .unwrap_or_default();
        let api_key = match input.api_key.as_deref().map(str::trim) {
            Some(key) if !key.is_empty() => key.to_string(),
            _ => existing_key,
        };
        let compression_model = input
            .compression_model
            .filter(|model| !model.trim().is_empty());

        llm.providers.insert(
            name,
            ProviderConfig {
                base_url: input.base_url.trim().to_string(),
                api_key,
                model: input.model.trim().to_string(),
                compression_model,
            },
        );
        self.config.set_llm(llm)?;
        Ok(self.config_view())
    }

    /// 切换默认 provider。
    pub fn set_default_provider(&self, name: &str) -> Result<LlmConfigView, ApiError> {
        let name = name.trim();
        let mut llm = self.config.llm();
        if !llm.providers.contains_key(name) {
            return Err(ApiError::not_found(format!("provider 不存在: {name}")));
        }
        llm.default_provider = name.to_string();
        self.config.set_llm(llm)?;
        Ok(self.config_view())
    }
}

/// 密钥脱敏：保留前 3 位与后 4 位，过短则整体隐藏。
fn mask_api_key(key: &str) -> String {
    let chars: Vec<char> = key.trim().chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    if chars.len() <= 7 {
        return "***".to_string();
    }
    let head: String = chars.iter().take(3).collect();
    let tail: String = chars.iter().skip(chars.len() - 4).collect();
    format!("{head}***{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, LlmConfig};
    use crate::interface::llm::ChatResponse;

    /// 记录型桩后端：仅用于验证客户端解析与视图，不发起网络请求。
    struct StubBackend;

    impl LlmBackend for StubBackend {
        fn complete<'a>(
            &'a self,
            _cfg: &'a ProviderConfig,
            _model: &'a str,
            _request: &'a ChatRequest,
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                Ok(ChatResponse {
                    content: "pong".to_string(),
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
            _sink: &'a mut (dyn FnMut(StreamEvent) + Send),
        ) -> BoxFut<'a, Result<ChatResponse, ApiError>> {
            Box::pin(async move {
                Ok(ChatResponse {
                    content: String::new(),
                    tool_calls: Vec::new(),
                    finish_reason: None,
                    usage: None,
                    compression: None,
                })
            })
        }
    }

    fn client() -> LlmClient {
        let mut app = AppConfig::default();
        let mut llm = LlmConfig::default();
        llm.default_provider = "edgee".to_string();
        llm.providers.insert(
            "edgee".to_string(),
            ProviderConfig {
                base_url: "https://edgee.io".to_string(),
                api_key: "sk-1234567890abcd".to_string(),
                model: "anthropic/claude-haiku-4-5".to_string(),
                compression_model: None,
            },
        );
        llm.providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                base_url: "https://api.deepseek.com".to_string(),
                api_key: "sk-deepseek-key".to_string(),
                model: "deepseek-chat".to_string(),
                compression_model: None,
            },
        );
        app.llm = llm;
        LlmClient::new(ConfigHandle::new(app), Arc::new(StubBackend))
    }

    #[test]
    fn resolve_prefers_session_records() {
        let client = client();

        let target = client.resolve(None, None).unwrap();
        assert_eq!(target.provider, "edgee");
        assert_eq!(target.model, "anthropic/claude-haiku-4-5");

        let target = client
            .resolve(Some("deepseek"), Some("deepseek-reasoner"))
            .unwrap();
        assert_eq!(target.provider, "deepseek");
        assert_eq!(target.model, "deepseek-reasoner");

        assert_eq!(
            client.resolve(Some("missing"), None).unwrap_err().code,
            "llm_unconfigured"
        );
    }

    #[test]
    fn resolve_requires_api_key() {
        let mut app = AppConfig::default();
        app.llm = LlmConfig::default();
        app.llm.providers.get_mut("edgee").unwrap().api_key = String::new();
        let client = LlmClient::new(ConfigHandle::new(app), Arc::new(StubBackend));

        std::env::remove_var("EDGEE_API_KEY");
        assert_eq!(
            client.resolve(None, None).unwrap_err().code,
            "llm_unconfigured"
        );
    }

    #[test]
    fn config_view_masks_keys_and_sorts() {
        let client = client();
        let view = client.config_view();

        assert_eq!(view.default_provider, "edgee");
        assert_eq!(view.max_steps, 8);
        assert_eq!(
            view.providers
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>(),
            vec!["deepseek", "edgee"]
        );
        let edgee = view.providers.iter().find(|p| p.name == "edgee").unwrap();
        assert!(edgee.api_key_configured);
        assert_eq!(edgee.api_key_masked, "sk-***abcd");
    }

    #[test]
    fn upsert_provider_keeps_key_when_omitted() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut app = AppConfig::default();
        app.llm = LlmConfig::default();
        let config = ConfigHandle::new(app);
        config.set_path(path);
        let client = LlmClient::new(config, Arc::new(StubBackend));

        let view = client
            .upsert_provider(LlmProviderInput {
                name: "deepseek".to_string(),
                base_url: "https://api.deepseek.com".to_string(),
                model: "deepseek-chat".to_string(),
                api_key: Some("sk-new-key-0000".to_string()),
                compression_model: None,
            })
            .unwrap();
        assert!(view.providers.iter().any(|p| p.name == "deepseek"));

        // 不传密钥时保留原值。
        let view = client
            .upsert_provider(LlmProviderInput {
                name: "deepseek".to_string(),
                base_url: "https://api.deepseek.com".to_string(),
                model: "deepseek-reasoner".to_string(),
                api_key: None,
                compression_model: None,
            })
            .unwrap();
        let deepseek = view
            .providers
            .iter()
            .find(|p| p.name == "deepseek")
            .unwrap();
        assert_eq!(deepseek.model, "deepseek-reasoner");
        assert_eq!(deepseek.api_key_masked, "sk-***0000");

        assert_eq!(
            client.set_default_provider("nope").unwrap_err().code,
            "not_found"
        );
        client.set_default_provider("deepseek").unwrap();
        assert_eq!(client.config_view().default_provider, "deepseek");
    }
}
