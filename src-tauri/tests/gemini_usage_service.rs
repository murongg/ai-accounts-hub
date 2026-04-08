use ai_accounts_hub_lib::codex_usage::models::RateWindowSnapshot;
use ai_accounts_hub_lib::gemini_accounts::paths::GeminiAccountPaths;
use ai_accounts_hub_lib::gemini_accounts::service::GeminiAccountService;
use ai_accounts_hub_lib::gemini_usage::models::FetchedGeminiUsage;
use ai_accounts_hub_lib::gemini_usage::oauth::{GeminiUsageFetchError, GeminiUsageFetcher};
use ai_accounts_hub_lib::gemini_usage::service::GeminiUsageService;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn id_token(claims: serde_json::Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("claims"));
    format!("{header}.{payload}.signature")
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

impl ai_accounts_hub_lib::gemini_accounts::service::GeminiLoginRunner for FakeLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        let gemini_dir = managed_home.join(".gemini");
        fs::create_dir_all(&gemini_dir).map_err(|error| error.to_string())?;
        fs::write(
            gemini_dir.join("oauth_creds.json"),
            serde_json::to_vec_pretty(&json!({
                "access_token": "access-token",
                "refresh_token": "refresh-token",
                "id_token": id_token(json!({
                    "email": "gemini@example.com",
                    "sub": "sub-123"
                }))
            }))
            .expect("json"),
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            gemini_dir.join("settings.json"),
            serde_json::to_vec_pretty(&json!({
                "security": {
                    "auth": {
                        "selectedType": "oauth-personal"
                    }
                }
            }))
            .expect("settings json"),
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

struct FakeUsageFetcher {
    result: Result<FetchedGeminiUsage, GeminiUsageFetchError>,
}

impl GeminiUsageFetcher for FakeUsageFetcher {
    fn fetch_usage(&self, _managed_home: &Path) -> Result<FetchedGeminiUsage, GeminiUsageFetchError> {
        self.result.clone()
    }
}

fn create_account(paths: &GeminiAccountPaths) -> String {
    let account_service = GeminiAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    account_service.start_login().expect("saved").id
}

#[test]
fn refresh_all_writes_real_gemini_snapshot_fields_into_list_results() {
    let temp = TempDir::new("gemini-usage-service-success");
    let paths = GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let account_id = create_account(&paths);
    let usage_service = GeminiUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Ok(FetchedGeminiUsage {
                plan: Some("Workspace".into()),
                pro: Some(RateWindowSnapshot {
                    remaining_percent: 91,
                    used_percent: 9,
                    reset_at: "2026-04-09T00:15:00Z".into(),
                }),
                flash: Some(RateWindowSnapshot {
                    remaining_percent: 73,
                    used_percent: 27,
                    reset_at: "2026-04-09T04:45:00Z".into(),
                }),
                flash_lite: Some(RateWindowSnapshot {
                    remaining_percent: 64,
                    used_percent: 36,
                    reset_at: "2026-04-09T06:15:00Z".into(),
                }),
            }),
        }),
    );

    usage_service.refresh_all().expect("refresh all");

    let account_service = GeminiAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    let accounts = account_service.list_accounts().expect("list");
    let account = accounts.iter().find(|item| item.id == account_id).expect("account");

    assert_eq!(account.plan.as_deref(), Some("Workspace"));
    assert_eq!(account.pro_remaining_percent, Some(91));
    assert_eq!(account.flash_remaining_percent, Some(73));
    assert_eq!(account.flash_lite_remaining_percent, Some(64));
    assert!(account.last_synced_at.is_some());
    assert!(account.last_sync_error.is_none());
}

#[test]
fn refresh_failure_keeps_previous_snapshot_and_marks_error() {
    let temp = TempDir::new("gemini-usage-service-error");
    let paths = GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let account_id = create_account(&paths);

    let success_service = GeminiUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Ok(FetchedGeminiUsage {
                plan: Some("Paid".into()),
                pro: Some(RateWindowSnapshot {
                    remaining_percent: 88,
                    used_percent: 12,
                    reset_at: "2026-04-09T02:30:00Z".into(),
                }),
                flash: None,
                flash_lite: None,
            }),
        }),
    );
    success_service.refresh_all().expect("initial refresh");

    let failing_service = GeminiUsageService::new(
        paths.clone(),
        Box::new(FakeUsageFetcher {
            result: Err(GeminiUsageFetchError::unauthorized("quota endpoint returned 401")),
        }),
    );
    failing_service.refresh_all().expect("refresh with error");

    let account_service = GeminiAccountService::new(paths.clone(), Box::new(FakeLoginRunner));
    let accounts = account_service.list_accounts().expect("list");
    let account = accounts.iter().find(|item| item.id == account_id).expect("account");

    assert_eq!(account.pro_remaining_percent, Some(88));
    assert_eq!(
        account.last_sync_error.as_deref(),
        Some("Gemini OAuth token expired or invalid (quota endpoint returned 401). Run `gemini` again.")
    );
    assert_eq!(account.needs_relogin, Some(true));
}
