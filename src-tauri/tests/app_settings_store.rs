use ai_accounts_hub_lib::app_settings::models::{AppLanguage, AppSettings, AppTheme};
use ai_accounts_hub_lib::app_settings::store::{load_app_settings, save_app_settings};
use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
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
fn app_settings_default_to_chinese_light_theme() {
    let temp = TempDir::new("app-settings-default");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let settings = load_app_settings(&paths).expect("default settings");

    assert_eq!(settings.language, AppLanguage::ZhCn);
    assert_eq!(settings.theme, AppTheme::Light);
}

#[test]
fn app_settings_round_trip_through_disk() {
    let temp = TempDir::new("app-settings-save");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    save_app_settings(
        &paths,
        AppSettings {
            language: AppLanguage::EnUs,
            theme: AppTheme::Dark,
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");
    assert_eq!(loaded.language, AppLanguage::EnUs);
    assert_eq!(loaded.theme, AppTheme::Dark);
}

#[test]
fn app_settings_support_system_theme_round_trip() {
    let temp = TempDir::new("app-settings-system");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    save_app_settings(
        &paths,
        AppSettings {
            language: AppLanguage::ZhCn,
            theme: AppTheme::System,
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");
    assert_eq!(loaded.theme, AppTheme::System);
}
