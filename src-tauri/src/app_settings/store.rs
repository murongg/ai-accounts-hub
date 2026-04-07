use crate::codex_accounts::paths::{atomic_write, CodexAccountPaths};

use super::models::AppSettings;

fn app_settings_path(paths: &CodexAccountPaths) -> std::path::PathBuf {
    paths.app_data_dir.join("settings.json")
}

pub fn load_app_settings(paths: &CodexAccountPaths) -> Result<AppSettings, String> {
    paths.ensure_dirs()?;

    match std::fs::read_to_string(app_settings_path(paths)) {
        Ok(text) => serde_json::from_str::<AppSettings>(&text)
            .map_err(|error| format!("failed to parse app settings: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(AppSettings::default()),
        Err(error) => Err(format!("failed to read app settings: {error}")),
    }
}

pub fn save_app_settings(
    paths: &CodexAccountPaths,
    settings: AppSettings,
) -> Result<AppSettings, String> {
    let bytes = serde_json::to_vec_pretty(&settings)
        .map_err(|error| format!("failed to serialize app settings: {error}"))?;
    atomic_write(&app_settings_path(paths), &bytes)?;
    Ok(settings)
}
