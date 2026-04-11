use ai_accounts_hub_lib::codex_accounts::paths::CodexAccountPaths;
use ai_accounts_hub_lib::codex_accounts::service::{CodexAccountService, CodexLoginRunner};
use serde_json::json;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const HEADER: &str = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
const WORK_PAYLOAD: &str = "eyJlbWFpbCI6IndvcmtAZXhhbXBsZS5jb20iLCJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9wbGFuX3R5cGUiOiJwbHVzIiwiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF8xMjMifX0";
const WORK_PAYLOAD_UPDATED: &str = "eyJlbWFpbCI6IndvcmtAZXhhbXBsZS5jb20iLCJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9wbGFuX3R5cGUiOiJwbHVzIiwiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF85OTkifX0";

fn id_token(payload: &str) -> String {
    format!("{HEADER}.{payload}.signature")
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
    auth_json: serde_json::Value,
}

impl CodexLoginRunner for FakeLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        fs::create_dir_all(managed_home).map_err(|error| error.to_string())?;
        fs::write(
            managed_home.join("auth.json"),
            serde_json::to_vec_pretty(&self.auth_json).expect("json"),
        )
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[test]
fn start_login_saves_a_managed_account() {
    let temp = TempDir::new("service-start");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let runner = FakeLoginRunner {
        auth_json: json!({
            "tokens": {
                "account_id": "acct_123",
                "id_token": id_token(WORK_PAYLOAD),
                "access_token": "access-token"
            }
        }),
    };
    let service = CodexAccountService::new(paths.clone(), Box::new(runner));

    let saved = service.start_login().expect("managed login");

    assert_eq!(saved.email, "work@example.com");
    assert!(Path::new(&saved.managed_home_path)
        .join("auth.json")
        .exists());
}

#[test]
fn start_login_replaces_existing_account_with_same_email() {
    let temp = TempDir::new("service-dedupe");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let first_service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );
    let second_service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_999",
                    "id_token": id_token(WORK_PAYLOAD_UPDATED),
                    "access_token": "access-token"
                }
            }),
        }),
    );

    let first = first_service.start_login().expect("first login");
    let second = second_service.start_login().expect("second login");

    assert_eq!(first.id, second.id);
    assert_ne!(first.managed_home_path, second.managed_home_path);
}

#[test]
fn list_accounts_marks_the_live_system_account_active() {
    let temp = TempDir::new("service-list");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );
    let saved = service.start_login().expect("save account");
    fs::create_dir_all(paths.system_codex_dir.clone()).expect("system dir");
    fs::write(
        &paths.system_auth_path,
        fs::read(Path::new(&saved.managed_home_path).join("auth.json")).expect("managed auth"),
    )
    .expect("live auth");

    let accounts = service.list_accounts().expect("list");
    assert_eq!(accounts.len(), 1);
    assert!(accounts[0].is_active);
}

#[test]
fn switch_account_overwrites_the_live_auth_file() {
    let temp = TempDir::new("service-switch");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );
    let saved = service.start_login().expect("save account");

    service.switch_account(&saved.id).expect("switch");

    let live_auth = fs::read_to_string(&paths.system_auth_path).expect("live auth");
    assert!(live_auth.contains("\"account_id\": \"acct_123\""));
}

#[test]
fn import_current_account_adds_the_live_system_codex_account() {
    let temp = TempDir::new("service-import-live");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );

    fs::create_dir_all(&paths.system_codex_dir).expect("system dir");
    fs::write(
        &paths.system_auth_path,
        serde_json::to_vec_pretty(&json!({
            "tokens": {
                "account_id": "acct_123",
                "id_token": id_token(WORK_PAYLOAD),
                "access_token": "access-token"
            }
        }))
        .expect("auth json"),
    )
    .expect("live auth");

    let imported = service
        .import_current_account_if_missing()
        .expect("import current account")
        .expect("imported account");

    assert_eq!(imported.email, "work@example.com");
    assert!(Path::new(&imported.managed_home_path)
        .join("auth.json")
        .exists());
    assert_eq!(service.list_accounts().expect("list").len(), 1);
}

#[test]
fn import_current_account_skips_existing_codex_account() {
    let temp = TempDir::new("service-import-existing");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );

    let saved = service.start_login().expect("save account");
    fs::create_dir_all(&paths.system_codex_dir).expect("system dir");
    fs::write(
        &paths.system_auth_path,
        fs::read(Path::new(&saved.managed_home_path).join("auth.json")).expect("managed auth"),
    )
    .expect("live auth");

    let imported = service
        .import_current_account_if_missing()
        .expect("import current account");

    assert!(imported.is_none());
    assert_eq!(service.list_accounts().expect("list").len(), 1);
}

#[test]
fn delete_account_removes_the_managed_home_directory() {
    let temp = TempDir::new("service-delete");
    let paths = CodexAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let service = CodexAccountService::new(
        paths.clone(),
        Box::new(FakeLoginRunner {
            auth_json: json!({
                "tokens": {
                    "account_id": "acct_123",
                    "id_token": id_token(WORK_PAYLOAD),
                    "access_token": "access-token"
                }
            }),
        }),
    );
    let saved = service.start_login().expect("save account");

    service.delete_account(&saved.id).expect("delete");

    assert!(!Path::new(&saved.managed_home_path).exists());
    assert!(service.list_accounts().expect("list").is_empty());
}
