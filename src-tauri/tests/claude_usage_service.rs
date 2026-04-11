use ai_accounts_hub_lib::claude_accounts::cli::ClaudeLoginRunner;
use ai_accounts_hub_lib::claude_accounts::keychain::InMemoryClaudeKeychainStore;
use ai_accounts_hub_lib::claude_accounts::live_credentials::{
    ClaudeLiveCredentialSnapshot, InMemoryClaudeLiveCredentialStore,
};
use ai_accounts_hub_lib::claude_accounts::paths::ClaudeAccountPaths;
use ai_accounts_hub_lib::claude_accounts::service::ClaudeAccountService;
use ai_accounts_hub_lib::claude_usage::cli_probe::{ClaudeCliUsageProbe, ClaudeCliUsageProbeError};
use ai_accounts_hub_lib::claude_usage::models::{ClaudeRateWindowSnapshot, FetchedClaudeUsage};
use ai_accounts_hub_lib::claude_usage::oauth::{ClaudeUsageFetchError, ClaudeUsageFetcher};
use ai_accounts_hub_lib::claude_usage::service::ClaudeUsageService;
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

#[derive(Clone)]
struct FakeLoginRunner;

impl ClaudeLoginRunner for FakeLoginRunner {
    fn run_login(&self, _managed_dir: &Path) -> Result<(), String> {
        Ok(())
    }
}

fn live_snapshot(access_token: &str, refresh_token: Option<&str>) -> ClaudeLiveCredentialSnapshot {
    let mut claude_ai_oauth = serde_json::json!({
        "accessToken": access_token,
        "subscriptionType": "pro"
    });
    if let Some(refresh_token) = refresh_token {
        claude_ai_oauth["refreshToken"] = serde_json::Value::String(refresh_token.to_string());
    }

    ClaudeLiveCredentialSnapshot {
        credentials_json: serde_json::to_vec(&serde_json::json!({
            "claudeAiOauth": claude_ai_oauth
        }))
        .expect("credentials json"),
        oauth_account_json: Some(
            serde_json::to_vec(&serde_json::json!({
                "emailAddress": "murong@example.com",
                "displayName": "Murong",
                "accountUuid": "owner-a"
            }))
            .expect("oauth account json"),
        ),
    }
}

struct SuccessOAuthFetcher;

impl ClaudeUsageFetcher for SuccessOAuthFetcher {
    fn fetch_usage(
        &self,
        _snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeUsageFetchError> {
        Ok(FetchedClaudeUsage {
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
        })
    }
}

struct FailingOAuthFetcher;

impl ClaudeUsageFetcher for FailingOAuthFetcher {
    fn fetch_usage(
        &self,
        _snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeUsageFetchError> {
        Err(ClaudeUsageFetchError::Unauthorized)
    }
}

struct SuccessCliProbe;

impl ClaudeCliUsageProbe for SuccessCliProbe {
    fn probe_usage(
        &self,
        _snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeCliUsageProbeError> {
        Ok(FetchedClaudeUsage {
            session: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 63,
                used_percent: 37,
                reset_at: "1800700000".into(),
            }),
            weekly: None,
            model_weekly_label: Some("Sonnet Weekly".into()),
            model_weekly: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 58,
                used_percent: 42,
                reset_at: "1800800000".into(),
            }),
        })
    }
}

#[test]
fn refresh_all_merges_usage_into_claude_account_list_items() {
    let temp = TempDir::new("claude-usage-service-success");
    let paths =
        ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "access-token",
        Some("refresh-token"),
    ));
    let mut account_service = ClaudeAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner),
        bundle_store.clone(),
        live_store,
    );
    let account = account_service.start_login().expect("login");

    let usage_service = ClaudeUsageService::new(
        paths.clone(),
        bundle_store,
        Box::new(SuccessOAuthFetcher),
        Box::new(SuccessCliProbe),
    );
    usage_service.refresh_all().expect("refresh");

    let accounts = account_service.list_accounts().expect("list");
    let account = accounts
        .iter()
        .find(|item| item.id == account.id)
        .expect("account");

    assert_eq!(account.session_remaining_percent, Some(82));
    assert_eq!(account.weekly_remaining_percent, Some(74));
    assert_eq!(account.model_weekly_label.as_deref(), Some("Opus Weekly"));
    assert_eq!(account.model_weekly_remaining_percent, Some(61));
    assert!(account.last_synced_at.is_some());
    assert_eq!(account.last_sync_error, None);
}

#[test]
fn oauth_failure_falls_back_to_cli_and_does_not_mark_relogin() {
    let temp = TempDir::new("claude-usage-service-cli-fallback");
    let paths =
        ClaudeAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "expired-token",
        Some("refresh-token"),
    ));
    let mut account_service = ClaudeAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner),
        bundle_store.clone(),
        live_store,
    );
    let account = account_service.start_login().expect("login");

    let usage_service = ClaudeUsageService::new(
        paths.clone(),
        bundle_store,
        Box::new(FailingOAuthFetcher),
        Box::new(SuccessCliProbe),
    );
    usage_service.refresh_all().expect("refresh");

    let accounts = account_service.list_accounts().expect("list");
    let account = accounts
        .iter()
        .find(|item| item.id == account.id)
        .expect("account");

    assert_eq!(account.session_remaining_percent, Some(63));
    assert_eq!(account.model_weekly_label.as_deref(), Some("Sonnet Weekly"));
    assert_eq!(account.needs_relogin, Some(false));
}
