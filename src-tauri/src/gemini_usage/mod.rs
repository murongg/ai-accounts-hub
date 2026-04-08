use tauri::State;

use crate::codex_usage::scheduler::CodexUsageSchedulerState;

pub mod models;
pub mod oauth;
pub mod service;
pub mod store;

#[tauri::command]
pub async fn refresh_gemini_usage_now(
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<(), String> {
    scheduler.refresh_gemini_now().await
}
