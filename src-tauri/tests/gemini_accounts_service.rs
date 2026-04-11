use ai_accounts_hub_lib::gemini_accounts::paths::GeminiAccountPaths;
use ai_accounts_hub_lib::gemini_accounts::service::{GeminiAccountService, GeminiLoginRunner};
use ai_accounts_hub_lib::gemini_usage::models::GeminiUsageSnapshot;
use ai_accounts_hub_lib::gemini_usage::store::{load_usage_snapshots, save_usage_snapshots};
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
struct FakeLoginRunner {
    oauth_creds: serde_json::Value,
    google_accounts: serde_json::Value,
    settings: serde_json::Value,
}

impl GeminiLoginRunner for FakeLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        let gemini_dir = managed_home.join(".gemini");
        fs::create_dir_all(&gemini_dir).map_err(|error| error.to_string())?;
        fs::write(
            gemini_dir.join("oauth_creds.json"),
            serde_json::to_vec_pretty(&self.oauth_creds).expect("oauth json"),
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            gemini_dir.join("google_accounts.json"),
            serde_json::to_vec_pretty(&self.google_accounts).expect("accounts json"),
        )
        .map_err(|error| error.to_string())?;
        fs::write(
            gemini_dir.join("settings.json"),
            serde_json::to_vec_pretty(&self.settings).expect("settings json"),
        )
        .map_err(|error| error.to_string())?;
        fs::create_dir_all(gemini_dir.join("history")).map_err(|error| error.to_string())?;
        fs::write(gemini_dir.join("history").join("session.log"), "history")
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

fn fake_runner(email: &str, subject: &str) -> FakeLoginRunner {
    FakeLoginRunner {
        oauth_creds: json!({
            "access_token": "access-token",
            "refresh_token": "refresh-token",
            "id_token": id_token(json!({
                "email": email,
                "sub": subject
            })),
        }),
        google_accounts: json!({
            "active": email,
            "old": []
        }),
        settings: json!({
            "security": {
                "auth": {
                    "selectedType": "oauth-personal"
                }
            }
        }),
    }
}

#[test]
fn start_login_saves_a_managed_gemini_account() {
    let temp = TempDir::new("gemini-service-start");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );

    let saved = service.start_login().expect("managed login");

    assert_eq!(saved.email, "gemini@example.com");
    assert!(Path::new(&saved.managed_home_path)
        .join(".gemini")
        .join("oauth_creds.json")
        .exists());
}

#[test]
fn list_accounts_marks_the_live_system_gemini_account_active() {
    let temp = TempDir::new("gemini-service-list");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );
    let saved = service.start_login().expect("save account");
    let managed_gemini_dir = Path::new(&saved.managed_home_path).join(".gemini");
    fs::create_dir_all(&paths.system_gemini_dir).expect("system dir");
    fs::write(
        paths.system_gemini_dir.join("oauth_creds.json"),
        fs::read(managed_gemini_dir.join("oauth_creds.json")).expect("managed oauth"),
    )
    .expect("live oauth");
    fs::write(
        paths.system_gemini_dir.join("settings.json"),
        fs::read(managed_gemini_dir.join("settings.json")).expect("managed settings"),
    )
    .expect("live settings");

    let accounts = service.list_accounts().expect("list");
    assert_eq!(accounts.len(), 1);
    assert!(accounts[0].is_active);
}

#[test]
fn switch_account_overwrites_only_live_auth_files() {
    let temp = TempDir::new("gemini-service-switch");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );
    let saved = service.start_login().expect("save account");
    fs::create_dir_all(&paths.system_gemini_dir).expect("system dir");
    fs::write(paths.system_gemini_dir.join("history.txt"), "keep-me").expect("history");

    service.switch_account(&saved.id).expect("switch");

    let live_oauth =
        fs::read_to_string(paths.system_gemini_dir.join("oauth_creds.json")).expect("live oauth");
    let live_settings =
        fs::read_to_string(paths.system_gemini_dir.join("settings.json")).expect("live settings");
    assert!(live_oauth.contains("refresh-token"));
    assert!(live_settings.contains("oauth-personal"));
    assert_eq!(
        fs::read_to_string(paths.system_gemini_dir.join("history.txt")).expect("history"),
        "keep-me"
    );
}

#[test]
fn delete_account_removes_the_managed_home_directory() {
    let temp = TempDir::new("gemini-service-delete");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );
    let saved = service.start_login().expect("save account");

    service.delete_account(&saved.id).expect("delete");

    assert!(!Path::new(&saved.managed_home_path).exists());
    assert!(service.list_accounts().expect("list").is_empty());
}

#[test]
fn start_login_clears_stale_relogin_state_for_matching_account() {
    let temp = TempDir::new("gemini-service-clear-relogin");
    let paths =
        GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let first_service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );
    let saved = first_service.start_login().expect("initial save");

    save_usage_snapshots(
        &paths,
        &[GeminiUsageSnapshot {
            managed_account_id: saved.id.clone(),
            plan: Some("Paid".into()),
            pro: None,
            flash: None,
            flash_lite: None,
            last_synced_at: Some("1800000001".into()),
            last_sync_error: Some(
                "Gemini OAuth token expired or invalid. Run `gemini` again.".into(),
            ),
            needs_relogin: true,
        }],
    )
    .expect("seed usage snapshot");

    let second_service = GeminiAccountService::new(
        paths.clone(),
        Box::new(fake_runner("gemini@example.com", "sub-123")),
    );
    let saved_again = second_service.start_login().expect("re-login");

    assert_eq!(saved_again.id, saved.id);
    let snapshots = load_usage_snapshots(&paths).expect("load snapshots");
    assert_eq!(snapshots.len(), 1);
    assert!(!snapshots[0].needs_relogin);
    assert_eq!(snapshots[0].last_sync_error, None);
}
