pub mod scheduler;

pub use aah_core::codex_usage::{models, oauth, service, store};

use tauri::{AppHandle, State};

use self::models::CodexRefreshSettings;
use self::scheduler::CodexUsageSchedulerState;
use crate::codex_accounts::paths::CodexAccountPaths;

fn paths_from_app() -> Result<CodexAccountPaths, String> {
    let managed = aah_core::bootstrap::bootstrap_managed_root(None, None)?;
    Ok(CodexAccountPaths::from_roots(
        managed.root,
        managed.user_home,
    ))
}

pub fn initialize_scheduler(
    app: &AppHandle,
    scheduler: &CodexUsageSchedulerState,
) -> Result<(), String> {
    scheduler.initialize(app.clone(), paths_from_app()?)
}

#[tauri::command]
pub async fn get_codex_refresh_settings(_app: AppHandle) -> Result<CodexRefreshSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        store::load_refresh_settings(&paths_from_app()?)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn update_codex_refresh_settings(
    _app: AppHandle,
    scheduler: State<'_, CodexUsageSchedulerState>,
    settings: CodexRefreshSettings,
) -> Result<CodexRefreshSettings, String> {
    let saved = tauri::async_runtime::spawn_blocking(move || {
        let paths = paths_from_app()?;
        store::save_refresh_settings(&paths, settings)
    })
    .await
    .map_err(|error| error.to_string())??;

    scheduler.update_settings(saved.clone())?;
    Ok(saved)
}

#[tauri::command]
pub async fn refresh_codex_usage_now(
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<(), String> {
    scheduler.refresh_codex_now().await
}
