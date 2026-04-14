pub use aah_core::claude_usage::{cli_probe, models, oauth, service, store};

use tauri::State;

use crate::codex_usage::scheduler::CodexUsageSchedulerState;

#[tauri::command]
pub async fn refresh_claude_usage_now(
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<(), String> {
    scheduler.refresh_claude_now().await
}
