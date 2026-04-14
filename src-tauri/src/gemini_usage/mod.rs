pub use aah_core::gemini_usage::{models, oauth, service, store};

use tauri::State;

use crate::codex_usage::scheduler::CodexUsageSchedulerState;

#[tauri::command]
pub async fn refresh_gemini_usage_now(
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<(), String> {
    scheduler.refresh_gemini_now().await
}
