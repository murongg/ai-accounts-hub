use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Stdio;
use std::time::{Duration, Instant};

use serde_json::Value;

use crate::cli_command_runner::cli_command;
use crate::codex_accounts::cli::resolve_codex_binary;

const TIMEOUT: Duration = Duration::from_secs(60);

pub(super) fn run(managed_home: &Path) -> Result<(), String> {
    let binary =
        resolve_codex_binary().ok_or_else(|| "Codex CLI is required for warm-up.".to_string())?;
    run_with_binary(&binary, managed_home, TIMEOUT)
}

fn run_with_binary(binary: &Path, managed_home: &Path, timeout: Duration) -> Result<(), String> {
    let workspace = tempfile::tempdir().map_err(|error| error.to_string())?;
    let stdout_path = workspace.path().join("events.jsonl");
    let stdout = std::fs::File::create(&stdout_path).map_err(|error| error.to_string())?;
    let args = [
        "--ask-for-approval",
        "never",
        "exec",
        "--sandbox",
        "read-only",
        "--skip-git-repo-check",
        "--ignore-user-config",
        "--ignore-rules",
        "--ephemeral",
        "--json",
        "--model",
        "gpt-6-luna",
        "-c",
        "model_reasoning_effort=\"low\"",
        "-c",
        "model_provider=\"openai\"",
        "-c",
        "forced_login_method=\"chatgpt\"",
        "-c",
        "cli_auth_credentials_store=\"file\"",
        "-c",
        "chatgpt_base_url=\"https://chatgpt.com/backend-api/\"",
        "Reply with exactly OK. Do not use tools.",
    ];
    let mut command = cli_command(binary, &args);
    command
        .current_dir(workspace.path())
        .env("CODEX_HOME", managed_home)
        .stdin(Stdio::null())
        .stdout(stdout)
        .stderr(Stdio::null());
    for key in [
        "OPENAI_API_KEY",
        "CODEX_API_KEY",
        "OPENAI_BASE_URL",
        "OPENAI_ORG_ID",
        "OPENAI_PROJECT_ID",
    ] {
        command.env_remove(key);
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("Could not start Codex CLI: {error}"))?;
    let started = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(Duration::from_millis(100))
            }
            _ => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(
                    "Could not confirm the Codex request. Refresh usage before trying again."
                        .into(),
                );
            }
        }
    };
    if !status.success() {
        return Err(format!(
            "Codex CLI exited with {status}. Refresh usage before trying again."
        ));
    }
    let events = std::fs::File::open(&stdout_path).map_err(|error| error.to_string())?;
    for line in BufReader::new(events).lines() {
        let line = line.map_err(|error| error.to_string())?;
        let Ok(event) = serde_json::from_str::<Value>(&line) else {
            continue;
        };
        match event["type"].as_str() {
            Some("turn.completed") => return Ok(()),
            Some("turn.failed") => break,
            _ => {}
        }
    }
    Err("Codex did not report a completed turn. Refresh usage before trying again.".into())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    #[test]
    fn uses_managed_login_in_isolated_workspace_and_requires_completed_turn() {
        let fixture = tempfile::tempdir().unwrap();
        let managed_home = fixture.path().join("managed-home");
        std::fs::create_dir(&managed_home).unwrap();
        let executable = fixture.path().join("codex");
        let fake_cli = r#"#!/bin/sh
set -eu
test "${CODEX_HOME##*/}" = managed-home
test "$PWD" != "$CODEX_HOME"
test -z "${OPENAI_API_KEY:-}"
test "$1" = --ask-for-approval
test "$2" = never
test "$3" = exec
for arg in "$@"; do
    case "$arg" in
        --ephemeral) ephemeral=1 ;;
        read-only) readonly_mode=1 ;;
    esac
done
test "$ephemeral$readonly_mode" = 11
printf '%s\n' '{"type":"EVENT"}'
"#;
        for (event, succeeds) in [("turn.failed", false), ("turn.completed", true)] {
            std::fs::write(&executable, fake_cli.replace("EVENT", event)).unwrap();
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
            let result = run_with_binary(&executable, &managed_home, Duration::from_secs(5));
            assert_eq!(result.is_ok(), succeeds, "{result:?}");
        }
    }
}
