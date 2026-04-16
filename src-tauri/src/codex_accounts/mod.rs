pub use aah_core::codex_accounts::{
    auth, cli, device_login, models, paths, schedule, service, store,
};

use serde::Deserialize;
use tauri::AppHandle;

use self::device_login::CodexDeviceAutofillLoginRequest;
use self::models::{CodexAccountListItem, StoredCodexAccount};
use self::paths::CodexAccountPaths;
use self::service::CodexAccountService;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

#[derive(Debug, Deserialize)]
pub struct CodexDeviceAutofillLoginInput {
    pub email: String,
    pub password: String,
}

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
pub async fn start_codex_account_device_autofill_login(
    app: AppHandle,
    scheduler: tauri::State<'_, CodexUsageSchedulerState>,
    input: CodexDeviceAutofillLoginInput,
) -> Result<StoredCodexAccount, String> {
    let email = input.email.trim().to_string();
    if email.is_empty() {
        return Err("Codex login email is required".to_string());
    }
    if input.password.is_empty() {
        return Err("Codex login password is required".to_string());
    }

    let refresh_app = app.clone();
    let account = tauri::async_runtime::spawn_blocking(move || {
        service_from_app()?.start_device_autofill_login(CodexDeviceAutofillLoginRequest {
            email,
            password: input.password,
        })
    })
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
