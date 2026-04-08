use std::fs;

use crate::codex_accounts::paths::CodexAccountPaths;
use crate::codex_usage::models::CodexRefreshSettings;
use crate::codex_usage::store::save_refresh_settings;

use super::models::{AppDataDirectoryInfo, AppSettings, ClearAllDataResult};
use super::store::save_app_settings;

pub fn current_data_directory_info(paths: &CodexAccountPaths) -> Result<AppDataDirectoryInfo, String> {
    paths.ensure_dirs()?;
    Ok(AppDataDirectoryInfo {
        current_dir: paths.codex_data_dir.display().to_string(),
        default_dir: paths.codex_data_dir.display().to_string(),
        is_default: true,
    })
}

pub fn reset_data_directory_to_default(paths: &CodexAccountPaths) -> Result<AppDataDirectoryInfo, String> {
    current_data_directory_info(paths)
}

pub fn clear_all_app_data(paths: &CodexAccountPaths) -> Result<ClearAllDataResult, String> {
    if paths.codex_data_dir.exists() {
        fs::remove_dir_all(&paths.codex_data_dir)
            .map_err(|error| format!("failed to remove app codex data dir: {error}"))?;
    }

    let gemini_data_dir = paths.app_data_dir.join("gemini");
    if gemini_data_dir.exists() {
        fs::remove_dir_all(&gemini_data_dir)
            .map_err(|error| format!("failed to remove app gemini data dir: {error}"))?;
    }

    let app_settings_path = paths.app_data_dir.join("settings.json");
    if app_settings_path.exists() {
        fs::remove_file(&app_settings_path)
            .map_err(|error| format!("failed to remove app settings file: {error}"))?;
    }

    paths.ensure_dirs()?;

    let app_settings = save_app_settings(paths, AppSettings::default())?;
    let refresh_settings = save_refresh_settings(paths, CodexRefreshSettings::default())?;
    let data_directory = current_data_directory_info(paths)?;

    Ok(ClearAllDataResult {
        app_settings,
        refresh_settings,
        data_directory,
    })
}
