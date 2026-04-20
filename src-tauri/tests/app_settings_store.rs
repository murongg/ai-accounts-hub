use ai_accounts_hub_lib::app_settings::models::{
    AppAccountsViewMode, AppLanguage, AppSettings, AppTheme, RelaySettings,
};
use ai_accounts_hub_lib::app_settings::store::{load_app_settings, save_app_settings};
use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
use serde_json::{json, to_value};
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
    assert!(settings.auto_switch_enabled);
    assert_eq!(settings.accounts_view_mode, AppAccountsViewMode::Cards);
    let settings_json = to_value(&settings).expect("settings json");
    assert_eq!(
        settings_json.get("auto_switch_five_hour_threshold_percent"),
        Some(&json!(0))
    );
    assert_eq!(
        settings_json.get("auto_switch_weekly_threshold_percent"),
        Some(&json!(0))
    );
}

#[test]
fn app_settings_default_to_relay_disabled_on_default_port() {
    let temp = TempDir::new("app-settings-relay-default");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let settings = load_app_settings(&paths).expect("default settings");

    assert_eq!(
        settings.relay,
        RelaySettings {
            enabled: false,
            port: 8765,
        }
    );
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
            auto_switch_enabled: true,
            auto_switch_five_hour_threshold_percent: 12,
            auto_switch_weekly_threshold_percent: 8,
            accounts_view_mode: AppAccountsViewMode::List,
            relay: RelaySettings::default(),
            ..AppSettings::default()
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");
    assert_eq!(loaded.language, AppLanguage::EnUs);
    assert_eq!(loaded.theme, AppTheme::Dark);
    assert!(loaded.auto_switch_enabled);
    assert_eq!(loaded.auto_switch_five_hour_threshold_percent, 12);
    assert_eq!(loaded.auto_switch_weekly_threshold_percent, 8);
    assert_eq!(loaded.accounts_view_mode, AppAccountsViewMode::List);
}

#[test]
fn app_settings_persist_mini_view_mode() {
    let temp = TempDir::new("app-settings-mini-view");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    save_app_settings(
        &paths,
        AppSettings {
            language: AppLanguage::EnUs,
            theme: AppTheme::Dark,
            auto_switch_enabled: true,
            accounts_view_mode: AppAccountsViewMode::Mini,
            relay: RelaySettings::default(),
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");
    assert_eq!(loaded.accounts_view_mode, AppAccountsViewMode::Mini);
}

#[test]
fn app_settings_round_trip_relay_settings() {
    let temp = TempDir::new("app-settings-relay-save");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    save_app_settings(
        &paths,
        AppSettings {
            language: AppLanguage::EnUs,
            theme: AppTheme::Dark,
            auto_switch_enabled: true,
            auto_switch_five_hour_threshold_percent: 0,
            auto_switch_weekly_threshold_percent: 0,
            accounts_view_mode: AppAccountsViewMode::List,
            relay: RelaySettings {
                enabled: true,
                port: 9876,
            },
            ..AppSettings::default()
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");

    assert_eq!(
        loaded.relay,
        RelaySettings {
            enabled: true,
            port: 9876,
        }
    );
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
            auto_switch_enabled: false,
            auto_switch_five_hour_threshold_percent: 9,
            auto_switch_weekly_threshold_percent: 4,
            accounts_view_mode: AppAccountsViewMode::Cards,
            relay: RelaySettings::default(),
            ..AppSettings::default()
        },
    )
    .expect("save settings");

    let loaded = load_app_settings(&paths).expect("load settings");
    assert_eq!(loaded.theme, AppTheme::System);
    assert_eq!(loaded.auto_switch_five_hour_threshold_percent, 9);
    assert_eq!(loaded.auto_switch_weekly_threshold_percent, 4);
}

#[test]
fn app_settings_loads_legacy_files_without_threshold_fields() {
    let temp = TempDir::new("app-settings-legacy");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    paths.ensure_dirs().expect("dirs");
    fs::write(
        paths.app_data_dir.join("settings.json"),
        r#"{"language":"en-US","theme":"dark"}"#,
    )
    .expect("legacy settings");

    let loaded = load_app_settings(&paths).expect("load legacy settings");

    assert_eq!(loaded.language, AppLanguage::EnUs);
    assert_eq!(loaded.theme, AppTheme::Dark);
    assert!(loaded.auto_switch_enabled);
    assert_eq!(loaded.auto_switch_five_hour_threshold_percent, 0);
    assert_eq!(loaded.auto_switch_weekly_threshold_percent, 0);
    assert_eq!(loaded.accounts_view_mode, AppAccountsViewMode::Cards);
}
