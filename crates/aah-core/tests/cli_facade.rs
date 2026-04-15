use aah_core::bootstrap::{bootstrap_context, BootstrapContext};
use aah_core::claude_accounts::paths::ClaudeAccountPaths;
use aah_core::claude_usage::models::{ClaudeRateWindowSnapshot, ClaudeUsageSnapshot};
use aah_core::claude_usage::store::save_usage_snapshots as save_claude_usage_snapshots;
use aah_core::cli_facade::{CliFacade, Provider, SwitchSelection};
use aah_core::codex_accounts::paths::CodexAccountPaths;
use aah_core::codex_usage::models::{CodexUsageSnapshot, RateWindowSnapshot};
use aah_core::codex_usage::store::save_usage_snapshots as save_codex_usage_snapshots;
use aah_core::gemini_accounts::paths::GeminiAccountPaths;
use aah_core::gemini_usage::models::GeminiUsageSnapshot;
use aah_core::gemini_usage::store::save_usage_snapshots as save_gemini_usage_snapshots;
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

#[test]
fn list_exposes_account_list_quota_rows_for_each_provider() {
    let home = temp_home("quota-rows");
    let managed_root = home.join(".ai-accounts-hub");
    write_quota_fixture(&managed_root, &home);
    let facade = CliFacade::new(BootstrapContext {
        managed_root,
        user_home: home,
        import_warnings: Vec::new(),
    });

    let rows = facade.list(None).expect("list");

    let codex = rows
        .iter()
        .find(|row| row.provider == Provider::Codex)
        .expect("codex row");
    assert_eq!(
        codex
            .quota_rows
            .iter()
            .map(|row| (
                row.label.as_str(),
                row.remaining_percent,
                row.refresh_at.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("5h", Some(82), Some("1800000000")),
            ("Weekly", Some(64), Some("1800600000")),
        ]
    );
    assert_eq!(codex.quota_meta.as_deref(), Some("Credits 12.5"));

    let claude = rows
        .iter()
        .find(|row| row.provider == Provider::Claude)
        .expect("claude row");
    assert_eq!(
        claude
            .quota_rows
            .iter()
            .map(|row| (
                row.label.as_str(),
                row.remaining_percent,
                row.refresh_at.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Session", Some(73), Some("1800100000")),
            ("Weekly", Some(58), Some("1800700000")),
            ("Opus Weekly", Some(41), Some("1800800000")),
        ]
    );

    let gemini = rows
        .iter()
        .find(|row| row.provider == Provider::Gemini)
        .expect("gemini row");
    assert_eq!(
        gemini
            .quota_rows
            .iter()
            .map(|row| (
                row.label.as_str(),
                row.remaining_percent,
                row.refresh_at.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Pro", Some(91), Some("2027-01-15T00:00:00Z")),
            ("Flash", Some(77), Some("2027-01-15T01:00:00Z")),
            ("Flash Lite", Some(62), Some("2027-01-15T02:00:00Z")),
        ]
    );
}

fn write_quota_fixture(managed_root: &std::path::Path, home: &std::path::Path) {
    write_codex_quota_fixture(managed_root, home);
    write_claude_quota_fixture(managed_root, home);
    write_gemini_quota_fixture(managed_root, home);
}

fn write_codex_quota_fixture(managed_root: &std::path::Path, home: &std::path::Path) {
    let paths = CodexAccountPaths::from_roots(managed_root.to_path_buf(), home.to_path_buf());
    fs::create_dir_all(&paths.codex_data_dir).expect("codex dir");
    let managed_home = paths.managed_homes_dir.join("codex-1");
    fs::create_dir_all(&managed_home).expect("codex managed home");
    fs::write(
        &paths.account_index_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "accounts": [{
                "id": "codex-1",
                "email": "codex@example.com",
                "account_id": "acct_codex",
                "plan": "Plus",
                "managed_home_path": managed_home,
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("codex account json"),
    )
    .expect("write codex accounts");
    save_codex_usage_snapshots(
        &paths,
        &[CodexUsageSnapshot {
            managed_account_id: "codex-1".to_string(),
            plan: Some("Plus".to_string()),
            five_hour: Some(RateWindowSnapshot {
                remaining_percent: 82,
                used_percent: 18,
                reset_at: "1800000000".to_string(),
            }),
            weekly: Some(RateWindowSnapshot {
                remaining_percent: 64,
                used_percent: 36,
                reset_at: "1800600000".to_string(),
            }),
            credits_balance: Some(12.5),
            last_synced_at: Some("1799990000".to_string()),
            last_sync_error: None,
            needs_relogin: false,
        }],
    )
    .expect("save codex usage");
}

fn write_claude_quota_fixture(managed_root: &std::path::Path, home: &std::path::Path) {
    let paths = ClaudeAccountPaths::from_roots(managed_root.to_path_buf(), home.to_path_buf());
    fs::create_dir_all(&paths.claude_data_dir).expect("claude dir");
    fs::create_dir_all(&paths.managed_bundle_dir).expect("claude bundles");
    fs::write(
        &paths.metadata_index_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "accounts": [{
                "id": "claude-1",
                "email": "claude@example.com",
                "display_name": "Claude User",
                "plan": "Max",
                "account_hint": "hint",
                "credential_bundle_key": "bundle-1",
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0",
                "last_used_at": null
            }]
        }))
        .expect("claude account json"),
    )
    .expect("write claude accounts");
    save_claude_usage_snapshots(
        &paths,
        &[ClaudeUsageSnapshot {
            managed_account_id: "claude-1".to_string(),
            session: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 73,
                used_percent: 27,
                reset_at: "1800100000".to_string(),
            }),
            weekly: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 58,
                used_percent: 42,
                reset_at: "1800700000".to_string(),
            }),
            model_weekly_label: Some("Opus Weekly".to_string()),
            model_weekly: Some(ClaudeRateWindowSnapshot {
                remaining_percent: 41,
                used_percent: 59,
                reset_at: "1800800000".to_string(),
            }),
            last_synced_at: Some("1799990000".to_string()),
            last_sync_error: None,
            needs_relogin: false,
        }],
    )
    .expect("save claude usage");
}

fn write_gemini_quota_fixture(managed_root: &std::path::Path, home: &std::path::Path) {
    let paths = GeminiAccountPaths::from_roots(managed_root.to_path_buf(), home.to_path_buf());
    fs::create_dir_all(&paths.gemini_data_dir).expect("gemini dir");
    let managed_home = paths.managed_homes_dir.join("gemini-1");
    fs::create_dir_all(&managed_home).expect("gemini managed home");
    fs::write(
        &paths.account_index_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "version": 1,
            "accounts": [{
                "id": "gemini-1",
                "email": "gemini@example.com",
                "subject": "subject",
                "auth_type": "oauth-personal",
                "managed_home_path": managed_home,
                "created_at": "0",
                "updated_at": "0",
                "last_authenticated_at": "0"
            }]
        }))
        .expect("gemini account json"),
    )
    .expect("write gemini accounts");
    save_gemini_usage_snapshots(
        &paths,
        &[GeminiUsageSnapshot {
            managed_account_id: "gemini-1".to_string(),
            plan: Some("Pro".to_string()),
            pro: Some(RateWindowSnapshot {
                remaining_percent: 91,
                used_percent: 9,
                reset_at: "2027-01-15T00:00:00Z".to_string(),
            }),
            flash: Some(RateWindowSnapshot {
                remaining_percent: 77,
                used_percent: 23,
                reset_at: "2027-01-15T01:00:00Z".to_string(),
            }),
            flash_lite: Some(RateWindowSnapshot {
                remaining_percent: 62,
                used_percent: 38,
                reset_at: "2027-01-15T02:00:00Z".to_string(),
            }),
            last_synced_at: Some("1799990000".to_string()),
            last_sync_error: None,
            needs_relogin: false,
        }],
    )
    .expect("save gemini usage");
}
