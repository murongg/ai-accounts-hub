use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(target_os = "macos")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::{Duration, Instant};
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use crate::proxy_env::build_proxy_export_block_from_current_env;

pub trait ClaudeLoginRunner: Send + Sync {
    fn run_login(&self, managed_config_dir: &Path) -> Result<(), String>;
}

pub struct ProcessClaudeLoginRunner;

impl ClaudeLoginRunner for ProcessClaudeLoginRunner {
    fn run_login(&self, managed_config_dir: &Path) -> Result<(), String> {
        fs::create_dir_all(managed_config_dir)
            .map_err(|error| format!("failed to create managed Claude config dir: {error}"))?;

        let binary = resolve_claude_binary()
            .ok_or_else(|| "未检测到 claude 命令，请先安装 Claude CLI".to_string())?;

        #[cfg(target_os = "macos")]
        {
            return run_login_via_terminal(&binary, managed_config_dir);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let status = Command::new(binary)
                .args(["auth", "login"])
                .env("CLAUDE_CONFIG_DIR", managed_config_dir)
                .status()
                .map_err(|error| format!("failed to launch claude auth login: {error}"))?;

            if status.success() {
                Ok(())
            } else {
                Err(format!("claude auth login exited with status {status}"))
            }
        }
    }
}

#[cfg(target_os = "macos")]
fn run_login_via_terminal(binary: &Path, managed_config_dir: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let script_path = std::env::temp_dir().join(format!(
        "claude_login_{}_{}.command",
        std::process::id(),
        unique_suffix()
    ));
    let success_marker = std::env::temp_dir().join(format!(
        "claude_login_success_{}_{}",
        std::process::id(),
        unique_suffix()
    ));
    let failure_marker = std::env::temp_dir().join(format!(
        "claude_login_failure_{}_{}",
        std::process::id(),
        unique_suffix()
    ));
    let proxy_export_block = build_proxy_export_block_from_current_env();

    let script = build_terminal_login_script(
        binary,
        managed_config_dir,
        &success_marker,
        &failure_marker,
        &proxy_export_block,
    );

    fs::write(&script_path, script)
        .map_err(|error| format!("failed to write Claude login script: {error}"))?;
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("failed to chmod Claude login script: {error}"))?;

    let open_result = Command::new("open")
        .arg(&script_path)
        .status()
        .map_err(|error| format!("failed to open Terminal for Claude login: {error}"));

    let wait_result = open_result.and_then(|status| {
        if status.success() {
            wait_for_terminal_markers(&success_marker, &failure_marker, Duration::from_secs(300))
        } else {
            Err(format!("open exited with status {status}"))
        }
    });

    let _ = fs::remove_file(&script_path);
    let _ = fs::remove_file(&success_marker);
    let _ = fs::remove_file(&failure_marker);
    wait_result
}

#[cfg(target_os = "macos")]
fn build_terminal_login_script(
    binary: &Path,
    managed_config_dir: &Path,
    success_marker: &Path,
    failure_marker: &Path,
    proxy_export_block: &str,
) -> String {
    let escaped_binary = shell_escape(binary);
    let escaped_config_dir = shell_escape(managed_config_dir);
    let escaped_success_marker = shell_escape(success_marker);
    let escaped_failure_marker = shell_escape(failure_marker);

    format!(
        "#!/bin/bash\nexport CLAUDE_CONFIG_DIR={escaped_config_dir}\n{proxy_export_block}mkdir -p {escaped_config_dir}\ncd ~\nif {escaped_binary} auth login; then\n  touch {escaped_success_marker}\nelse\n  touch {escaped_failure_marker}\n  exit 1\nfi\n"
    )
}

#[cfg(target_os = "macos")]
fn wait_for_terminal_markers(
    success_marker: &Path,
    failure_marker: &Path,
    timeout: Duration,
) -> Result<(), String> {
    let started_at = Instant::now();

    while started_at.elapsed() < timeout {
        if success_marker.exists() {
            return Ok(());
        }
        if failure_marker.exists() {
            return Err("Claude 登录未完成，请检查 Terminal 里的报错信息".to_string());
        }
        thread::sleep(Duration::from_secs(1));
    }

    Err("Claude 登录未完成或已超时".to_string())
}

pub fn resolve_claude_binary() -> Option<PathBuf> {
    which_in_path("claude")
        .or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".local/bin/claude"))
                .filter(|path| path.exists())
        })
        .or_else(|| {
            ["/opt/homebrew/bin/claude", "/usr/local/bin/claude"]
                .into_iter()
                .map(PathBuf::from)
                .find(|path| path.exists())
        })
}

fn which_in_path(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.exists() && is_executable(candidate))
    })
}

fn is_executable(path: &Path) -> bool {
    path.is_file()
}

#[cfg(target_os = "macos")]
fn shell_escape(path: &Path) -> String {
    let raw = path.to_string_lossy();
    format!("'{}'", raw.replace('\'', "'\\''"))
}

#[cfg(target_os = "macos")]
fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_terminal_script_runs_claude_auth_login_with_managed_config_dir() {
        let script = build_terminal_login_script(
            Path::new("/opt/homebrew/bin/claude"),
            Path::new("/tmp/claude-session"),
            Path::new("/tmp/success-marker"),
            Path::new("/tmp/failure-marker"),
            "export HTTP_PROXY='http://127.0.0.1:7890'\nexport HTTPS_PROXY='http://127.0.0.1:7890'\n",
        );

        assert!(script.contains("CLAUDE_CONFIG_DIR='/tmp/claude-session'"));
        assert!(script.contains("'/opt/homebrew/bin/claude' auth login"));
        assert!(script.contains("touch '/tmp/success-marker'"));
        assert!(script.contains("touch '/tmp/failure-marker'"));
        assert!(script.contains("export HTTP_PROXY='http://127.0.0.1:7890'"));
        assert!(script.contains("export HTTPS_PROXY='http://127.0.0.1:7890'"));
    }
}
