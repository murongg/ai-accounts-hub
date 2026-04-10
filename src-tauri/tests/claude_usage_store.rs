use ai_accounts_hub_lib::claude_accounts::paths::ClaudeAccountPaths;
use ai_accounts_hub_lib::claude_usage::models::{
    ClaudeRateWindowSnapshot, ClaudeUsageSnapshot, FetchedClaudeUsage,
};
use ai_accounts_hub_lib::claude_usage::store::{load_usage_snapshots, save_usage_snapshots, ClaudeUsageStore};
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
fn usage_snapshots_round_trip_through_disk() {
    let temp = TempDir::new("claude-usage-snapshots");
    let paths = ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let snapshots = vec![ClaudeUsageSnapshot {
        managed_account_id: "managed-1".into(),
        session: Some(ClaudeRateWindowSnapshot {
            remaining_percent: 82,
            used_percent: 18,
            reset_at: "1800000000".into(),
        }),
        weekly: Some(ClaudeRateWindowSnapshot {
            remaining_percent: 74,
            used_percent: 26,
            reset_at: "1800500000".into(),
        }),
        model_weekly_label: Some("Opus Weekly".into()),
        model_weekly: Some(ClaudeRateWindowSnapshot {
            remaining_percent: 61,
            used_percent: 39,
            reset_at: "1800600000".into(),
        }),
        last_synced_at: Some("1800000001".into()),
        last_sync_error: None,
        needs_relogin: false,
    }];

    save_usage_snapshots(&paths, &snapshots).expect("save");
    let loaded = load_usage_snapshots(&paths).expect("load");

    assert_eq!(loaded, snapshots);
}

#[test]
fn upsert_success_persists_session_weekly_and_model_windows() {
    let temp = TempDir::new("claude-usage-store-success");
    let paths = ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = ClaudeUsageStore::load(&paths).expect("load");

    store.upsert_success(
        "claude-1",
        FetchedClaudeUsage {
            session: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 82,
                used_percent: 18,
                reset_at: "1800000000".into(),
            }),
            weekly: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 74,
                used_percent: 26,
                reset_at: "1800500000".into(),
            }),
            model_weekly_label: Some("Opus Weekly".into()),
            model_weekly: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 61,
                used_percent: 39,
                reset_at: "1800600000".into(),
            }),
        },
    );
    store.persist(&paths).expect("persist");

    let reloaded = ClaudeUsageStore::load(&paths).expect("reload");
    let snapshot = reloaded.get("claude-1").expect("snapshot");

    assert_eq!(
        snapshot.session.as_ref().map(|window| window.remaining_percent),
        Some(82)
    );
    assert_eq!(
        snapshot.weekly.as_ref().map(|window| window.remaining_percent),
        Some(74)
    );
    assert_eq!(snapshot.model_weekly_label.as_deref(), Some("Opus Weekly"));
    assert_eq!(
        snapshot
            .model_weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(61)
    );
    assert!(snapshot.last_synced_at.is_some());
    assert!(snapshot.last_sync_error.is_none());
    assert!(!snapshot.needs_relogin);
}

#[test]
fn upsert_error_keeps_previous_quota_and_marks_relogin() {
    let temp = TempDir::new("claude-usage-store-error");
    let paths = ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = ClaudeUsageStore::load(&paths).expect("load");

    store.upsert_success(
        "claude-1",
        FetchedClaudeUsage {
            session: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 45,
                used_percent: 55,
                reset_at: "1800000000".into(),
            }),
            weekly: None,
            model_weekly_label: None,
            model_weekly: None,
        },
    );
    store.upsert_error("claude-1", "oauth expired".into(), true);

    let snapshot = store.get("claude-1").expect("snapshot");
    assert_eq!(
        snapshot.session.as_ref().map(|window| window.remaining_percent),
        Some(45)
    );
    assert_eq!(snapshot.last_sync_error.as_deref(), Some("oauth expired"));
    assert!(snapshot.needs_relogin);
}

#[test]
fn retain_only_removes_deleted_accounts() {
    let temp = TempDir::new("claude-usage-store-retain");
    let paths = ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = ClaudeUsageStore::load(&paths).expect("load");

    store.upsert_error("keep-me", "temporary".into(), false);
    store.upsert_error("drop-me", "temporary".into(), false);
    store.retain_only(&["keep-me".to_string()]);

    assert!(store.get("keep-me").is_some());
    assert!(store.get("drop-me").is_none());
}
