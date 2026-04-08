use ai_accounts_hub_lib::gemini_accounts::models::GeminiAccountIdentity;
use ai_accounts_hub_lib::gemini_accounts::paths::GeminiAccountPaths;
use ai_accounts_hub_lib::gemini_accounts::store::GeminiAccountStore;
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
fn upsert_creates_a_new_gemini_account_index_entry() {
    let temp = TempDir::new("gemini-store-create");
    let paths = GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = GeminiAccountStore::load(&paths).expect("load store");

    let saved = store
        .upsert_identity(
            &paths,
            GeminiAccountIdentity {
                email: "gemini@example.com".into(),
                subject: Some("google-sub-123".into()),
                auth_type: Some("oauth-personal".into()),
            },
            paths.managed_homes_dir.join("account-a"),
        )
        .expect("save");

    assert_eq!(saved.email, "gemini@example.com");
    assert_eq!(store.accounts().len(), 1);
}

#[test]
fn upsert_dedupes_by_subject_and_replaces_managed_home() {
    let temp = TempDir::new("gemini-store-dedupe");
    let paths = GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = GeminiAccountStore::load(&paths).expect("load store");

    let first = store
        .upsert_identity(
            &paths,
            GeminiAccountIdentity {
                email: "gemini@example.com".into(),
                subject: Some("google-sub-123".into()),
                auth_type: Some("oauth-personal".into()),
            },
            paths.managed_homes_dir.join("account-a"),
        )
        .expect("first save");
    let second = store
        .upsert_identity(
            &paths,
            GeminiAccountIdentity {
                email: "renamed@example.com".into(),
                subject: Some("google-sub-123".into()),
                auth_type: Some("oauth-personal".into()),
            },
            paths.managed_homes_dir.join("account-b"),
        )
        .expect("second save");

    assert_eq!(first.id, second.id);
    assert_eq!(store.accounts().len(), 1);
    assert!(second.managed_home_path.ends_with("account-b"));
}

#[test]
fn delete_removes_the_gemini_account_from_the_index() {
    let temp = TempDir::new("gemini-store-delete");
    let paths = GeminiAccountPaths::for_test(temp.path().join("app-data"), temp.path().join("home"));
    let mut store = GeminiAccountStore::load(&paths).expect("load store");
    let saved = store
        .upsert_identity(
            &paths,
            GeminiAccountIdentity {
                email: "gemini@example.com".into(),
                subject: Some("google-sub-123".into()),
                auth_type: Some("oauth-personal".into()),
            },
            paths.managed_homes_dir.join("account-a"),
        )
        .expect("save");

    store.delete(&paths, &saved.id).expect("delete");

    assert!(store.accounts().is_empty());
}
