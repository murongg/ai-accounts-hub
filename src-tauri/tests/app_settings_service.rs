use ai_accounts_hub_lib::app_settings::models::{AppLanguage, AppSettings, AppTheme};
use ai_accounts_hub_lib::app_settings::service::{clear_all_app_data, current_data_directory_info, reset_data_directory_to_default};
use ai_accounts_hub_lib::app_settings::store::{load_app_settings, save_app_settings};
use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
use ai_accounts_hub_lib::codex_usage::models::CodexRefreshSettings;
use ai_accounts_hub_lib::codex_usage::store::{
    load_refresh_settings, load_usage_snapshots, save_refresh_settings, save_usage_snapshots,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
        fs::create_dir_all(&path).expect("temp dir");
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn current_data_directory_points_at_default_codex_store() {
    let temp = TempDir::new("app-settings-dir");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let info = current_data_directory_info(&paths).expect("directory info");

    assert!(info.is_default);
    assert_eq!(info.current_dir, paths.codex_data_dir.display().to_string());
    assert_eq!(info.default_dir, paths.codex_data_dir.display().to_string());
}

#[test]
fn clear_all_data_resets_settings_and_removes_private_codex_data() {
    let temp = TempDir::new("app-settings-clear");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    paths.ensure_dirs().expect("dirs");

    save_app_settings(
        &paths,
        AppSettings {
            language: AppLanguage::EnUs,
            theme: AppTheme::Dark,
        },
    )
    .expect("save app settings");
    save_refresh_settings(
        &paths,
        CodexRefreshSettings {
            enabled: false,
            interval_seconds: 900,
        },
    )
    .expect("save refresh");
    save_usage_snapshots(&paths, &[]).expect("save usage snapshots");
    fs::write(&paths.account_index_path, "{\"version\":1,\"accounts\":[{\"id\":\"a1\"}]}")
        .expect("account index");
    fs::create_dir_all(paths.managed_homes_dir.join("managed-1")).expect("managed dir");
    let gemini_data_dir = paths.app_data_dir.join("gemini");
    fs::create_dir_all(gemini_data_dir.join("managed-gemini-homes").join("managed-1"))
        .expect("gemini dir");
    fs::write(gemini_data_dir.join("accounts.json"), "{\"version\":1,\"accounts\":[{\"id\":\"g1\"}]}")
        .expect("gemini account index");

    let result = clear_all_app_data(&paths).expect("clear all data");

    assert_eq!(result.app_settings.language, AppLanguage::ZhCn);
    assert_eq!(result.app_settings.theme, AppTheme::Light);
    assert!(result.refresh_settings.enabled);
    assert_eq!(result.refresh_settings.interval_seconds, 300);
    assert!(result.data_directory.is_default);
    assert!(!paths.managed_homes_dir.join("managed-1").exists());
    assert!(!gemini_data_dir.exists());
    assert!(load_usage_snapshots(&paths).expect("usage snapshots").is_empty());
    assert_eq!(load_app_settings(&paths).expect("app settings"), AppSettings::default());
    assert_eq!(
        load_refresh_settings(&paths).expect("refresh settings"),
        CodexRefreshSettings::default()
    );
}

#[test]
fn resetting_data_directory_returns_default_directory_info() {
    let temp = TempDir::new("app-settings-reset-dir");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let info = reset_data_directory_to_default(&paths).expect("reset");

    assert!(info.is_default);
    assert_eq!(info.current_dir, paths.codex_data_dir.display().to_string());
}
