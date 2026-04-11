use ai_accounts_hub_lib::codex_usage::models::RateWindowSnapshot;
use ai_accounts_hub_lib::gemini_accounts::paths::GeminiAccountPaths;
use ai_accounts_hub_lib::gemini_usage::models::GeminiUsageSnapshot;
use ai_accounts_hub_lib::gemini_usage::store::{load_usage_snapshots, save_usage_snapshots};
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
fn gemini_usage_snapshots_round_trip_through_disk() {
    let temp = TempDir::new("gemini-usage-snapshots");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));

    let snapshots = vec![GeminiUsageSnapshot {
        managed_account_id: "managed-1".into(),
        plan: Some("Paid".into()),
        pro: Some(RateWindowSnapshot {
            remaining_percent: 82,
            used_percent: 18,
            reset_at: "2026-04-09T02:30:00Z".into(),
        }),
        flash: Some(RateWindowSnapshot {
            remaining_percent: 65,
            used_percent: 35,
            reset_at: "2026-04-09T03:45:00Z".into(),
        }),
        flash_lite: Some(RateWindowSnapshot {
            remaining_percent: 58,
            used_percent: 42,
            reset_at: "2026-04-09T05:15:00Z".into(),
        }),
        last_synced_at: Some("1800000001".into()),
        last_sync_error: None,
        needs_relogin: false,
    }];

    save_usage_snapshots(&paths, &snapshots).expect("save");
    let loaded = load_usage_snapshots(&paths).expect("load");

    assert_eq!(loaded, snapshots);
}
