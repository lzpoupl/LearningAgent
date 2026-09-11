use crate::config::SchedulerConfig;
use crate::interface::anki::*;
use crate::AppState;

#[tauri::command]
pub fn anki_get_subdecks(
    state: tauri::State<'_, AppState>,
    deck_path: String,
) -> Result<Vec<Deck>, AnkiError> {
    state.anki.get_subdecks(&deck_path)
}

#[tauri::command]
pub fn anki_get_cards(
    state: tauri::State<'_, AppState>,
    deck_path: String,
    query: CardQuery,
) -> Result<Vec<Card>, AnkiError> {
    state.anki.get_cards(&deck_path, query)
}

#[tauri::command]
pub fn anki_get_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
) -> Result<Card, AnkiError> {
    state.anki.get_card(&card_id)
}

#[tauri::command]
pub fn anki_search_cards(
    state: tauri::State<'_, AppState>,
    keyword: String,
    search: CardSearch,
) -> Result<Vec<Card>, AnkiError> {
    state.anki.search_cards(&keyword, search)
}

#[tauri::command]
pub fn anki_create_deck(
    state: tauri::State<'_, AppState>,
    deck_path: String,
) -> Result<String, AnkiError> {
    state.anki.create_deck(&deck_path)
}

#[tauri::command]
pub fn anki_create_card(
    state: tauri::State<'_, AppState>,
    new_card: NewCard,
) -> Result<String, AnkiError> {
    state.anki.create_card(new_card)
}

#[tauri::command]
pub fn anki_move_deck(
    state: tauri::State<'_, AppState>,
    source_path: String,
    target_path: String,
) -> Result<(), AnkiError> {
    state.anki.move_deck(&source_path, &target_path)
}

#[tauri::command]
pub fn anki_move_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
    target_deck_path: String,
) -> Result<(), AnkiError> {
    state.anki.move_card(&card_id, &target_deck_path)
}

#[tauri::command]
pub fn anki_grade_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
    grade: CardGrade,
) -> Result<ReviewOutcome, AnkiError> {
    state.anki.grade_card(&card_id, grade)
}

#[tauri::command]
pub fn anki_get_review_options(
    state: tauri::State<'_, AppState>,
    card_id: String,
) -> Result<Vec<ReviewOption>, AnkiError> {
    state.anki.get_review_options(&card_id)
}

#[tauri::command]
pub fn anki_reset_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
) -> Result<ReviewOutcome, AnkiError> {
    state.anki.reset_card(&card_id)
}

#[tauri::command]
pub fn anki_update_card_content(
    state: tauri::State<'_, AppState>,
    card_id: String,
    content: UpdateCardContent,
) -> Result<(), AnkiError> {
    state.anki.update_card_content(&card_id, content)
}

#[tauri::command]
pub fn anki_delete_deck(
    state: tauri::State<'_, AppState>,
    deck_path: String,
) -> Result<(), AnkiError> {
    state.anki.delete_deck(&deck_path)
}

#[tauri::command]
pub fn anki_delete_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
) -> Result<(), AnkiError> {
    state.anki.delete_card(&card_id)
}

#[tauri::command]
pub fn anki_get_scheduler_config(
    state: tauri::State<'_, AppState>,
) -> Result<SchedulerConfig, AnkiError> {
    Ok(state.config.anki().scheduler)
}

#[tauri::command]
pub fn anki_update_scheduler_config(
    state: tauri::State<'_, AppState>,
    scheduler: SchedulerConfig,
) -> Result<SchedulerConfig, AnkiError> {
    state.anki.update_scheduler_config(scheduler)
}

#[macro_export]
macro_rules! anki_handlers {
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
        ]
    };
}
