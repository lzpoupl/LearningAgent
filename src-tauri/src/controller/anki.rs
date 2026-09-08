use crate::interface::anki::*;
use crate::service::anki::AnkiService;
use crate::AppState;

fn not_implemented<T>() -> Result<T, AnkiError> {
    Err(AnkiError {
        code: "internal".into(),
        message: "not implemented".into(),
    })
}

fn db_lock_error() -> AnkiError {
    AnkiError {
        code: "internal".into(),
        message: "数据库连接不可用".into(),
    }
}

#[tauri::command]
pub fn anki_get_subdecks(deck_path: String) -> Result<Vec<Deck>, AnkiError> {
    let _ = deck_path;
    not_implemented()
}

#[tauri::command]
pub fn anki_get_cards(deck_path: String, query: CardQuery) -> Result<Vec<Card>, AnkiError> {
    let _ = (deck_path, query);
    not_implemented()
}

#[tauri::command]
pub fn anki_get_card(card_id: String) -> Result<Card, AnkiError> {
    let _ = card_id;
    not_implemented()
}

#[tauri::command]
pub fn anki_search_cards(keyword: String, search: CardSearch) -> Result<Vec<Card>, AnkiError> {
    let _ = (keyword, search);
    not_implemented()
}

#[tauri::command]
pub fn anki_create_deck(deck_path: String) -> Result<Deck, AnkiError> {
    let _ = deck_path;
    not_implemented()
}

#[tauri::command]
pub fn anki_create_card(new_card: NewCard) -> Result<Card, AnkiError> {
    let _ = new_card;
    not_implemented()
}

#[tauri::command]
pub fn anki_move_deck(source_path: String, target_path: String) -> Result<Deck, AnkiError> {
    let _ = (source_path, target_path);
    not_implemented()
}

#[tauri::command]
pub fn anki_move_card(card_id: String, target_deck_path: String) -> Result<Card, AnkiError> {
    let _ = (card_id, target_deck_path);
    not_implemented()
}

#[tauri::command]
pub fn anki_grade_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
    grade: CardGrade,
) -> Result<ReviewOutcome, AnkiError> {
    let conn = state.db.lock().map_err(|_| db_lock_error())?;
    AnkiService::new(&conn).grade_card(&card_id, grade)
}

#[tauri::command]
pub fn anki_reset_card(
    state: tauri::State<'_, AppState>,
    card_id: String,
) -> Result<ReviewOutcome, AnkiError> {
    let conn = state.db.lock().map_err(|_| db_lock_error())?;
    AnkiService::new(&conn).reset_card(&card_id)
}

#[tauri::command]
pub fn anki_update_card_content(card_id: String, content: UpdateCardContent) -> Result<Card, AnkiError> {
    let _ = (card_id, content);
    not_implemented()
}

#[tauri::command]
pub fn anki_delete_deck(deck_path: String) -> Result<(), AnkiError> {
    let _ = deck_path;
    not_implemented()
}

#[tauri::command]
pub fn anki_delete_card(card_id: String) -> Result<(), AnkiError> {
    let _ = card_id;
    not_implemented()
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
            crate::controller::anki::anki_reset_card,
            crate::controller::anki::anki_update_card_content,
            crate::controller::anki::anki_delete_deck,
            crate::controller::anki::anki_delete_card,
        ]
    };
}
