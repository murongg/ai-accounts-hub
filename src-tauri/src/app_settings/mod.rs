pub use aah_core::app_settings::{models, service, store};

use tauri::State;

use self::models::{AppDataDirectoryInfo, AppSettings, ClearAllDataResult};
use crate::codex_accounts::paths::CodexAccountPaths;
use crate::codex_usage::models::CodexRefreshSettings;
use crate::codex_usage::scheduler::CodexUsageSchedulerState;

fn paths_from_app() -> Result<CodexAccountPaths, String> {
    let managed = aah_core::bootstrap::bootstrap_managed_root(None, None)?;
    Ok(CodexAccountPaths::from_roots(
        managed.root,
        managed.user_home,
    ))
}

fn should_trigger_immediate_auto_switch_refresh(
    previous: &AppSettings,
    saved: &AppSettings,
) -> bool {
    !previous.auto_switch_enabled && saved.auto_switch_enabled
}

#[tauri::command]
pub async fn get_app_settings(_app: tauri::AppHandle) -> Result<AppSettings, String> {
    tauri::async_runtime::spawn_blocking(move || store::load_app_settings(&paths_from_app()?))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn update_app_settings(
    app: tauri::AppHandle,
    scheduler: State<'_, CodexUsageSchedulerState>,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let (previous, saved) = tauri::async_runtime::spawn_blocking(move || {
        let paths = paths_from_app()?;
        let previous = store::load_app_settings(&paths)?;
        let saved = store::save_app_settings(&paths, settings)?;
        Ok::<_, String>((previous, saved))
    })
    .await
    .map_err(|error| error.to_string())??;

    let _ = crate::relay::apply_relay_settings_from_app(app).await;
    if should_trigger_immediate_auto_switch_refresh(&previous, &saved) {
        let _ = scheduler.refresh_all_now().await;
    }
    Ok(saved)
}

#[tauri::command]
pub async fn get_app_data_directory_info(
    _app: tauri::AppHandle,
) -> Result<AppDataDirectoryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        service::current_data_directory_info(&paths_from_app()?)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn reset_app_data_directory(
    _app: tauri::AppHandle,
) -> Result<AppDataDirectoryInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        service::reset_data_directory_to_default(&paths_from_app()?)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn clear_all_app_data(
    app: tauri::AppHandle,
    scheduler: State<'_, CodexUsageSchedulerState>,
) -> Result<ClearAllDataResult, String> {
    let result = tauri::async_runtime::spawn_blocking(move || {
        service::clear_all_app_data(&paths_from_app()?)
    })
    .await
    .map_err(|error| error.to_string())??;

    scheduler.update_settings(CodexRefreshSettings::default())?;
    let _ = crate::relay::apply_relay_settings_from_app(app).await;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::should_trigger_immediate_auto_switch_refresh;
    use crate::app_settings::models::AppSettings;

    #[test]
    fn immediate_auto_switch_refresh_runs_only_when_switch_changes_from_disabled_to_enabled() {
        let previous = AppSettings {
            auto_switch_enabled: false,
            ..AppSettings::default()
        };
        let enabled = AppSettings {
            auto_switch_enabled: true,
            ..AppSettings::default()
        };
        let still_disabled = AppSettings {
            auto_switch_enabled: false,
            ..AppSettings::default()
        };
        let still_enabled = AppSettings {
            auto_switch_enabled: true,
            ..AppSettings::default()
        };

        assert!(should_trigger_immediate_auto_switch_refresh(
            &previous,
            &enabled,
        ));
        assert!(!should_trigger_immediate_auto_switch_refresh(
            &previous,
            &still_disabled,
        ));
        assert!(!should_trigger_immediate_auto_switch_refresh(
            &enabled,
            &still_enabled,
        ));
        assert!(!should_trigger_immediate_auto_switch_refresh(
            &enabled,
            &previous,
        ));
    }
}
