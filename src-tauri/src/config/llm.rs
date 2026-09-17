//! 大语言模型接入配置。
//!
//! 字段名与 `config.toml` 中的键保持一致，采用 snake_case。`providers` 以
//! provider 名为键，每个 provider 描述一个 OpenAI 兼容端点；密钥只存在配置
//! 文件中，不进入数据库。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// LLM 接入配置。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct LlmConfig {
    /// 未指定会话记录时使用的默认 provider 名；未配置时为 `None`。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,
    /// 单轮循环的步数上限（含工具调用轮次）。
    pub max_steps: u32,
    /// 单次模型调用的超时时间（秒）。
    pub request_timeout_seconds: u64,
    /// 可重试错误的最大重试次数。
    pub max_retries: u32,
    /// 是否允许在无工具的请求上启用流式输出。
    pub allow_streaming: bool,
    /// provider 配置表，键为 provider 名；默认空表，由设置页或配置文件新增。
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: None,
            max_steps: 8,
            request_timeout_seconds: 120,
            max_retries: 2,
            allow_streaming: true,
            providers: HashMap::new(),
        }
    }
}

/// 单个 provider 的连接与默认模型配置。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ProviderConfig {
    /// 端点根地址，不含 `/v1`，如 `https://edgee.io`。
    pub base_url: String,
    /// API 密钥；为空时回退环境变量 `EDGEE_API_KEY`。
    pub api_key: String,
    /// 该 provider 的默认模型名。
    pub model: String,
    /// 仅 Edgee 网关有意义的压缩模型；为空表示不启用压缩。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compression_model: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_toml(source: &str) -> LlmConfig {
        ::config::Config::builder()
            .add_source(::config::File::from_str(source, ::config::FileFormat::Toml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn toml_overrides_keys_and_providers() {
        let llm = from_toml(
            r#"
            default_provider = "deepseek"
            max_steps = 4

            [providers.deepseek]
            base_url = "https://api.deepseek.com"
            api_key = "sk-test"
            model = "deepseek-chat"
            "#,
        );

        assert_eq!(llm.default_provider.as_deref(), Some("deepseek"));
        assert_eq!(llm.max_steps, 4);
        assert_eq!(llm.allow_streaming, true);
        assert_eq!(llm.providers["deepseek"].model, "deepseek-chat");
        assert!(llm.providers["deepseek"].compression_model.is_none());
    }

    #[test]
    fn missing_defaults_are_unset_and_empty() {
        let llm = from_toml("max_steps = 3\n");
        assert!(llm.default_provider.is_none());
        assert!(llm.providers.is_empty());
        assert_eq!(llm.max_steps, 3);
    }
}
