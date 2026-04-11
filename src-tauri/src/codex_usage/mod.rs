pub mod models;
pub mod oauth;
pub mod scheduler;
pub mod service;
pub mod store;

use dirs::home_dir;
use tauri::{AppHandle, Manager, State};

use self::models::CodexRefreshSettings;
use self::scheduler::CodexUsageSchedulerState;
use crate::codex_accounts::paths::CodexAccountPaths;

fn paths_from_app(app: &AppHandle) -> Result<CodexAccountPaths, String> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
    let user_home = home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;

    Ok(CodexAccountPaths::from_roots(app_data_dir, user_home))
}

pub fn initialize_scheduler(
    app: &AppHandle,
    scheduler: &CodexUsageSchedulerState,
) -> Result<(), String> {
    scheduler.initialize(app.clone(), paths_from_app(app)?)
}

#[tauri::command]
pub async fn get_codex_refresh_settings(app: AppHandle) -> Result<CodexRefreshSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        store::load_refresh_settings(&paths_from_app(&app)?)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn update_codex_refresh_settings(
    app: AppHandle,
    scheduler: State<'_, CodexUsageSchedulerState>,
    settings: CodexRefreshSettings,
) -> Result<CodexRefreshSettings, String> {
    let saved = tauri::async_runtime::spawn_blocking(move || {
        let paths = paths_from_app(&app)?;
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
