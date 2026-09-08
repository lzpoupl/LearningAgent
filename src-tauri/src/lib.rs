mod controller;
mod interface;
mod repository;
mod service;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use crate::service::anki::AnkiService;
use crate::service::scheduler::SchedulerRegistry;

/// 应用全局共享状态。
pub struct AppState {
    /// Anki 服务：数据库连接与调度算法注册表在启动时构建一次。
    pub anki: AnkiService,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = repository::db::open(data_dir.join("learningagent.db"))?;
            let anki = AnkiService::new(
                Arc::new(Mutex::new(conn)),
                Arc::new(SchedulerRegistry::new()),
            );
            let _ = app.manage(AppState { anki });
            Ok(())
        })
        .invoke_handler(controller_handlers!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
