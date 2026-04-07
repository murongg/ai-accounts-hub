use std::path::{Path, PathBuf};
use std::process::Command;

pub trait CodexLoginRunner: Send + Sync {
    fn run_login(&self, managed_home: &Path) -> Result<(), String>;
}

pub struct ProcessCodexLoginRunner;

impl CodexLoginRunner for ProcessCodexLoginRunner {
    fn run_login(&self, managed_home: &Path) -> Result<(), String> {
        std::fs::create_dir_all(managed_home)
            .map_err(|error| format!("failed to create managed home: {error}"))?;

        let binary = resolve_codex_binary()
            .ok_or_else(|| "未检测到 codex 命令，请先安装 Codex CLI".to_string())?;

        let status = Command::new(binary)
            .arg("login")
            .env("CODEX_HOME", managed_home)
            .status()
            .map_err(|error| format!("failed to launch codex login: {error}"))?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("codex login exited with status {status}"))
        }
    }
}

fn resolve_codex_binary() -> Option<PathBuf> {
    which_in_path("codex").or_else(|| {
        dirs::home_dir()
            .map(|home| home.join(".local/bin/codex"))
            .filter(|path| path.exists())
    })
    .or_else(|| {
        ["/opt/homebrew/bin/codex", "/usr/local/bin/codex"]
            .into_iter()
            .map(PathBuf::from)
            .find(|path| path.exists())
    })
}

fn which_in_path(binary: &str) -> Option<PathBuf> {
    std::env::var_os("PATH").and_then(|path_var| {
        std::env::split_paths(&path_var)
            .map(|dir| dir.join(binary))
            .find(|candidate| candidate.exists())
    })
}
