use std::path::{Path, PathBuf};
use std::process::Command;

#[cfg(test)]
use crate::cli_binary_resolver::resolve_binary_from;
use crate::cli_binary_resolver::{resolve_binary, CliBinaryResolver};

pub trait CodexLoginRunner: Send + Sync {
    fn run_login(&self, managed_home: &Path) -> Result<(), String>;
}

pub struct ProcessCodexLoginRunner;

const CODEX_BINARY_RESOLVER: CliBinaryResolver<'static> = CliBinaryResolver {
    binary_name: "codex",
    home_relative_paths: &[".local/bin/codex"],
    fixed_locations: &["/opt/homebrew/bin/codex", "/usr/local/bin/codex"],
    include_nvm_bin_env: true,
    include_nvm_scan: true,
};

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
    resolve_binary(&CODEX_BINARY_RESOLVER)
}

#[cfg(test)]
fn resolve_codex_binary_from(
    path_var: Option<std::ffi::OsString>,
    home_dir: Option<PathBuf>,
    nvm_bin: Option<PathBuf>,
) -> Option<PathBuf> {
    resolve_binary_from(&CODEX_BINARY_RESOLVER, path_var, home_dir, nvm_bin)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn resolves_codex_from_nvm_bin_env_before_fixed_locations() {
        let home = temp_test_dir("codex-nvm-bin-home");
        let nvm_bin = home.join(".nvm/versions/node/v22.21.1/bin");
        let codex_path = nvm_bin.join("codex");
        fs::create_dir_all(&nvm_bin).unwrap();
        fs::write(&codex_path, "").unwrap();

        let resolved = resolve_codex_binary_from(
            Some(std::ffi::OsString::from("/usr/bin:/bin")),
            Some(home),
            Some(nvm_bin),
        );

        assert_eq!(resolved, Some(codex_path));
    }

    #[test]
    fn resolves_codex_from_latest_nvm_install_when_path_is_missing() {
        let home = temp_test_dir("codex-nvm-scan-home");
        let older_bin = home.join(".nvm/versions/node/v20.10.0/bin");
        let newer_bin = home.join(".nvm/versions/node/v22.21.1/bin");
        let older_codex = older_bin.join("codex");
        let newer_codex = newer_bin.join("codex");
        fs::create_dir_all(&older_bin).unwrap();
        fs::create_dir_all(&newer_bin).unwrap();
        fs::write(&older_codex, "").unwrap();
        fs::write(&newer_codex, "").unwrap();

        let resolved = resolve_codex_binary_from(
            Some(std::ffi::OsString::from("/usr/bin:/bin")),
            Some(home),
            None,
        );

        assert_eq!(resolved, Some(newer_codex));
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
