use ai_accounts_hub_lib::claude_accounts::models::ClaudeAccountIdentity;
use ai_accounts_hub_lib::claude_accounts::paths::ClaudeAccountPaths;
use ai_accounts_hub_lib::claude_accounts::store::ClaudeAccountStore;
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
fn upsert_creates_a_new_claude_account_entry() {
    let temp = TempDir::new("claude-store-create");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = ClaudeAccountStore::load(&paths).expect("load store");

    let saved = store
        .upsert_identity(
            &paths,
            ClaudeAccountIdentity {
                email: "murong@example.com".into(),
                display_name: Some("Murong".into()),
                plan: Some("Pro".into()),
                account_hint: Some("owner-a".into()),
            },
            "bundle-1".into(),
        )
        .expect("save account");

    assert_eq!(saved.email, "murong@example.com");
    assert_eq!(store.accounts().len(), 1);
}

#[test]
fn upsert_dedupes_existing_claude_account_by_email() {
    let temp = TempDir::new("claude-store-dedupe");
    let paths =
        ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = ClaudeAccountStore::load(&paths).expect("load store");

    let first = store
        .upsert_identity(
            &paths,
            ClaudeAccountIdentity {
                email: "murong@example.com".into(),
                display_name: Some("Murong".into()),
                plan: Some("Pro".into()),
                account_hint: Some("owner-a".into()),
            },
            "bundle-1".into(),
        )
        .expect("first save");
    let second = store
        .upsert_identity(
            &paths,
            ClaudeAccountIdentity {
                email: "murong@example.com".into(),
                display_name: Some("Murong Updated".into()),
                plan: Some("Max".into()),
                account_hint: Some("owner-a".into()),
            },
            "bundle-2".into(),
        )
        .expect("second save");

    assert_eq!(first.id, second.id);
    assert_eq!(store.accounts().len(), 1);
    assert_eq!(store.accounts()[0].credential_bundle_key, "bundle-2");
}
