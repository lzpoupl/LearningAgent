//! 应用配置：从配置文件加载为全局读写状态，并通过依赖注入下发到各层。
//!
//! 配置由 [`ConfigHandle`] 持有，内部使用 [`RwLock`] 保证并发访问串行：
//! 多个读者可并行，写者独占。应用启动时调用 [`init`] 建立全局实例，随后
//! 各层通过注入的句柄访问同一份配置。

pub mod anki;

use std::path::Path;
use std::sync::{Arc, OnceLock, RwLock};

use serde::{Deserialize, Serialize};

pub use anki::{AnkiConfig, SchedulerConfig};

/// 应用配置聚合根，对应 config.toml。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    /// Anki 模块配置。
    pub anki: AnkiConfig,
}

/// 配置句柄：克隆后共享同一份配置与同一把读写锁，可注入到任意层级。
#[derive(Clone)]
pub struct ConfigHandle {
    inner: Arc<RwLock<AppConfig>>,
}

impl ConfigHandle {
    pub fn new(config: AppConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(config)),
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
}

static GLOBAL: OnceLock<ConfigHandle> = OnceLock::new();

/// 初始化全局配置；重复调用保留首次建立的实例。之后用 [`global`] 取句柄注入各层。
pub fn init(config: AppConfig) {
    GLOBAL.get_or_init(|| ConfigHandle::new(config));
}

/// 获取全局配置句柄；需先调用 [`init`]。
pub fn global() -> &'static ConfigHandle {
    GLOBAL
        .get()
        .expect("全局配置尚未初始化，请先调用 config::init")
}

/// 从配置文件加载配置；文件不存在时使用内置默认值。
pub fn load(path: &Path) -> Result<AppConfig, ::config::ConfigError> {
    let mut builder = ::config::Config::builder();
    if path.exists() {
        builder = builder.add_source(::config::File::from(path));
    }
    builder.build()?.try_deserialize()
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
    fn handle_clones_share_single_rwlock() {
        let handle = ConfigHandle::new(AppConfig::default());
        let clone = handle.clone();

        let mut scheduler = clone.anki().scheduler;
        scheduler.algorithm = "fsrs".to_string();
        clone.set_scheduler(scheduler);

        assert_eq!(handle.anki().scheduler.algorithm, "fsrs");
    }
}
