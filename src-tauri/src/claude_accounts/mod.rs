pub mod auth;
pub mod cli;
pub mod keychain;
pub mod live_credentials;
pub mod models;
pub mod paths;
pub mod service;
pub mod store;

use dirs::home_dir;
use tauri::{AppHandle, Manager};

use self::models::{ClaudeAccountListItem, StoredClaudeAccount};
use self::paths::ClaudeAccountPaths;
use self::service::ClaudeAccountService;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

fn service_from_app(
    app: &AppHandle,
) -> Result<
    ClaudeAccountService<
        self::keychain::ManagedClaudeKeychainStore,
        self::live_credentials::FileSystemClaudeLiveCredentialStore,
    >,
    String,
> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
    let user_home = home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;

    Ok(ClaudeAccountService::with_process_runner(
        ClaudeAccountPaths::from_roots(app_data_dir, user_home),
    ))
}

#[tauri::command]
pub async fn list_claude_accounts(app: AppHandle) -> Result<Vec<ClaudeAccountListItem>, String> {
    tauri::async_runtime::spawn_blocking(move || service_from_app(&app)?.list_accounts())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn start_claude_account_login(
    app: AppHandle,
    scheduler: tauri::State<'_, CodexUsageSchedulerState>,
) -> Result<StoredClaudeAccount, String> {
    let refresh_app = app.clone();
    let account = tauri::async_runtime::spawn_blocking(move || {
        let mut service = service_from_app(&app)?;
        service.start_login()
    })
    .await
    .map_err(|error| error.to_string())??;

    let _ = scheduler.refresh_claude_now().await;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(account)
}

#[tauri::command]
pub async fn switch_claude_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut service = service_from_app(&app)?;
        service.switch_account(&account_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}

#[tauri::command]
pub async fn delete_claude_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let mut service = service_from_app(&app)?;
        service.delete_account(&account_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}
