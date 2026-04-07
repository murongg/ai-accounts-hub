use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
use ai_accounts_hub_lib::codex_usage::models::{
    CodexRefreshSettings, CodexUsageSnapshot, RateWindowSnapshot,
};
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
fn refresh_settings_default_to_enabled_five_minutes() {
    let temp = TempDir::new("usage-settings");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let settings = load_refresh_settings(&paths).expect("default settings");

    assert!(settings.enabled);
    assert_eq!(settings.interval_seconds, 300);
}

#[test]
fn refresh_settings_round_trip_through_disk() {
    let temp = TempDir::new("usage-settings-save");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    save_refresh_settings(
        &paths,
        CodexRefreshSettings {
            enabled: false,
            interval_seconds: 900,
        },
    )
    .expect("save settings");

    let saved = load_refresh_settings(&paths).expect("load settings");
    assert!(!saved.enabled);
    assert_eq!(saved.interval_seconds, 900);
}

#[test]
fn usage_snapshots_round_trip_through_disk() {
    let temp = TempDir::new("usage-snapshots");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let snapshots = vec![CodexUsageSnapshot {
        managed_account_id: "managed-1".into(),
        plan: Some("Pro".into()),
        five_hour: Some(RateWindowSnapshot {
            remaining_percent: 82,
            used_percent: 18,
            reset_at: "1800000000".into(),
        }),
        weekly: None,
        credits_balance: Some(10.0),
        last_synced_at: Some("1800000001".into()),
        last_sync_error: None,
        needs_relogin: false,
    }];

    save_usage_snapshots(&paths, &snapshots).expect("save usage snapshots");
    let loaded = load_usage_snapshots(&paths).expect("load usage snapshots");

    assert_eq!(loaded, snapshots);
}
