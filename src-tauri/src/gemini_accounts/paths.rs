use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct GeminiAccountPaths {
    pub gemini_data_dir: PathBuf,
    pub managed_homes_dir: PathBuf,
    pub account_index_path: PathBuf,
    pub usage_snapshot_path: PathBuf,
    pub system_gemini_dir: PathBuf,
}

impl GeminiAccountPaths {
    pub fn from_roots(app_data_dir: PathBuf, home_dir: PathBuf) -> Self {
        let gemini_data_dir = app_data_dir.join("gemini");
        let managed_homes_dir = gemini_data_dir.join("managed-gemini-homes");
        let account_index_path = gemini_data_dir.join("accounts.json");
        let usage_snapshot_path = gemini_data_dir.join("usage-snapshots.json");
        let system_gemini_dir = home_dir.join(".gemini");

        Self {
            gemini_data_dir,
            managed_homes_dir,
            account_index_path,
            usage_snapshot_path,
            system_gemini_dir,
        }
    }

    pub fn for_test(app_data_dir: PathBuf, home_dir: PathBuf) -> Self {
        Self::from_roots(app_data_dir, home_dir)
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.gemini_data_dir)
            .map_err(|error| format!("failed to create app gemini dir: {error}"))?;
        fs::create_dir_all(&self.managed_homes_dir)
            .map_err(|error| format!("failed to create managed Gemini homes dir: {error}"))?;
        fs::create_dir_all(&self.system_gemini_dir)
            .map_err(|error| format!("failed to create live Gemini dir: {error}"))?;
        Ok(())
    }
}

pub fn gemini_dir_for_home(managed_home_path: &Path) -> PathBuf {
    managed_home_path.join(".gemini")
}

pub fn oauth_creds_path_for_home(managed_home_path: &Path) -> PathBuf {
    gemini_dir_for_home(managed_home_path).join("oauth_creds.json")
}

pub fn google_accounts_path_for_home(managed_home_path: &Path) -> PathBuf {
    gemini_dir_for_home(managed_home_path).join("google_accounts.json")
}

pub fn settings_path_for_home(managed_home_path: &Path) -> PathBuf {
    gemini_dir_for_home(managed_home_path).join("settings.json")
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
