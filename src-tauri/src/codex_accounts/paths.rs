use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct CodexAccountPaths {
    pub app_data_dir: PathBuf,
    pub codex_data_dir: PathBuf,
    pub managed_homes_dir: PathBuf,
    pub account_index_path: PathBuf,
    pub usage_snapshot_path: PathBuf,
    pub refresh_settings_path: PathBuf,
    pub system_codex_dir: PathBuf,
    pub system_auth_path: PathBuf,
}

impl CodexAccountPaths {
    pub fn from_roots(app_data_dir: PathBuf, home_dir: PathBuf) -> Self {
        let codex_data_dir = app_data_dir.join("codex");
        let managed_homes_dir = codex_data_dir.join("managed-codex-homes");
        let account_index_path = codex_data_dir.join("accounts.json");
        let usage_snapshot_path = codex_data_dir.join("usage-snapshots.json");
        let refresh_settings_path = codex_data_dir.join("refresh-settings.json");
        let system_codex_dir = home_dir.join(".codex");
        let system_auth_path = system_codex_dir.join("auth.json");

        Self {
            app_data_dir,
            codex_data_dir,
            managed_homes_dir,
            account_index_path,
            usage_snapshot_path,
            refresh_settings_path,
            system_codex_dir,
            system_auth_path,
        }
    }

    pub fn for_test(app_data_dir: PathBuf, home_dir: PathBuf) -> Self {
        Self::from_roots(app_data_dir, home_dir)
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.codex_data_dir)
            .map_err(|error| format!("failed to create app codex dir: {error}"))?;
        fs::create_dir_all(&self.managed_homes_dir)
            .map_err(|error| format!("failed to create managed homes dir: {error}"))?;
        fs::create_dir_all(&self.system_codex_dir)
            .map_err(|error| format!("failed to create live codex dir: {error}"))?;
        Ok(())
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path.parent().ok_or_else(|| "invalid file path".to_string())?;
    fs::create_dir_all(parent).map_err(|error| format!("failed to create parent dir: {error}"))?;

    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "invalid file name".to_string())?;
    let temp_path = parent.join(format!(".{file_name}.tmp"));

    fs::write(&temp_path, bytes).map_err(|error| format!("failed to write temp file: {error}"))?;
    if path.exists() {
        fs::remove_file(path).map_err(|error| format!("failed to remove old file: {error}"))?;
    }
    fs::rename(&temp_path, path).map_err(|error| format!("failed to replace file: {error}"))?;

    Ok(())
}
