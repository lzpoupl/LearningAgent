pub mod agent;
pub mod anki;
pub mod asset;
pub mod llm;
pub mod session;
pub mod statistics;
pub mod ui;

/// 汇总各模块的 Tauri 命令。
#[macro_export]
macro_rules! controller_handlers {
    () => {
        tauri::generate_handler![
            crate::controller::anki::anki_get_subdecks,
            crate::controller::anki::anki_get_cards,
            crate::controller::anki::anki_get_card,
            crate::controller::anki::anki_search_cards,
            crate::controller::anki::anki_create_deck,
            crate::controller::anki::anki_create_card,
            crate::controller::anki::anki_move_deck,
            crate::controller::anki::anki_move_card,
            crate::controller::anki::anki_grade_card,
            crate::controller::anki::anki_get_review_options,
            crate::controller::anki::anki_reset_card,
            crate::controller::anki::anki_update_card_content,
            crate::controller::anki::anki_delete_deck,
            crate::controller::anki::anki_delete_card,
            crate::controller::anki::anki_get_scheduler_config,
            crate::controller::anki::anki_update_scheduler_config,
            crate::controller::statistics::stats_get_today_progress,
            crate::controller::statistics::stats_get_card_breakdown,
            crate::controller::statistics::stats_get_review_history,
            crate::controller::statistics::stats_get_added_cards,
            crate::controller::agent::agent_list,
            crate::controller::agent::agent_get,
            crate::controller::agent::agent_create,
            crate::controller::agent::agent_update,
            crate::controller::agent::agent_delete,
            crate::controller::agent::tool_list,
            crate::controller::agent::get_all_tool_groups,
            crate::controller::agent::agent_get_tool_permissions,
            crate::controller::agent::agent_set_tool_permission,
            crate::controller::agent::agent_set_tool_permissions,
            crate::controller::agent::agent_resolve_tool_permission,
            crate::controller::asset::bucket_list,
            crate::controller::asset::bucket_create,
            crate::controller::asset::bucket_update,
            crate::controller::asset::bucket_delete,
            crate::controller::asset::asset_list,
            crate::controller::asset::asset_upload,
            crate::controller::asset::asset_get_url,
            crate::controller::asset::asset_delete,
            crate::controller::asset::asset_upload_image,
            crate::controller::session::session_list,
            crate::controller::session::session_get,
            crate::controller::session::session_rename,
            crate::controller::session::session_delete,
            crate::controller::session::agent_start_session,
            crate::controller::session::agent_send_message,
            crate::controller::session::agent_cancel_turn,
            crate::controller::session::agent_approve_tool_call,
            crate::controller::session::agent_answer_question,
            crate::controller::session::agent_skip_question,
            crate::controller::llm::llm_get_config,
            crate::controller::llm::llm_upsert_provider,
            crate::controller::llm::llm_set_default_provider,
            crate::controller::llm::llm_test_provider,
            crate::controller::ui::ui_get_theme,
            crate::controller::ui::ui_set_theme,
        ]
    };
}
