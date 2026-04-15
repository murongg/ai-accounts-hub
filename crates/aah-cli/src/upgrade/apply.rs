use std::path::Path;
use std::process::{Command, Stdio};

use super::UpgradeAction;

pub enum ApplyResult {
    AlreadyCurrent(String),
    Upgraded(String),
    Manual(String),
}

pub fn package_manager_command(manager: &str) -> (String, Vec<String>) {
    let args = match manager {
        "pnpm" => vec!["add", "-g", "@murongg/aah-cli@latest"],
        "yarn" => vec!["global", "add", "@murongg/aah-cli@latest"],
        "bun" => vec!["add", "-g", "@murongg/aah-cli@latest"],
        _ => vec!["install", "-g", "@murongg/aah-cli@latest"],
    };
    (
        manager.to_string(),
        args.into_iter().map(str::to_string).collect(),
    )
}

#[cfg(test)]
pub fn render_action_message(action: &UpgradeAction, current_version: &semver::Version) -> String {
    match action {
        UpgradeAction::UpToDate { current_version } => {
            format!("aah {current_version} is already up to date.")
        }
        UpgradeAction::RunPackageManager {
            manager,
            target_version,
            ..
        } => {
            format!("Upgraded aah from {current_version} to {target_version} via {manager}.")
        }
        UpgradeAction::ReplaceBinary { target_version, .. } => {
            format!("Upgraded aah from {current_version} to {target_version} via direct binary replacement.")
        }
        UpgradeAction::PrintManual {
            reason, command, ..
        } => {
            format!("{reason}\nRun: {command}")
        }
    }
}

pub fn validate_downloaded_binary(path: &Path) -> Result<(), String> {
    let version_status = Command::new(path)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to execute downloaded binary: {error}"))?;
    if version_status.success() {
        return Ok(());
    }

    let help_status = Command::new(path)
        .arg("--help")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("failed to execute downloaded binary help: {error}"))?;
    if help_status.success() {
        return Ok(());
    }

    Err(format!(
        "downloaded binary {} could not pass --version or --help validation",
        path.display()
    ))
}

pub fn preferred_package_manager() -> Option<String> {
    ["npm", "pnpm", "yarn", "bun"]
        .into_iter()
        .find(|command| {
            Command::new(command)
                .arg("--version")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .is_ok_and(|status| status.success())
        })
        .map(str::to_string)
}

pub fn apply_upgrade_action(
    action: UpgradeAction,
    current_version: &semver::Version,
) -> Result<ApplyResult, String> {
    match action {
        UpgradeAction::UpToDate { .. } => Ok(ApplyResult::AlreadyCurrent(format!(
            "aah {current_version} is already up to date."
        ))),
        UpgradeAction::RunPackageManager {
            manager,
            args,
            target_version,
        } => {
            let status = match Command::new(&manager)
                .args(args.iter().map(String::as_str))
                .status()
            {
                Ok(status) => status,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    return Ok(ApplyResult::Manual(format!(
                        "Could not upgrade automatically because {manager} is not available on PATH.\nRun: {manager} {}",
                        args.join(" ")
                    )));
                }
                Err(error) => return Err(format!("failed to launch {manager}: {error}")),
            };

            if status.success() {
                Ok(ApplyResult::Upgraded(format!(
                    "Upgraded aah from {current_version} to {target_version} via {manager}."
                )))
            } else {
                Err(format!(
                    "Could not upgrade automatically because {manager} exited unsuccessfully.\nRun: {manager} {}",
                    args.join(" ")
                ))
            }
        }
        UpgradeAction::ReplaceBinary {
            current_exe,
            download_url,
            target_version,
        } => {
            let parent = current_exe
                .parent()
                .ok_or_else(|| "current executable has no parent directory".to_string())?;
            let temp_path = parent.join(".aah.upgrade.tmp");
            let manual_command = "curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh";

            let bytes = reqwest::blocking::get(&download_url)
                .map_err(|error| format!("failed to download replacement binary: {error}"))?
                .error_for_status()
                .map_err(|error| format!("replacement download failed: {error}"))?
                .bytes()
                .map_err(|error| format!("failed to read replacement binary: {error}"))?;

            if let Err(error) = std::fs::write(&temp_path, &bytes) {
                if error.kind() == std::io::ErrorKind::PermissionDenied {
                    return Ok(ApplyResult::Manual(format!(
                        "Could not upgrade automatically because {} is not writable.\nRun: {manual_command}",
                        parent.display()
                    )));
                }
                return Err(format!("failed to write replacement binary: {error}"));
            }

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;

                let mut permissions = std::fs::metadata(&temp_path)
                    .map_err(|error| format!("failed to stat replacement binary: {error}"))?
                    .permissions();
                permissions.set_mode(0o755);
                std::fs::set_permissions(&temp_path, permissions)
                    .map_err(|error| format!("failed to chmod replacement binary: {error}"))?;
            }

            validate_downloaded_binary(&temp_path)?;

            if let Err(error) = std::fs::rename(&temp_path, &current_exe) {
                if error.kind() == std::io::ErrorKind::PermissionDenied {
                    return Ok(ApplyResult::Manual(format!(
                        "Could not upgrade automatically because {} is not writable.\nRun: {manual_command}",
                        current_exe.display()
                    )));
                }
                return Err(format!(
                    "failed to replace current executable {}: {error}",
                    current_exe.display()
                ));
            }

            Ok(ApplyResult::Upgraded(format!(
                "Upgraded aah from {current_version} to {target_version} via direct binary replacement."
            )))
        }
        UpgradeAction::PrintManual {
            reason, command, ..
        } => Ok(ApplyResult::Manual(format!("{reason}\nRun: {command}"))),
    }
}
