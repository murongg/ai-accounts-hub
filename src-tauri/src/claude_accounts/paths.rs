use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ClaudeAccountPaths {
    pub app_data_dir: PathBuf,
    pub claude_data_dir: PathBuf,
    pub metadata_index_path: PathBuf,
    pub usage_snapshot_path: PathBuf,
    pub managed_bundle_dir: PathBuf,
    pub login_claude_dir: PathBuf,
    pub login_credentials_path: PathBuf,
    pub login_global_config_path: PathBuf,
    pub system_claude_dir: PathBuf,
    pub system_credentials_path: PathBuf,
    pub system_global_config_path: PathBuf,
}

impl ClaudeAccountPaths {
    pub fn from_roots(app_data_dir: PathBuf, user_home: PathBuf) -> Self {
        let claude_data_dir = app_data_dir.join("claude");
        let managed_bundle_dir = claude_data_dir.join("managed-credential-bundles");
        let metadata_index_path = claude_data_dir.join("accounts.json");
        let usage_snapshot_path = claude_data_dir.join("usage-snapshots.json");
        let login_claude_dir = claude_data_dir.join("login-session");
        let login_credentials_path = login_claude_dir.join(".credentials.json");
        let login_global_config_path = login_claude_dir.join(".claude.json");
        let system_claude_dir = user_home.join(".claude");
        let system_credentials_path = system_claude_dir.join(".credentials.json");
        let system_global_config_path = user_home.join(".claude.json");

        Self {
            app_data_dir,
            claude_data_dir,
            metadata_index_path,
            usage_snapshot_path,
            managed_bundle_dir,
            login_claude_dir,
            login_credentials_path,
            login_global_config_path,
            system_claude_dir,
            system_credentials_path,
            system_global_config_path,
        }
    }

    pub fn for_test(app_data_dir: PathBuf, user_home: PathBuf) -> Self {
        Self::from_roots(app_data_dir, user_home)
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.claude_data_dir)
            .map_err(|error| format!("failed to create app claude dir: {error}"))?;
        fs::create_dir_all(&self.managed_bundle_dir)
            .map_err(|error| format!("failed to create managed Claude bundle dir: {error}"))?;
        fs::create_dir_all(&self.login_claude_dir)
            .map_err(|error| format!("failed to create Claude login session dir: {error}"))?;
        fs::create_dir_all(&self.system_claude_dir)
            .map_err(|error| format!("failed to create live Claude dir: {error}"))?;
        Ok(())
    }
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "invalid file path".to_string())?;
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
