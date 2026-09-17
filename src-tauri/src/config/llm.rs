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
    /// 未指定会话记录时使用的默认 provider 名。
    pub default_provider: String,
    /// 单轮循环的步数上限（含工具调用轮次）。
    pub max_steps: u32,
    /// 单次模型调用的超时时间（秒）。
    pub request_timeout_seconds: u64,
    /// 可重试错误的最大重试次数。
    pub max_retries: u32,
    /// 是否允许在无工具的请求上启用流式输出。
    pub allow_streaming: bool,
    /// provider 配置表，键为 provider 名。
    #[serde(default = "default_providers")]
    pub providers: HashMap<String, ProviderConfig>,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            default_provider: "edgee".to_string(),
            max_steps: 8,
            request_timeout_seconds: 120,
            max_retries: 2,
            allow_streaming: true,
            providers: default_providers(),
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

/// 内置的默认 provider：Edgee 网关，密钥留空待用户填写。
pub fn default_providers() -> HashMap<String, ProviderConfig> {
    let mut providers = HashMap::new();
    providers.insert(
        "edgee".to_string(),
        ProviderConfig {
            base_url: "https://edgee.io".to_string(),
            api_key: String::new(),
            model: "anthropic/claude-haiku-4-5".to_string(),
            compression_model: None,
        },
    );
    providers
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
    fn defaults_are_edgee_with_reasonable_limits() {
        let llm = LlmConfig::default();
        assert_eq!(llm.default_provider, "edgee");
        assert_eq!(llm.max_steps, 8);
        assert_eq!(llm.request_timeout_seconds, 120);
        assert_eq!(llm.max_retries, 2);
        assert!(llm.allow_streaming);
        assert_eq!(llm.providers["edgee"].base_url, "https://edgee.io");
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

        assert_eq!(llm.default_provider, "deepseek");
        assert_eq!(llm.max_steps, 4);
        assert_eq!(llm.allow_streaming, true);
        assert_eq!(llm.providers["deepseek"].model, "deepseek-chat");
        assert!(llm.providers["deepseek"].compression_model.is_none());
    }

    #[test]
    fn missing_providers_falls_back_to_seeded_edgee() {
        let llm = from_toml("max_steps = 3\n");
        assert!(llm.providers.contains_key("edgee"));
    }
}
