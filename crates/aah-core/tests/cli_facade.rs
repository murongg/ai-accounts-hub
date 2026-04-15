use aah_core::bootstrap::bootstrap_context;
use aah_core::bootstrap::BootstrapContext;
use aah_core::cli_facade::{CliFacade, Provider, SwitchSelection};
use aah_core::managed_root::legacy_root_candidates;
use std::fs;
use std::path::PathBuf;

fn temp_home(prefix: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("aah-cli-facade-{}-{}", prefix, std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("temp dir");
    path
}

#[test]
fn list_current_and_refresh_run_against_the_bootstrapped_root() {
    let home = temp_home("list-current-refresh");
    let context = bootstrap_context(Some(home), None).expect("bootstrap context");
    let facade = CliFacade::new(context);

    let list = facade.list(None).expect("list");
    let current = facade.current(None).expect("current");
    let refresh = facade.refresh(None).expect("refresh");

    assert!(list.is_empty());
    assert_eq!(
        current.iter().map(|row| row.provider).collect::<Vec<_>>(),
        vec![Provider::Codex, Provider::Claude, Provider::Gemini]
    );
    assert_eq!(refresh.len(), 3);
}

#[test]
fn switch_requires_an_explicit_provider_and_selector() {
    let home = temp_home("switch");
    let context = bootstrap_context(Some(home), None).expect("bootstrap context");
    let facade = CliFacade::new(context);

    let error = facade
        .switch(
            Provider::Codex,
            SwitchSelection::Email("missing@example.com".to_string()),
        )
        .expect_err("missing account should fail");

    assert!(error.to_string().contains("missing@example.com"));
}

#[test]
fn doctor_fix_creates_provider_and_relay_directories() {
    let home = temp_home("doctor-fix-dirs");
    let managed_root = home.join(".ai-accounts-hub");
    let facade = CliFacade::new(BootstrapContext {
        managed_root: managed_root.clone(),
        user_home: home,
        import_warnings: Vec::new(),
    });

    let report = facade.doctor_fix().expect("doctor fix");

    assert!(managed_root.join("codex/managed-codex-homes").exists());
    assert!(managed_root
        .join("claude/managed-credential-bundles")
        .exists());
    assert!(managed_root.join("gemini/managed-gemini-homes").exists());
    assert!(managed_root.join("relay").exists());
    assert!(report
        .fixes
        .iter()
        .any(|fix| fix.name == "managed directories" && fix.status == "fixed"));
}

#[test]
fn doctor_fix_normalizes_stale_managed_account_paths() {
    let home = temp_home("doctor-fix-paths");
    let managed_root = home.join(".ai-accounts-hub");
    let legacy = legacy_root_candidates(&home).remove(0);
    let stale_home = legacy
        .join("codex")
        .join("managed-codex-homes")
        .join("account-a");
    fs::create_dir_all(managed_root.join("codex")).expect("codex dir");
    fs::write(
        managed_root.join("codex/accounts.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "accounts": [{
                "id": "codex-account",
                "email": "codex@example.com",
                "account_id": "acct_123",
                "plan": "Plus",
                "managed_home_path": stale_home,
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("accounts json"),
    )
    .expect("write accounts");
    let facade = CliFacade::new(BootstrapContext {
        managed_root: managed_root.clone(),
        user_home: home,
        import_warnings: Vec::new(),
    });

    let report = facade.doctor_fix().expect("doctor fix");
    let accounts: serde_json::Value =
        serde_json::from_slice(&fs::read(managed_root.join("codex/accounts.json")).unwrap())
            .expect("accounts");

    assert_eq!(
        accounts["accounts"][0]["managed_home_path"],
        managed_root
            .join("codex/managed-codex-homes/account-a")
            .display()
            .to_string()
    );
    assert!(report
        .fixes
        .iter()
        .any(|fix| fix.name == "account paths" && fix.status == "fixed"));
}

#[test]
fn doctor_fix_removes_invalid_relay_runtime_record() {
    let home = temp_home("doctor-fix-relay");
    let managed_root = home.join(".ai-accounts-hub");
    let runtime_path = managed_root.join("relay/runtime.json");
    fs::create_dir_all(runtime_path.parent().expect("runtime parent")).expect("relay dir");
    fs::write(&runtime_path, "{not-json").expect("runtime");
    let facade = CliFacade::new(BootstrapContext {
        managed_root,
        user_home: home,
        import_warnings: Vec::new(),
    });

    let report = facade.doctor_fix().expect("doctor fix");

    assert!(!runtime_path.exists());
    assert!(report
        .fixes
        .iter()
        .any(|fix| fix.name == "relay runtime" && fix.status == "fixed"));
}
