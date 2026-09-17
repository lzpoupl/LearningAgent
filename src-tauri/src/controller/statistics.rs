use crate::interface::anki::AnkiError;
use crate::interface::statistics::{
    AddedCardsStats, CardBreakdown, ReviewHistoryStats, TimeRange, TodayProgress,
};
use crate::AppState;

#[tauri::command]
pub fn stats_get_today_progress(
    state: tauri::State<'_, AppState>,
) -> Result<TodayProgress, AnkiError> {
    state.statistics.today_progress()
}

#[tauri::command]
pub fn stats_get_card_breakdown(
    state: tauri::State<'_, AppState>,
) -> Result<CardBreakdown, AnkiError> {
    state.statistics.card_breakdown()
}

#[tauri::command]
pub fn stats_get_review_history(
    state: tauri::State<'_, AppState>,
    range: TimeRange,
) -> Result<ReviewHistoryStats, AnkiError> {
    state.statistics.review_history(range)
}

#[tauri::command]
pub fn stats_get_added_cards(
    state: tauri::State<'_, AppState>,
    range: TimeRange,
) -> Result<AddedCardsStats, AnkiError> {
    state.statistics.added_cards(range)
}