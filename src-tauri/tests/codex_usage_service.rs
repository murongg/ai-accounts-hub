use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
use ai_accounts_hub_lib::codex_accounts::service::CodexAccountService;
use ai_accounts_hub_lib::codex_usage::models::{FetchedCodexUsage, RateWindowSnapshot};
use ai_accounts_hub_lib::codex_usage::oauth::{CodexUsageFetchError, CodexUsageFetcher};
use ai_accounts_hub_lib::codex_usage::service::CodexUsageService;
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HEADER: &str = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
const PAYLOAD: &str = "eyJlbWFpbCI6IndvcmtAZXhhbXBsZS5jb20iLCJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9wbGFuX3R5cGUiOiJwbHVzIiwiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF8xMjMifX0";

fn id_token() -> String {
    format!("{HEADER}.{PAYLOAD}.signature")
}

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

#[derive(Clone)]
struct FakeLoginRunner;

impl ai_accounts_hub_lib::codex_accounts::service::CodexLoginRunner for FakeLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        fs::create_dir_all(managed_home).map_err(|error| error.to_string())?;
        fs::write(
            managed_home.join("auth.json"),
            serde_json::to_vec_pretty(&json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(),
                    "access_token": "access-token"
                }
            }))
            .expect("json"),
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

struct FakeUsageFetcher {
    result: Result<FetchedCodexUsage, CodexUsageFetchError>,
}

impl CodexUsageFetcher for FakeUsageFetcher {
    fn fetch_usage(&self, _managed_home: &Path) -> Result<FetchedCodexUsage, CodexUsageFetchError> {
        self.result.clone()
    }
}

fn create_account(paths: &CodexAccountPaths) -> String {
    let account_service = CodexAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    account_service.start_login().expect("saved").id
}

#[test]
fn refresh_all_writes_real_snapshot_fields_into_list_results() {
    let temp = TempDir::new("usage-service-success");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let account_id = create_account(&paths);
    let usage_service = CodexUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Ok(FetchedCodexUsage {
                plan: Some("Pro".into()),
                five_hour: Some(RateWindowSnapshot {
                    remaining_percent: 82,
                    used_percent: 18,
                    reset_at: "1800000000".into(),
                }),
                weekly: Some(RateWindowSnapshot {
                    remaining_percent: 88,
                    used_percent: 12,
                    reset_at: "1800500000".into(),
                }),
                credits_balance: Some(42.0),
            }),
        }),
    );

    usage_service.refresh_all().expect("refresh all");

    let account_service = CodexAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    let accounts = account_service.list_accounts().expect("list");
    let account = accounts
        .iter()
        .find(|item| item.id == account_id)
        .expect("account");

    assert_eq!(account.plan.as_deref(), Some("Pro"));
    assert_eq!(account.five_hour_remaining_percent, Some(82));
    assert_eq!(account.weekly_remaining_percent, Some(88));
    assert_eq!(account.credits_balance, Some(42.0));
    assert!(account.last_synced_at.is_some());
    assert!(account.last_sync_error.is_none());
}

#[test]
fn refresh_failure_keeps_previous_snapshot_and_marks_error() {
    let temp = TempDir::new("usage-service-error");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let account_id = create_account(&paths);

    let success_service = CodexUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Ok(FetchedCodexUsage {
                plan: Some("Plus".into()),
                five_hour: Some(RateWindowSnapshot {
                    remaining_percent: 77,
                    used_percent: 23,
                    reset_at: "1800000000".into(),
                }),
                weekly: None,
                credits_balance: None,
            }),
        }),
    );
    success_service.refresh_all().expect("initial refresh");

    let failing_service = CodexUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Err(CodexUsageFetchError::Unauthorized),
        }),
    );
    failing_service.refresh_all().expect("refresh with error");

    let account_service = CodexAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    let accounts = account_service.list_accounts().expect("list");
    let account = accounts
        .iter()
        .find(|item| item.id == account_id)
        .expect("account");

    assert_eq!(account.five_hour_remaining_percent, Some(77));
    assert_eq!(account.last_sync_error.as_deref(), Some("Codex OAuth token expired or invalid. Run `codex login` again."));
    assert_eq!(account.needs_relogin, Some(true));
}
