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
