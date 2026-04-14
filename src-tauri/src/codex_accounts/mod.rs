pub use aah_core::codex_accounts::{auth, cli, models, paths, schedule, service, store};

use tauri::AppHandle;

use self::models::{CodexAccountListItem, StoredCodexAccount};
use self::paths::CodexAccountPaths;
use self::service::CodexAccountService;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

fn service_from_app() -> Result<CodexAccountService, String> {
    let managed = aah_core::bootstrap::bootstrap_managed_root(None, None)?;
    Ok(CodexAccountService::with_process_runner(
        CodexAccountPaths::from_roots(managed.root, managed.user_home),
    ))
}

#[tauri::command]
pub async fn list_codex_accounts(_app: AppHandle) -> Result<Vec<CodexAccountListItem>, String> {
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.list_accounts())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn start_codex_account_login(
    app: AppHandle,
    scheduler: tauri::State<'_, CodexUsageSchedulerState>,
) -> Result<StoredCodexAccount, String> {
    let refresh_app = app.clone();
    let account = tauri::async_runtime::spawn_blocking(move || service_from_app()?.start_login())
        .await
        .map_err(|error| error.to_string())??;

    let _ = scheduler.refresh_codex_now().await;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);

    Ok(account)
}

#[tauri::command]
pub async fn switch_codex_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.switch_account(&account_id))
        .await
        .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}

#[tauri::command]
pub async fn delete_codex_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.delete_account(&account_id))
        .await
        .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}
