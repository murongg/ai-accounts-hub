pub mod cli_probe;
pub mod models;
pub mod oauth;
pub mod service;
pub mod store;

use tauri::State;

use crate::codex_usage::scheduler::CodexUsageSchedulerState;

#[tauri::command]
pub async fn refresh_claude_usage_now(
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<(), String> {
    scheduler.refresh_claude_now().await
}
