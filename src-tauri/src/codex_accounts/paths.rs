use std::fs;
use std::path::PathBuf;

pub use crate::fs_utils::atomic_write;

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
