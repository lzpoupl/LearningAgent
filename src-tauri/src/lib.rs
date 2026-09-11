mod config;
mod controller;
mod interface;
mod repository;
mod service;

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::config::ConfigHandle;
use crate::service::anki::AnkiService;
use crate::service::scheduler::SchedulerRegistry;

/// 应用全局共享状态。
pub struct AppState {
    /// Anki 服务：数据库连接、调度算法注册表与配置句柄在启动时构建一次。
    pub anki: AnkiService,
    /// 全局配置句柄：与各层注入的句柄共享同一份配置。
    pub config: ConfigHandle,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // 读取配置文件并建立全局读写状态，随后取出全局句柄注入各层。
            let app_config = config::load(&resolve_config_path())?;
            config::init(app_config);
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

            let anki = AnkiService::new(
                Arc::new(Mutex::new(conn)),
                Arc::new(SchedulerRegistry::new()),
                config.clone(),
            );
            let _ = app.manage(AppState { anki, config });
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
