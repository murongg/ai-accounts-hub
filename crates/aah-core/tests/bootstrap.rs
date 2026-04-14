use aah_core::bootstrap::bootstrap_managed_root;
use aah_core::managed_root::legacy_root_candidates;
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
