use assert_cmd::Command;
use predicates::prelude::*;

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
