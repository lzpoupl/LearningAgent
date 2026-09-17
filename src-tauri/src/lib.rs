mod config;
mod controller;
mod interface;
mod repository;
mod service;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::config::ConfigHandle;
use crate::service::agent::AgentService;
use crate::service::anki::AnkiService;
use crate::service::asset::AssetService;
use crate::service::llm::edgee::EdgeeBackend;
use crate::service::llm::LlmClient;
use crate::service::scheduler::SchedulerRegistry;
use crate::service::session::SessionService;
use crate::service::statistics::StatisticsService;
use crate::service::tool::ToolRegistry;

/// 应用全局共享状态。
pub struct AppState {
    /// Anki 服务：数据库连接、调度算法注册表与配置句柄在启动时构建一次。
    pub anki: AnkiService,
    /// Agent 服务：Agent 配置与工具权限编排。
    pub agent: AgentService,
    /// 资产服务：bucket 映射与非结构化资产读写。
    pub asset: AssetService,
    /// 统计服务：Anki 卡片统计的只读查询。
    pub statistics: StatisticsService,
    /// 会话服务：会话 CRUD 与 Agent 轮次编排。
    pub session: SessionService,
    /// 模型客户端：provider 解析、调用与探测。
    pub llm: LlmClient,
    /// 工具实现注册表：本阶段为空，各 service 共享同一实例。
    pub tool_registry: Arc<ToolRegistry>,
    /// 全局配置句柄：与各层注入的句柄共享同一份配置。
    pub config: ConfigHandle,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 读取配置文件并建立全局读写状态，随后取出全局句柄注入各层。
            let config_path = resolve_config_path();
            let app_config = config::load(&config_path)?;
            config::init(app_config, config_path);
            let config = config::global().clone();

            // Vite 的开发模式对应 Tauri 的 debug 构建；前端使用 mock 时，后端
            // 也使用进程级内存数据库，确保测试数据不会污染正式数据。
            #[cfg(debug_assertions)]
            let conn = repository::db::open_in_memory()?;

            #[cfg(not(debug_assertions))]
            let conn = {
                let data_dir = app.path().app_data_dir()?;
                std::fs::create_dir_all(&data_dir)?;
                repository::db::open(data_dir.join("learningagent.db"))?
            };

            let db = Arc::new(Mutex::new(conn));
            let tool_registry = Arc::new(ToolRegistry::new());

            let anki = AnkiService::new(
                db.clone(),
                Arc::new(SchedulerRegistry::new()),
                config.clone(),
            );
            let agent = AgentService::new(db.clone(), tool_registry.clone(), config.clone());
            let asset = AssetService::new(db.clone());
            let statistics = StatisticsService::new(db.clone());

            let llm = LlmClient::new(config.clone(), Arc::new(EdgeeBackend::new(config.clone())));
            let session = SessionService::new(
                db.clone(),
                tool_registry.clone(),
                llm.clone(),
                config.clone(),
            );

            let _ = app.manage(AppState {
                anki,
                agent,
                asset,
                statistics,
                session,
                llm,
                tool_registry,
                config,
            });
            Ok(())
        })
        .invoke_handler(controller_handlers!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// 解析配置文件路径：开发时使用工作目录下的 config.toml，
/// 发布时使用可执行文件同目录下的 config.toml。
fn resolve_config_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        PathBuf::from("config.toml")
    }
    #[cfg(not(debug_assertions))]
    {
        std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join("config.toml")))
            .unwrap_or_else(|| PathBuf::from("config.toml"))
    }
}
