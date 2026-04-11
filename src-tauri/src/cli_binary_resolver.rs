use std::ffi::{OsStr, OsString};
use std::fs;
use std::path::{Path, PathBuf};

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
                nvm_bin
                    .map(|dir| dir.join(config.binary_name))
                    .filter(|path| path.exists())
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
        .or_else(|| {
            config
                .fixed_locations
                .iter()
                .map(PathBuf::from)
                .find(|path| path.exists())
        })
}

fn resolve_under_home(home_dir: Option<&Path>, relative_paths: &[&str]) -> Option<PathBuf> {
    let home_dir = home_dir?;
    relative_paths
        .iter()
        .map(|relative_path| home_dir.join(relative_path))
        .find(|path| path.exists())
}

fn resolve_in_path(binary_name: &str, path_var: Option<&OsStr>) -> Option<PathBuf> {
    path_var.and_then(|path_var| {
        std::env::split_paths(path_var)
            .map(|dir| dir.join(binary_name))
            .find(|candidate| candidate.exists())
    })
}

fn resolve_nvm_binary(home_dir: Option<&Path>, binary_name: &str) -> Option<PathBuf> {
    let versions_dir = home_dir?.join(".nvm/versions/node");
    let mut candidates = fs::read_dir(versions_dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("bin").join(binary_name))
        .filter(|path| path.exists())
        .collect::<Vec<_>>();
    candidates.sort();
    candidates.pop()
}
