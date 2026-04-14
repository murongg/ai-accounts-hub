use ai_accounts_hub_lib::claude_accounts::cli::ClaudeLoginRunner;
use ai_accounts_hub_lib::claude_accounts::keychain::InMemoryClaudeKeychainStore;
use ai_accounts_hub_lib::claude_accounts::live_credentials::{
    ClaudeLiveCredentialSnapshot, ClaudeLiveCredentialStore, InMemoryClaudeLiveCredentialStore,
};
use ai_accounts_hub_lib::claude_accounts::paths::ClaudeAccountPaths;
use ai_accounts_hub_lib::claude_accounts::service::ClaudeAccountService;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
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

struct FakeLoginRunner;

impl ClaudeLoginRunner for FakeLoginRunner {
    fn run_login(&self, _managed_config_dir: &Path) -> Result<(), String> {
        Ok(())
    }
}

fn live_snapshot(
    email: &str,
    display_name: &str,
    subscription_type: &str,
    account_hint: &str,
) -> ClaudeLiveCredentialSnapshot {
    let subscription_type = subscription_type.to_lowercase();
    ClaudeLiveCredentialSnapshot {
        credentials_json: format!(
            r#"{{"claudeAiOauth":{{"subscriptionType":"{subscription_type}"}}}}"#
        )
        .into_bytes(),
        oauth_account_json: Some(
            format!(
                r#"{{"emailAddress":"{email}","displayName":"{display_name}","accountUuid":"{account_hint}"}}"#
            )
            .into_bytes(),
        ),
    }
}

#[derive(Clone)]
struct CountingLiveCredentialStore {
    captures: Arc<AtomicUsize>,
    snapshot: Result<ClaudeLiveCredentialSnapshot, String>,
}

impl CountingLiveCredentialStore {
    fn new(snapshot: Result<ClaudeLiveCredentialSnapshot, String>) -> Self {
        Self {
            captures: Arc::new(AtomicUsize::new(0)),
            snapshot,
        }
    }

    fn capture_count(&self) -> usize {
        self.captures.load(Ordering::SeqCst)
    }
}

impl ClaudeLiveCredentialStore for CountingLiveCredentialStore {
    fn has_noninteractive_credentials_hint(&self) -> Result<bool, String> {
        Ok(false)
    }

    fn capture(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        self.captures.fetch_add(1, Ordering::SeqCst);
        self.snapshot.clone()
    }

    fn restore(
        &mut self,
        _bundle: &ai_accounts_hub_lib::claude_accounts::keychain::ClaudeCredentialBundle,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn start_login_imports_current_live_claude_account() {
    let temp = TempDir::new("claude-service-start");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);

    let account = service.start_login().expect("start login");

    assert_eq!(account.email, "murong@example.com");
    assert_eq!(account.display_name.as_deref(), Some("Murong"));
}

#[test]
fn list_accounts_marks_the_live_system_claude_account_active() {
    let temp = TempDir::new("claude-service-list");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);
    let saved = service.start_login().expect("save account");

    let accounts = service.list_accounts().expect("list accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, saved.id);
    assert!(accounts[0].is_active);
}

#[test]
fn list_accounts_does_not_capture_live_claude_credentials_when_no_accounts_are_managed() {
    let temp = TempDir::new("claude-service-list-empty");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = CountingLiveCredentialStore::new(Ok(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    )));
    let live_handle = live_store.clone();
    let service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);

    let accounts = service.list_accounts().expect("list accounts");

    assert!(accounts.is_empty());
    assert_eq!(live_handle.capture_count(), 0);
}

#[test]
fn switch_account_restores_live_claude_credentials_and_verifies_identity() {
    let temp = TempDir::new("claude-service-switch");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let live_handle = live_store.clone();
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);

    let saved = service.start_login().expect("save account");
    live_handle.set_snapshot(live_snapshot(
        "other@example.com",
        "Other",
        "Max",
        "owner-b",
    ));

    service
        .switch_account(&saved.id)
        .expect("switch should restore");

    let restored = live_handle.capture().expect("capture restored");
    let restored_json = String::from_utf8(restored.credentials_json)
        .expect("restored credentials should be utf8 json");
    let restored_account =
        String::from_utf8(restored.oauth_account_json.expect("restored oauth account"))
            .expect("restored oauth account should be utf8 json");
    assert!(restored_json.contains("\"subscriptionType\":\"pro\""));
    assert!(restored_account.contains("murong@example.com"));
}

#[test]
fn import_current_account_adds_the_live_system_claude_account() {
    let temp = TempDir::new("claude-service-import-live");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);

    let imported = service
        .import_current_account_if_missing()
        .expect("import current account")
        .expect("imported account");

    assert_eq!(imported.email, "murong@example.com");
    assert_eq!(service.list_accounts().expect("list accounts").len(), 1);
}

#[test]
fn import_current_account_skips_live_capture_without_a_claude_credentials_hint() {
    let temp = TempDir::new("claude-service-import-no-hint");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = CountingLiveCredentialStore::new(Err(
        "Claude secure storage is unavailable for test".to_string(),
    ));
    let live_handle = live_store.clone();
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);

    let imported = service
        .import_current_account_if_missing()
        .expect("import current account");

    assert!(imported.is_none());
    assert_eq!(live_handle.capture_count(), 0);
}

#[test]
fn import_current_account_skips_existing_claude_account() {
    let temp = TempDir::new("claude-service-import-existing");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);
    let saved = service.start_login().expect("save account");

    let imported = service
        .import_current_account_if_missing()
        .expect("import current account");

    assert!(imported.is_none());
    let accounts = service.list_accounts().expect("list accounts");
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id, saved.id);
}

#[test]
fn delete_account_removes_the_managed_claude_account_and_bundle() {
    let temp = TempDir::new("claude-service-delete");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let bundle_store = InMemoryClaudeKeychainStore::default();
    let live_store = InMemoryClaudeLiveCredentialStore::new(live_snapshot(
        "murong@example.com",
        "Murong",
        "Pro",
        "owner-a",
    ));
    let mut service =
        ClaudeAccountService::new(paths, Box::new(FakeLoginRunner), bundle_store, live_store);
    let saved = service.start_login().expect("save account");

    service.delete_account(&saved.id).expect("delete");

    assert!(service.list_accounts().expect("list accounts").is_empty());
}
