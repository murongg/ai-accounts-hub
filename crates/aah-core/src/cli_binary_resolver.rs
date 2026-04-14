use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const SHELL_BINARY_START_MARKER: &str = "__AIHUB_CLI_BINARY_START__";
const SHELL_BINARY_END_MARKER: &str = "__AIHUB_CLI_BINARY_END__";
const COMMAND_WRAPPER_SUFFIXES: &[&str] = &["", ".exe", ".cmd", ".bat"];
const WINDOWS_ENV_INSTALL_DIRS: &[(&str, &str)] = &[
    ("APPDATA", "npm"),
    ("LOCALAPPDATA", "pnpm"),
    ("LOCALAPPDATA", "Volta/bin"),
    ("LOCALAPPDATA", "Microsoft/WinGet/Links"),
    ("NPM_CONFIG_PREFIX", ""),
    ("PNPM_HOME", ""),
    ("SCOOP", "shims"),
    ("SCOOP_GLOBAL", "shims"),
];

pub struct CliBinaryResolver<'a> {
    pub binary_name: &'a str,
    pub home_relative_paths: &'a [&'a str],
    pub fixed_locations: &'a [&'a str],
    pub include_nvm_bin_env: bool,
    pub include_nvm_scan: bool,
}

pub fn resolve_binary(config: &CliBinaryResolver<'_>) -> Option<PathBuf> {
    resolve_binary_from(
        config,
        std::env::var_os("PATH"),
        dirs::home_dir(),
        std::env::var_os("NVM_BIN").map(PathBuf::from),
    )
    .or_else(|| resolve_binary_from_login_shell(config.binary_name))
}

pub fn resolve_binary_from(
    config: &CliBinaryResolver<'_>,
    path_var: Option<OsString>,
    home_dir: Option<PathBuf>,
    nvm_bin: Option<PathBuf>,
) -> Option<PathBuf> {
    resolve_in_path(config.binary_name, path_var.as_deref())
        .or_else(|| resolve_under_home(home_dir.as_deref(), config.home_relative_paths))
        .or_else(|| {
            if config.include_nvm_bin_env {
                nvm_bin.and_then(|dir| resolve_in_dir(&dir, config.binary_name))
            } else {
                None
            }
        })
        .or_else(|| {
            if config.include_nvm_scan {
                resolve_nvm_binary(home_dir.as_deref(), config.binary_name)
            } else {
                None
            }
        })
        .or_else(|| resolve_under_env_install_dirs(config.binary_name))
        .or_else(|| {
            config
                .fixed_locations
                .iter()
                .map(PathBuf::from)
                .flat_map(command_candidate_paths)
                .find(|path| path.exists())
        })
}

fn resolve_under_home(home_dir: Option<&Path>, relative_paths: &[&str]) -> Option<PathBuf> {
    let home_dir = home_dir?;
    relative_paths
        .iter()
        .flat_map(|relative_path| command_candidate_paths(home_dir.join(relative_path)))
        .find(|path| path.exists())
}

fn resolve_under_env_install_dirs(binary_name: &str) -> Option<PathBuf> {
    WINDOWS_ENV_INSTALL_DIRS
        .iter()
        .filter_map(|(env_var, relative_dir)| {
            let root = std::env::var_os(env_var)?;
            if root.is_empty() {
                return None;
            }

            let root = PathBuf::from(root);
            let candidate = if relative_dir.is_empty() {
                root.join(binary_name)
            } else {
                root.join(relative_dir).join(binary_name)
            };
            Some(candidate)
        })
        .flat_map(command_candidate_paths)
        .find(|path| path.exists())
}

fn resolve_in_path(binary_name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    path_var.and_then(|path_var| {
        std::env::split_paths(path_var)
            .flat_map(|dir| command_candidate_paths(dir.join(binary_name)))
            .find(|candidate| candidate.exists())
    })
}

fn resolve_nvm_binary(home_dir: Option<&Path>, binary_name: &str) -> Option<PathBuf> {
    let versions_dir = home_dir?.join(".nvm/versions/node");
    let mut candidates = fs::read_dir(versions_dir)
        .ok()?
        .filter_map(Result::ok)
        .flat_map(|entry| command_candidate_paths(entry.path().join("bin").join(binary_name)))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}

fn resolve_in_dir(dir: &Path, binary_name: &str) -> Option<PathBuf> {
    command_candidate_paths(dir.join(binary_name))
        .into_iter()
        .find(|path| path.exists())
}

fn command_candidate_paths(base: PathBuf) -> Vec<PathBuf> {
    let mut candidates = Vec::with_capacity(COMMAND_WRAPPER_SUFFIXES.len());
    let base_string = base.to_string_lossy().to_ascii_lowercase();

    for suffix in COMMAND_WRAPPER_SUFFIXES {
        if suffix.is_empty() || base_string.ends_with(suffix) {
            candidates.push(base.clone());
            continue;
        }

        let mut candidate = base.as_os_str().to_os_string();
        candidate.push(suffix);
        candidates.push(PathBuf::from(candidate));
    }

    candidates
}

fn resolve_binary_from_login_shell(binary_name: &str) -> Option<PathBuf> {
    if !should_probe_login_shell() {
        return None;
    }

    let shell = resolve_login_shell();
    let command = build_login_shell_binary_probe_command(binary_name);
    let output = Command::new(shell).args(["-ilc", &command]).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout);

    parse_login_shell_binary_output(&stdout).filter(|path| path.exists())
}

fn should_probe_login_shell() -> bool {
    should_probe_login_shell_for_os(cfg!(windows))
}

fn should_probe_login_shell_for_os(is_windows: bool) -> bool {
    !is_windows
}

fn resolve_login_shell() -> PathBuf {
    std::env::var_os("SHELL")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/bin/zsh"))
}

fn build_login_shell_binary_probe_command(binary_name: &str) -> String {
    format!(
        "printf '%s\\n' '{SHELL_BINARY_START_MARKER}'; command -v {binary_name}; printf '%s\\n' '{SHELL_BINARY_END_MARKER}'"
    )
}

fn parse_login_shell_binary_output(output: &str) -> Option<PathBuf> {
    let mut inside_markers = false;

    for line in output.lines().map(str::trim) {
        if line == SHELL_BINARY_START_MARKER {
            inside_markers = true;
            continue;
        }
        if line == SHELL_BINARY_END_MARKER {
            break;
        }
        if !inside_markers || line.is_empty() {
            continue;
        }

        return Some(PathBuf::from(line));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsString;
    use std::sync::Mutex;

    #[test]
    fn parses_login_shell_binary_lookup_between_markers() {
        let output = "\
zsh startup noise
__AIHUB_CLI_BINARY_START__
/Users/murong/.vite-plus/bin/gemini
__AIHUB_CLI_BINARY_END__
more noise
";

        assert_eq!(
            parse_login_shell_binary_output(output),
            Some(PathBuf::from("/Users/murong/.vite-plus/bin/gemini")),
        );
    }

    #[test]
    fn builds_login_shell_probe_with_command_v() {
        let command = build_login_shell_binary_probe_command("gemini");

        assert!(command.contains("command -v gemini"));
        assert!(command.contains("__AIHUB_CLI_BINARY_START__"));
        assert!(command.contains("__AIHUB_CLI_BINARY_END__"));
    }

    #[test]
    fn resolves_windows_cmd_wrapper_from_path() {
        let temp = temp_test_dir("resolver-windows-path");
        let bin_dir = temp.join("AppData/Roaming/npm");
        let codex_cmd = bin_dir.join("codex.cmd");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::write(&codex_cmd, "").unwrap();
        let path_var = std::env::join_paths([bin_dir]).unwrap();
        let config = CliBinaryResolver {
            binary_name: "codex",
            home_relative_paths: &[],
            fixed_locations: &[],
            include_nvm_bin_env: false,
            include_nvm_scan: false,
        };

        let resolved = resolve_binary_from(&config, Some(path_var), None, None);

        assert_eq!(resolved, Some(codex_cmd));
    }

    #[test]
    fn skips_login_shell_probe_on_windows() {
        assert!(!should_probe_login_shell_for_os(true));
        assert!(should_probe_login_shell_for_os(false));
    }

    #[test]
    fn resolves_windows_localappdata_volta_bin_without_home_dir() {
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = temp_test_dir("resolver-windows-volta-localappdata");
        let local_app_data = temp.join("LocalAppData");
        let volta_bin = local_app_data.join("Volta/bin");
        let codex_cmd = volta_bin.join("codex.cmd");
        fs::create_dir_all(&volta_bin).unwrap();
        fs::write(&codex_cmd, "").unwrap();
        let _env = EnvVarGuard::set("LOCALAPPDATA", &local_app_data);

        let resolved = resolve_binary_from(
            &test_resolver("codex"),
            Some(OsString::from("/usr/bin:/bin")),
            None,
            None,
        );

        assert_eq!(resolved, Some(codex_cmd));
    }

    #[test]
    fn resolves_windows_localappdata_winget_links_without_home_dir() {
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = temp_test_dir("resolver-windows-winget-localappdata");
        let local_app_data = temp.join("LocalAppData");
        let winget_links = local_app_data.join("Microsoft/WinGet/Links");
        let gemini_exe = winget_links.join("gemini.exe");
        fs::create_dir_all(&winget_links).unwrap();
        fs::write(&gemini_exe, "").unwrap();
        let _env = EnvVarGuard::set("LOCALAPPDATA", &local_app_data);

        let resolved = resolve_binary_from(
            &test_resolver("gemini"),
            Some(OsString::from("/usr/bin:/bin")),
            None,
            None,
        );

        assert_eq!(resolved, Some(gemini_exe));
    }

    #[test]
    fn resolves_windows_scoop_env_without_home_dir() {
        let _lock = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp = temp_test_dir("resolver-windows-scoop-env");
        let scoop_root = temp.join("scoop");
        let scoop_shims = scoop_root.join("shims");
        let claude_cmd = scoop_shims.join("claude.cmd");
        fs::create_dir_all(&scoop_shims).unwrap();
        fs::write(&claude_cmd, "").unwrap();
        let _env = EnvVarGuard::set("SCOOP", &scoop_root);

        let resolved = resolve_binary_from(
            &test_resolver("claude"),
            Some(OsString::from("/usr/bin:/bin")),
            None,
            None,
        );

        assert_eq!(resolved, Some(claude_cmd));
    }

    fn test_resolver(binary_name: &'static str) -> CliBinaryResolver<'static> {
        CliBinaryResolver {
            binary_name,
            home_relative_paths: &[],
            fixed_locations: &[],
            include_nvm_bin_env: false,
            include_nvm_scan: false,
        }
    }

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &Path) -> Self {
            let previous = std::env::var_os(key);
            std::env::set_var(key, value);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }
}
