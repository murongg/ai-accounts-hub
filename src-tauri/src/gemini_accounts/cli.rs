use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::{Duration, Instant};
#[cfg(target_os = "macos")]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
use crate::proxy_env::build_proxy_export_block_from_current_env;

use super::paths::oauth_creds_path_for_home;

pub trait GeminiLoginRunner: Send + Sync {
    fn run_login(&self, managed_home: &Path) -> Result<(), String>;
}

pub struct ProcessGeminiLoginRunner;

impl GeminiLoginRunner for ProcessGeminiLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        fs::create_dir_all(managed_home)
            .map_err(|error| format!("failed to create managed Gemini home: {error}"))?;

        let binary = resolve_gemini_binary()
            .ok_or_else(|| "未检测到 gemini 命令，请先安装 Gemini CLI".to_string())?;

        #[cfg(target_os = "macos")]
        {
            return run_login_via_terminal(&binary, managed_home);
        }

        #[cfg(not(target_os = "macos"))]
        {
            let status = Command::new(binary)
                .env("GEMINI_CLI_HOME", managed_home)
                .status()
                .map_err(|error| format!("failed to launch gemini login: {error}"))?;

            if !status.success() {
                return Err(format!("gemini exited with status {status}"));
            }

            wait_for_credentials(managed_home, Duration::from_secs(5))
        }
    }
}

#[cfg(target_os = "macos")]
fn run_login_via_terminal(binary: &Path, managed_home: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let script_path = std::env::temp_dir().join(format!(
        "gemini_login_{}_{}.command",
        std::process::id(),
        unique_suffix()
    ));
    let proxy_export_block = build_proxy_export_block_from_current_env();
    let script = build_terminal_login_script(binary, managed_home, &proxy_export_block);

    fs::write(&script_path, script)
        .map_err(|error| format!("failed to write Gemini login script: {error}"))?;
    fs::set_permissions(&script_path, fs::Permissions::from_mode(0o755))
        .map_err(|error| format!("failed to chmod Gemini login script: {error}"))?;

    let open_result = Command::new("open")
        .arg(&script_path)
        .status()
        .map_err(|error| format!("failed to open Terminal for Gemini login: {error}"));

    let wait_result = open_result.and_then(|status| {
        if status.success() {
            wait_for_credentials(managed_home, Duration::from_secs(300))
        } else {
            Err(format!("open exited with status {status}"))
        }
    });

    let _ = fs::remove_file(&script_path);
    wait_result
}

#[cfg(target_os = "macos")]
fn build_terminal_login_script(binary: &Path, managed_home: &Path, proxy_export_block: &str) -> String {
    let escaped_binary = shell_escape(binary);
    let escaped_home = shell_escape(managed_home);

    format!("#!/bin/bash\nexport GEMINI_CLI_HOME={escaped_home}\n{proxy_export_block}cd ~\n{escaped_binary}\n")
}

fn wait_for_credentials(managed_home: &Path, timeout: Duration) -> Result<(), String> {
    let oauth_path = oauth_creds_path_for_home(managed_home);
    let started_at = Instant::now();

    while started_at.elapsed() < timeout {
        if oauth_path.exists() {
            thread::sleep(Duration::from_millis(500));
            return Ok(());
        }
        thread::sleep(Duration::from_secs(1));
    }

    Err("Gemini 登录未完成或已超时".to_string())
}

pub fn resolve_gemini_binary() -> Option<PathBuf> {
    which_in_path("gemini")
        .or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".local/bin/gemini"))
                .filter(|path| path.exists())
        })
        .or_else(|| {
            dirs::home_dir()
                .map(|home| home.join(".bun/bin/gemini"))
                .filter(|path| path.exists())
        })
        .or_else(resolve_nvm_gemini_binary)
        .or_else(|| {
            ["/opt/homebrew/bin/gemini", "/usr/local/bin/gemini"]
                .into_iter()
                .map(PathBuf::from)
                .find(|path| path.exists())
        })
}

fn resolve_nvm_gemini_binary() -> Option<PathBuf> {
    let versions_dir = dirs::home_dir()?.join(".nvm/versions/node");
    let mut candidates = fs::read_dir(versions_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("bin/gemini"))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn which_in_path(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.exists())
    })
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
    fn macos_terminal_script_runs_gemini_with_managed_home_and_proxy_env() {
        let script = build_terminal_login_script(
            Path::new("/opt/homebrew/bin/gemini"),
            Path::new("/tmp/gemini-home"),
            "export HTTP_PROXY='http://127.0.0.1:7890'\nexport HTTPS_PROXY='http://127.0.0.1:7890'\n",
        );

        assert!(script.contains("GEMINI_CLI_HOME='/tmp/gemini-home'"));
        assert!(script.contains("'/opt/homebrew/bin/gemini'"));
        assert!(script.contains("export HTTP_PROXY='http://127.0.0.1:7890'"));
        assert!(script.contains("export HTTPS_PROXY='http://127.0.0.1:7890'"));
    }
}
