use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn tui_help_is_available_as_a_subcommand() {
    Command::cargo_bin("aah")
        .expect("binary")
        .args(["tui", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Open the interactive TUI"));
}

#[test]
fn tui_snapshot_renders_dashboard_without_entering_interactive_mode() {
    let temp = tempfile::tempdir().expect("temp dir");

    Command::cargo_bin("aah")
        .expect("binary")
        .env("HOME", temp.path())
        .env("USERPROFILE", temp.path())
        .args(["tui", "--snapshot", "--data-dir"])
        .arg(temp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("AI Accounts Hub"))
        .stdout(predicate::str::contains("Codex"))
        .stdout(predicate::str::contains("Claude"))
        .stdout(predicate::str::contains("Gemini"))
        .stdout(predicate::str::contains("Enter switch"))
        .stdout(predicate::str::contains("r refresh"))
        .stdout(predicate::str::contains("1 Codex"))
        .stdout(predicate::str::contains("2 Claude"))
        .stdout(predicate::str::contains("3 Gemini"))
        .stdout(predicate::str::contains("q/Esc quit"));
}
