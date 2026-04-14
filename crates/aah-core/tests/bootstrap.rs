use aah_core::bootstrap::bootstrap_managed_root;
use aah_core::managed_root::legacy_root_candidates;
use serde_json::json;
use std::fs;
use std::path::PathBuf;

fn temp_dir(prefix: &str) -> PathBuf {
    let unique = format!("{}-{}", prefix, std::process::id());
    let path = std::env::temp_dir().join(unique);
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn bootstrap_defaults_to_home_dot_ai_accounts_hub() {
    let home = temp_dir("aah-bootstrap-home");

    let managed = bootstrap_managed_root(Some(home.clone()), None).expect("bootstrap");

    assert_eq!(managed.root, home.join(".ai-accounts-hub"));
    assert!(managed.root.exists());
}

#[test]
fn bootstrap_moves_legacy_desktop_root_once() {
    let home = temp_dir("aah-bootstrap-migrate-home");
    let legacy = legacy_root_candidates(&home).remove(0);
    fs::create_dir_all(legacy.join("codex")).expect("legacy codex");
    fs::write(
        legacy.join("codex").join("accounts.json"),
        "{\"version\":1,\"accounts\":[]}",
    )
    .expect("legacy accounts");

    let managed = bootstrap_managed_root(Some(home.clone()), None).expect("bootstrap");

    assert_eq!(managed.root, home.join(".ai-accounts-hub"));
    assert!(managed.root.join("codex").join("accounts.json").exists());
    assert!(!legacy.join("codex").join("accounts.json").exists());
}

#[test]
fn bootstrap_rewrites_managed_home_paths_after_legacy_migration() {
    let home = temp_dir("aah-bootstrap-rewrite-home");
    let legacy = legacy_root_candidates(&home).remove(0);
    let legacy_codex_home = legacy
        .join("codex")
        .join("managed-codex-homes")
        .join("account-a");
    let legacy_gemini_home = legacy
        .join("gemini")
        .join("managed-gemini-homes")
        .join("account-b");

    fs::create_dir_all(&legacy_codex_home).expect("legacy codex home");
    fs::create_dir_all(legacy_gemini_home.join(".gemini")).expect("legacy gemini home");
    fs::write(legacy_codex_home.join("auth.json"), "{}").expect("legacy codex auth");
    fs::write(
        legacy_gemini_home.join(".gemini").join("oauth_creds.json"),
        "{}",
    )
    .expect("legacy gemini auth");

    fs::create_dir_all(legacy.join("codex")).expect("legacy codex dir");
    fs::write(
        legacy.join("codex").join("accounts.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "accounts": [{
                "id": "codex-account",
                "email": "codex@example.com",
                "account_id": "acct_123",
                "plan": "Plus",
                "managed_home_path": legacy_codex_home,
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("codex accounts json"),
    )
    .expect("write codex accounts");

    fs::create_dir_all(legacy.join("gemini")).expect("legacy gemini dir");
    fs::write(
        legacy.join("gemini").join("accounts.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "accounts": [{
                "id": "gemini-account",
                "email": "gemini@example.com",
                "subject": "sub-123",
                "auth_type": "oauth-personal",
                "managed_home_path": legacy_gemini_home,
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("gemini accounts json"),
    )
    .expect("write gemini accounts");

    let managed = bootstrap_managed_root(Some(home.clone()), None).expect("bootstrap");
    let codex_accounts: serde_json::Value = serde_json::from_slice(
        &fs::read(managed.root.join("codex").join("accounts.json")).expect("codex accounts"),
    )
    .expect("parse codex accounts");
    let gemini_accounts: serde_json::Value = serde_json::from_slice(
        &fs::read(managed.root.join("gemini").join("accounts.json")).expect("gemini accounts"),
    )
    .expect("parse gemini accounts");

    assert_eq!(
        codex_accounts["accounts"][0]["managed_home_path"],
        managed
            .root
            .join("codex")
            .join("managed-codex-homes")
            .join("account-a")
            .display()
            .to_string()
    );
    assert_eq!(
        gemini_accounts["accounts"][0]["managed_home_path"],
        managed
            .root
            .join("gemini")
            .join("managed-gemini-homes")
            .join("account-b")
            .display()
            .to_string()
    );
}

#[test]
fn bootstrap_rewrites_stale_managed_home_paths_in_existing_root() {
    let home = temp_dir("aah-bootstrap-rewrite-existing-home");
    let legacy = legacy_root_candidates(&home).remove(0);
    let managed_root = home.join(".ai-accounts-hub");
    let migrated_codex_home = managed_root
        .join("codex")
        .join("managed-codex-homes")
        .join("account-a");

    fs::create_dir_all(&migrated_codex_home).expect("managed codex home");
    fs::write(migrated_codex_home.join("auth.json"), "{}").expect("managed codex auth");
    fs::create_dir_all(managed_root.join("codex")).expect("managed codex dir");
    fs::write(
        managed_root.join("codex").join("accounts.json"),
        serde_json::to_vec_pretty(&json!({
            "version": 1,
            "accounts": [{
                "id": "codex-account",
                "email": "codex@example.com",
                "account_id": "acct_123",
                "plan": "Plus",
                "managed_home_path": legacy.join("codex").join("managed-codex-homes").join("account-a"),
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("codex accounts json"),
    )
    .expect("write codex accounts");

    let managed = bootstrap_managed_root(Some(home.clone()), None).expect("bootstrap");
    let codex_accounts: serde_json::Value = serde_json::from_slice(
        &fs::read(managed.root.join("codex").join("accounts.json")).expect("codex accounts"),
    )
    .expect("parse codex accounts");

    assert_eq!(
        codex_accounts["accounts"][0]["managed_home_path"],
        managed
            .root
            .join("codex")
            .join("managed-codex-homes")
            .join("account-a")
            .display()
            .to_string()
    );
}
