pub use aah_core::gemini_accounts::{auth, cli, models, paths, service, store};

use tauri::AppHandle;

use self::models::{GeminiAccountListItem, StoredGeminiAccount};
use self::paths::GeminiAccountPaths;
use self::service::GeminiAccountService;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

fn service_from_app() -> Result<GeminiAccountService, String> {
    let managed = aah_core::bootstrap::bootstrap_managed_root(None, None)?;
    Ok(GeminiAccountService::with_process_runner(
        GeminiAccountPaths::from_roots(managed.root, managed.user_home),
    ))
}

#[tauri::command]
pub async fn list_gemini_accounts(_app: AppHandle) -> Result<Vec<GeminiAccountListItem>, String> {
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.list_accounts())
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn start_gemini_account_login(
    app: AppHandle,
    scheduler: tauri::State<'_, CodexUsageSchedulerState>,
) -> Result<StoredGeminiAccount, String> {
    let refresh_app = app.clone();
    let account = tauri::async_runtime::spawn_blocking(move || service_from_app()?.start_login())
        .await
        .map_err(|error| error.to_string())??;

    let _ = scheduler.refresh_gemini_now().await;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);

    Ok(account)
}

#[tauri::command]
pub async fn switch_gemini_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.switch_account(&account_id))
        .await
        .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}

#[tauri::command]
pub async fn delete_gemini_account(app: AppHandle, account_id: String) -> Result<(), String> {
    let refresh_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || service_from_app()?.delete_account(&account_id))
        .await
        .map_err(|error| error.to_string())??;
    let _ = crate::status_bar::refresh_status_menu(&refresh_app);
    Ok(())
}
