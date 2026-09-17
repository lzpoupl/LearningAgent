//! 应用配置：从配置文件加载为可共享的读写状态，并通过依赖注入下发到各层。
//!
//! 配置由 [`ConfigHandle`] 持有，内部使用 [`RwLock`] 保证并发访问串行：
//! 多个读者可并行，写者独占。应用启动时在 `lib.rs` 构造句柄，随后各层通过
//! 注入的句柄访问同一份配置。

pub mod anki;
pub mod llm;
pub mod ui;

use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};

use crate::interface::error::ApiError;

pub use anki::{AnkiConfig, SchedulerConfig};
pub use llm::{LlmConfig, ProviderConfig};
pub use ui::UiConfig;

/// 应用配置聚合根，对应 config.toml。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Anki 模块配置。
    pub anki: AnkiConfig,
    /// 大语言模型接入配置。
    pub llm: LlmConfig,
    /// 界面外观配置。
    pub ui: UiConfig,
}

/// 配置句柄：克隆后共享同一份配置、同一把读写锁与同一个配置文件路径。
#[derive(Clone)]
pub struct ConfigHandle {
    inner: Arc<RwLock<AppConfig>>,
    /// 配置文件路径；为 `None` 时只更新内存（测试场景）。
    path: Arc<RwLock<Option<PathBuf>>>,
}

impl ConfigHandle {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
            path: Arc::new(RwLock::new(None)),
        }
    }

    /// 读取 Anki 配置快照。
    pub fn anki(&self) -> AnkiConfig {
        self.inner.read().expect("配置读写锁已中毒").anki.clone()
    }

    /// 仅更新复习调度配置。
    pub fn set_scheduler(&self, scheduler: SchedulerConfig) {
        self.inner.write().expect("配置读写锁已中毒").anki.scheduler = scheduler;
    }

    /// 读取 LLM 配置快照。
    pub fn llm(&self) -> LlmConfig {
        self.inner.read().expect("配置读写锁已中毒").llm.clone()
    }

    /// 更新 LLM 配置：先回写 `config.toml`，成功后再更新内存，保证重启后仍在。
    pub fn set_llm(&self, llm: LlmConfig) -> Result<(), ApiError> {
        self.update(|config| config.llm = llm.clone())
    }

    /// 读取界面外观配置快照。
    pub fn ui(&self) -> UiConfig {
        self.inner.read().expect("配置读写锁已中毒").ui.clone()
    }

    /// 更新界面外观配置：取值非法或写盘失败时返回错误，内存保持原值。
    pub fn set_ui(&self, ui: UiConfig) -> Result<(), ApiError> {
        UiConfig::validate_theme(&ui.theme)?;

        self.update(|config| config.ui = ui.clone())
    }

    /// 以候选值回写 `config.toml`，成功后再替换内存中本次改动的字段。
    ///
    /// `apply` 会被调用两次：先在配置快照上生成待写入文件的内容，再在持有写锁的
    /// 内存配置上重放，这样并发写入的其他字段不会被快照覆盖。
    fn update(&self, mut apply: impl FnMut(&mut AppConfig)) -> Result<(), ApiError> {
        let candidate = {
            let guard = self.inner.read().expect("配置读写锁已中毒");
            let mut candidate = guard.clone();
            apply(&mut candidate);
            candidate
        };

        self.persist(&candidate)?;

        apply(&mut self.inner.write().expect("配置读写锁已中毒"));
        Ok(())
    }

    /// 将配置写入 `config.toml`；未记录路径（测试场景）时只保留在内存。
    fn persist(&self, config: &AppConfig) -> Result<(), ApiError> {
        let path = self.path.read().expect("配置读写锁已中毒").clone();

        let Some(path) = path else {
            return Ok(());
        };

        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)
                .map_err(|e| ApiError::internal(format!("配置目录创建失败: {e}")))?;
        }

        let text = toml::to_string_pretty(config)
            .map_err(|e| ApiError::internal(format!("配置序列化失败: {e}")))?;
        std::fs::write(&path, text)
            .map_err(|e| ApiError::internal(format!("配置文件写入失败: {e}")))?;

        Ok(())
    }

    /// 记录配置文件路径，供写回使用。
    pub fn set_path(&self, path: PathBuf) {
        *self.path.write().expect("配置读写锁已中毒") = Some(path);
    }
}

/// 从配置文件加载配置。
///
/// 文件不存在时先写入默认配置（必要时创建父目录），保证首次启动即生成配置文件；
/// 文件已存在时只读取，不覆盖用户内容；单个键缺失时回落到内置默认值。
pub fn load(path: &Path) -> Result<AppConfig, ::config::ConfigError> {
    if !path.exists() {
        write_default_config(path).map_err(|e| ::config::ConfigError::Foreign(Box::new(e)))?;
    }

    ::config::Config::builder()
        .add_source(::config::File::from(path))
        .build()?
        .try_deserialize()
}

/// 生成默认配置文件内容：直接序列化 [`AppConfig::default`]，
/// 使默认值与模板只在一处维护。
fn default_config_toml() -> String {
    let body = toml::to_string_pretty(&AppConfig::default()).expect("默认配置可以序列化为 TOML");
    format!(
        "# LearningAgent 配置文件。\n\
         # 本文件缺失时启动会自动生成；某个键缺失时回落到内置默认值。\n\n{body}"
    )
}

/// 写入默认配置，父目录不存在时一并创建。
fn write_default_config(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, default_config_toml())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn from_toml(source: &str) -> AppConfig {
        ::config::Config::builder()
            .add_source(::config::File::from_str(source, ::config::FileFormat::Toml))
            .build()
            .unwrap()
            .try_deserialize()
            .unwrap()
    }

    #[test]
    fn defaults_match_learning_steps_and_sm2() {
        let scheduler = SchedulerConfig::default();
        assert_eq!(scheduler.algorithm, "sm2");
        assert_eq!(scheduler.learning_again_minutes, 1);
        assert_eq!(scheduler.learning_hard_minutes, 6);
        assert_eq!(scheduler.learning_good_minutes, 10);
    }

    #[test]
    fn toml_overrides_only_present_keys() {
        let config = from_toml(
            r#"
            [anki.scheduler]
            algorithm = "fsrs"
            learning_good_minutes = 30
            "#,
        );

        assert_eq!(config.anki.scheduler.algorithm, "fsrs");
        assert_eq!(config.anki.scheduler.learning_good_minutes, 30);
        // 未出现的键回落到默认值。
        assert_eq!(config.anki.scheduler.learning_again_minutes, 1);
        assert_eq!(config.anki.scheduler.learning_hard_minutes, 6);
    }

    #[test]
    fn empty_source_falls_back_to_defaults() {
        let config = from_toml("");
        assert_eq!(config.anki.scheduler.algorithm, "sm2");
    }

    #[test]
    fn missing_file_is_created_with_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested").join("config.toml");

        let config = load(&path).unwrap();

        // 首次加载生成配置文件（含父目录），内容为内置默认值。
        assert!(path.exists());
        assert_eq!(config.anki.scheduler.algorithm, "sm2");
        assert_eq!(config.llm.max_steps, 8);

        // 生成的默认文件可被再次加载。
        let reloaded = load(&path).unwrap();
        assert_eq!(reloaded.anki.scheduler.learning_good_minutes, 10);
    }

    #[test]
    fn set_llm_persists_and_reloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let handle = ConfigHandle::new(AppConfig::default());
        handle.set_path(path.clone());

        let mut llm = handle.llm();
        llm.default_provider = Some("deepseek".to_string());
        llm.providers.insert(
            "deepseek".to_string(),
            ProviderConfig {
                base_url: "https://api.deepseek.com".to_string(),
                api_key: "sk-test".to_string(),
                model: "deepseek-chat".to_string(),
                compression_model: None,
            },
        );
        handle.set_llm(llm).unwrap();

        assert!(path.exists());
        let reloaded = load(&path).unwrap();
        assert_eq!(reloaded.llm.default_provider.as_deref(), Some("deepseek"));
        assert_eq!(reloaded.llm.providers["deepseek"].api_key, "sk-test");
        // 内存中的配置同样更新。
        assert_eq!(handle.llm().default_provider.as_deref(), Some("deepseek"));
        // 未改动的 Anki 配置一并回写且保持一致。
        assert_eq!(reloaded.anki.scheduler.algorithm, "sm2");
    }

    #[test]
    fn set_ui_persists_theme_and_rejects_unknown_mode() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");

        let handle = ConfigHandle::new(AppConfig::default());
        handle.set_path(path.clone());

        // 未配置时默认跟随系统。
        assert_eq!(handle.ui().theme, "system");

        let mut ui = handle.ui();
        ui.theme = "dark".to_string();
        handle.set_ui(ui).unwrap();

        assert_eq!(load(&path).unwrap().ui.theme, "dark");
        assert_eq!(handle.ui().theme, "dark");

        // 非法取值既不进内存也不落盘。
        let mut invalid = handle.ui();
        invalid.theme = "midnight".to_string();
        let error = handle.set_ui(invalid).unwrap_err();
        assert_eq!(error.code, "invalid_input");
        assert_eq!(handle.ui().theme, "dark");
        assert_eq!(load(&path).unwrap().ui.theme, "dark");
    }

    #[test]
    fn handle_clones_share_single_rwlock() {
        let handle = ConfigHandle::new(AppConfig::default());
        let clone = handle.clone();

        let mut scheduler = clone.anki().scheduler;
        scheduler.algorithm = "fsrs".to_string();
        clone.set_scheduler(scheduler);

        assert_eq!(handle.anki().scheduler.algorithm, "fsrs");
    }
}
