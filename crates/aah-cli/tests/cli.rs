use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;

#[test]
fn list_json_works_against_an_empty_temp_root() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["list", "--json", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("[]"));
}

#[test]
fn switch_requires_provider() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["switch", "missing@example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--provider"));
}

#[test]
fn add_requires_provider() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--provider"));
}

#[test]
fn add_help_lists_provider_flag() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["add", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--provider"))
        .stdout(predicate::str::contains(
            "Start a provider login flow and add the account to the managed pool",
        ))
        .stdout(predicate::str::contains("Provider to add"))
        .stdout(predicate::str::contains("add"));
}

#[test]
fn scriptable_subcommand_help_has_descriptions() {
    let cases = [
        (
            ["list", "--help"].as_slice(),
            "List managed accounts in the shared account pool",
        ),
        (
            ["current", "--help"].as_slice(),
            "Show the currently active account for each provider",
        ),
        (
            ["switch", "--help"].as_slice(),
            "Switch the active account for a provider",
        ),
        (
            ["refresh", "--help"].as_slice(),
            "Refresh usage and account status for managed accounts",
        ),
        (
            ["remove", "--help"].as_slice(),
            "Remove a managed account from the shared account pool",
        ),
        (
            ["label", "--help"].as_slice(),
            "Set or clear a managed account display label",
        ),
        (
            ["doctor", "--help"].as_slice(),
            "Check local CLI setup and account hub health",
        ),
        (
            ["paths", "--help"].as_slice(),
            "Show account hub data and provider paths",
        ),
        (
            ["completion", "--help"].as_slice(),
            "Generate shell completion scripts",
        ),
    ];

    for (args, description) in cases {
        Command::cargo_bin("aah")
            .expect("binary")
            .args(args)
            .assert()
            .success()
            .stdout(predicate::str::contains(description));
    }
}

#[test]
fn relay_subcommand_help_has_descriptions() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["relay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Manage the local relay server"))
        .stdout(predicate::str::contains("Show local relay runtime status"))
        .stdout(predicate::str::contains("Start the local relay server"))
        .stdout(predicate::str::contains("Stop the local relay server"))
        .stdout(predicate::str::contains("Persist the relay port setting"));

    Command::cargo_bin("aah")
        .expect("binary")
        .args(["relay", "start", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Port to bind the relay server to"));
}

#[test]
fn upgrade_help_describes_self_update_command() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["upgrade", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Upgrade the installed CLI"));
}

#[test]
fn upgrade_rejects_global_json_output() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["--json", "upgrade"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--json is not supported for upgrade",
        ));
}

#[test]
fn label_sets_account_display_label() {
    let temp = tempfile::tempdir().expect("temp dir");
    write_codex_account(temp.path(), "codex-1", "user@example.com");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args([
            "label",
            "--provider",
            "codex",
            "codex-1",
            "Work",
            "--data-dir",
        ])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Labelled Codex account user@example.com (codex-1) as \"Work\".",
        ));

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["list", "--provider", "codex", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Work <user@example.com>"));

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args([
            "label",
            "--provider",
            "codex",
            "codex-1",
            "--clear",
            "--data-dir",
        ])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Cleared label for Codex account user@example.com (codex-1).",
        ));
}

#[test]
fn remove_deletes_account_when_confirmed() {
    let temp = tempfile::tempdir().expect("temp dir");
    write_codex_account(temp.path(), "codex-1", "user@example.com");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args([
            "remove",
            "--provider",
            "codex",
            "codex-1",
            "--yes",
            "--data-dir",
        ])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed Codex account user@example.com (codex-1).",
        ));

    let index = fs::read_to_string(temp.path().join("codex/accounts.json")).expect("index");
    assert!(!index.contains("codex-1"), "{index}");
    assert!(!index.contains("user@example.com"), "{index}");
}

fn write_codex_account(root: &std::path::Path, id: &str, email: &str) {
    let codex_dir = root.join("codex");
    let managed_home = codex_dir.join("managed-codex-homes").join(id);
    fs::create_dir_all(&managed_home).expect("managed home");
    let index = format!(
        r#"{{
  "version": 1,
  "accounts": [
    {{
      "id": "{id}",
      "email": "{email}",
      "account_id": "acct-{id}",
      "plan": "Plus",
      "managed_home_path": "{}",
      "created_at": "0",
      "updated_at": "0",
      "last_authenticated_at": "0"
    }}
  ]
}}"#,
        managed_home.display()
    );
    fs::create_dir_all(&codex_dir).expect("codex dir");
    fs::write(codex_dir.join("accounts.json"), index).expect("accounts.json");
}

#[test]
fn paths_prints_managed_and_provider_paths() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["paths", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("managed root:"))
        .stdout(predicate::str::contains(temp.path().display().to_string()))
        .stdout(predicate::str::contains("codex data:"))
        .stdout(predicate::str::contains("claude data:"))
        .stdout(predicate::str::contains("gemini data:"));
}

#[test]
fn paths_does_not_auto_import_live_accounts() {
    let temp = tempfile::tempdir().expect("temp dir");
    let data_dir = temp.path().join("data");
    let codex_dir = temp.path().join(".codex");
    fs::create_dir_all(&codex_dir).expect("codex dir");
    fs::write(
        codex_dir.join("auth.json"),
        r#"{"tokens":{"account_id":"acct-test","id_token":"header.eyJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20ifQ.signature"}}"#,
    )
    .expect("auth.json");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["paths", "--data-dir"])
        .arg(&data_dir)
        .assert()
        .success();

    assert!(
        !data_dir.join("codex/accounts.json").exists(),
        "paths should not mutate account indexes"
    );
}

#[test]
fn doctor_succeeds_against_an_empty_temp_root() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["doctor", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AAH doctor"))
        .stdout(predicate::str::contains("managed root:"))
        .stdout(predicate::str::contains("relay:"))
        .stdout(predicate::str::contains("codex"))
        .stdout(predicate::str::contains("claude"))
        .stdout(predicate::str::contains("gemini"));
}

#[test]
fn completion_generates_bash_script() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["completion", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("_aah"))
        .stdout(predicate::str::contains("doctor"))
        .stdout(predicate::str::contains("paths"));
}

#[test]
fn relay_help_lists_runtime_and_config_commands() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["relay", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"))
        .stdout(predicate::str::contains("start"))
        .stdout(predicate::str::contains("stop"))
        .stdout(predicate::str::contains("set-port"));
}

#[test]
fn relay_status_json_reports_stopped_when_no_owner_is_running() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["--json", "relay", "status", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": false"))
        .stdout(predicate::str::contains("\"port\": 8765"))
        .stdout(predicate::str::contains(
            "\"codex_base_url\": \"http://127.0.0.1:8765/codex\"",
        ));
}

#[test]
fn relay_start_persists_relay_settings() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["relay", "start", "--port", "9876", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success();

    let settings = fs::read_to_string(temp.path().join("settings.json")).expect("settings.json");
    assert!(settings.contains("\"enabled\": true"), "{settings}");
    assert!(settings.contains("\"port\": 9876"), "{settings}");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["relay", "stop", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success();
}

#[test]
fn relay_start_and_stop_manage_a_single_runtime_instance() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["relay", "start", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success();

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["--json", "relay", "status", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": true"));

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["relay", "stop", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success();

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["--json", "relay", "status", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"running\": false"));

    let settings = fs::read_to_string(temp.path().join("settings.json")).expect("settings.json");
    assert!(settings.contains("\"enabled\": false"), "{settings}");
}
