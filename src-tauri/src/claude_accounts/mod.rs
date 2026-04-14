pub use aah_core::claude_accounts::{
    auth, cli, keychain, live_credentials, models, paths, service, store,
};

use tauri::AppHandle;

use self::models::{ClaudeAccountListItem, StoredClaudeAccount};
use self::paths::ClaudeAccountPaths;
use self::service::ClaudeAccountService;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

fn service_from_app() -> Result<
    ClaudeAccountService<
        self::keychain::ManagedClaudeKeychainStore,
        self::live_credentials::FileSystemClaudeLiveCredentialStore,
    >,
    String,
> {
    let managed = aah_core::bootstrap::bootstrap_managed_root(None, None)?;
    Ok(ClaudeAccountService::with_process_runner(
        ClaudeAccountPaths::from_roots(managed.root, managed.user_home),
    ))
}

#[tauri::command]
pub async fn list_claude_accounts(_app: AppHandle) -> Result<Vec<ClaudeAccountListItem>, String> {
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.list_accounts())
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
        let mut service = service_from_app()?;
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
        let mut service = service_from_app()?;
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
        let mut service = service_from_app()?;
        service.delete_account(&account_id)
    })
    .await
    .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}
