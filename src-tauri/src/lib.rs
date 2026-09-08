mod controller;
mod interface;
mod repository;
mod service;

use std::sync::Mutex;

use tauri::Manager;

/// 应用全局共享状态。
pub struct AppState {
    /// 数据库连接（SQLite 单连接，用互斥锁串行化访问）。
    pub db: Mutex<rusqlite::Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = repository::db::open(data_dir.join("learningagent.db"))?;
            let _ = app.manage(AppState {
                db: Mutex::new(conn),
            });
            Ok(())
        })
        .invoke_handler(controller_handlers!())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
