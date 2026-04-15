mod apply;
mod detect;
mod github_release;
mod install_metadata;

use std::path::PathBuf;

use apply::{apply_upgrade_action, preferred_package_manager, ApplyResult};
use detect::detect_installation;
use detect::DetectedInstall;
use github_release::fetch_latest_cli_release_version;
use install_metadata::load_install_metadata;

pub fn run(json: bool) -> Result<(), String> {
    if json {
        return Err("--json is not supported for upgrade".to_string());
    }

    let current_version = semver::Version::parse(env!("CARGO_PKG_VERSION"))
        .map_err(|error| format!("failed to parse current version: {error}"))?;
    let current_exe = std::env::current_exe()
        .map_err(|error| format!("failed to resolve current exe: {error}"))?;
    let metadata = load_install_metadata(None)?;
    let detected_install = detect_installation(&current_exe, metadata.as_ref());
    let latest_version = fetch_latest_cli_release_version("murongg/ai-accounts-hub")?;
    let action = plan_upgrade(
        current_version.clone(),
        latest_version,
        current_exe,
        detected_install,
        preferred_package_manager(),
    )?;

    match apply_upgrade_action(action, &current_version)? {
        ApplyResult::AlreadyCurrent(message)
        | ApplyResult::Upgraded(message)
        | ApplyResult::Manual(message) => {
            println!("{message}");
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpgradeAction {
    UpToDate {
        current_version: semver::Version,
    },
    RunPackageManager {
        manager: String,
        args: Vec<String>,
        target_version: semver::Version,
    },
    ReplaceBinary {
        current_exe: PathBuf,
        download_url: String,
        target_version: semver::Version,
    },
    PrintManual {
        reason: String,
        command: String,
        target_version: semver::Version,
    },
}

pub fn plan_upgrade(
    current_version: semver::Version,
    latest_version: semver::Version,
    current_exe: PathBuf,
    detected_install: DetectedInstall,
    preferred_package_manager: Option<String>,
) -> Result<UpgradeAction, String> {
    if latest_version <= current_version {
        return Ok(UpgradeAction::UpToDate { current_version });
    }

    let action = match detected_install {
        DetectedInstall::PackageManager {
            package_manager, ..
        } => {
            let Some(manager) = package_manager.or(preferred_package_manager) else {
                return Ok(UpgradeAction::PrintManual {
                    reason: "Could not upgrade automatically because no supported package manager was found on PATH.".to_string(),
                    command: "npm install -g @murongg/aah-cli@latest".to_string(),
                    target_version: latest_version,
                });
            };
            let (manager, args) = apply::package_manager_command(&manager);

            UpgradeAction::RunPackageManager {
                manager,
                args,
                target_version: latest_version,
            }
        }
        DetectedInstall::Binary { .. } => {
            let asset_name = binary_asset_name_for_target(&latest_version)?;
            let release_tag = format!("cli-v{latest_version}");
            UpgradeAction::ReplaceBinary {
                current_exe,
                download_url: format!(
                    "https://github.com/murongg/ai-accounts-hub/releases/download/{release_tag}/{asset_name}"
                ),
                target_version: latest_version,
            }
        }
        DetectedInstall::Unknown => UpgradeAction::PrintManual {
            reason: "Could not determine how this aah binary was installed.".to_string(),
            command:
                "curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh"
                    .to_string(),
            target_version: latest_version,
        },
    };

    Ok(action)
}

fn binary_asset_name_for_target(version: &semver::Version) -> Result<String, String> {
    let target = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc.exe",
        (os, arch) => return Err(format!("unsupported upgrade target {os}/{arch}")),
    };
    Ok(format!("aah_{version}_{target}"))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::detect::{detect_installation, DetectedInstall, DetectionSource};
    use super::install_metadata::{install_metadata_path_from, InstallMetadata, InstallMethod};
    use super::{
        apply::{package_manager_command, render_action_message, validate_downloaded_binary},
        github_release::{select_latest_cli_release_version, GithubReleaseSummary},
        plan_upgrade, UpgradeAction,
    };

    #[test]
    fn metadata_path_uses_platform_config_dir() {
        let root = tempfile::tempdir().expect("temp dir");
        let path =
            install_metadata_path_from(Some(root.path().to_path_buf())).expect("metadata path");
        assert_eq!(path, root.path().join("aah/cli-install.json"));
    }

    #[test]
    fn detect_installation_prefers_matching_metadata() {
        let current_exe = PathBuf::from("/usr/local/lib/node_modules/@murongg/aah-cli/vendor/aah");
        let metadata = InstallMetadata {
            schema_version: 1,
            install_method: InstallMethod::PackageManager,
            package_manager: Some("npm".to_string()),
            package_name: Some("@murongg/aah-cli".to_string()),
            binary_path: current_exe.clone(),
            install_dir: None,
            repository: None,
            installed_at: Some("2026-04-15T10:00:00Z".to_string()),
        };

        let detected = detect_installation(&current_exe, Some(&metadata));

        assert_eq!(
            detected,
            DetectedInstall::PackageManager {
                package_manager: Some("npm".to_string()),
                source: DetectionSource::Metadata,
            }
        );
    }

    #[test]
    fn detect_installation_falls_back_to_vendor_path_pattern() {
        let current_exe = PathBuf::from("/usr/local/lib/node_modules/@murongg/aah-cli/vendor/aah");
        let detected = detect_installation(&current_exe, None);

        assert_eq!(
            detected,
            DetectedInstall::PackageManager {
                package_manager: None,
                source: DetectionSource::Heuristic,
            }
        );
    }

    #[test]
    fn detect_installation_returns_unknown_for_unrecognized_paths() {
        let current_exe = PathBuf::from("/tmp/custom/aah");
        assert_eq!(
            detect_installation(&current_exe, None),
            DetectedInstall::Unknown
        );
    }

    #[test]
    fn select_latest_cli_release_version_skips_app_tags_and_prereleases() {
        let version = select_latest_cli_release_version(&[
            GithubReleaseSummary {
                tag_name: "v0.3.20".to_string(),
                draft: false,
                prerelease: false,
            },
            GithubReleaseSummary {
                tag_name: "cli-v0.1.6-beta.1".to_string(),
                draft: false,
                prerelease: true,
            },
            GithubReleaseSummary {
                tag_name: "cli-v0.1.5".to_string(),
                draft: false,
                prerelease: false,
            },
        ])
        .expect("latest version");

        assert_eq!(version, semver::Version::parse("0.1.5").unwrap());
    }

    #[test]
    fn build_manual_upgrade_for_unknown_install_method() {
        let action = plan_upgrade(
            semver::Version::parse("0.1.4").unwrap(),
            semver::Version::parse("0.1.5").unwrap(),
            PathBuf::from("/tmp/custom/aah"),
            DetectedInstall::Unknown,
            None,
        )
        .expect("plan");

        assert_eq!(
            action,
            UpgradeAction::PrintManual {
                reason: "Could not determine how this aah binary was installed.".to_string(),
                command: "curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh".to_string(),
                target_version: semver::Version::parse("0.1.5").unwrap(),
            }
        );
    }

    #[test]
    fn build_package_manager_upgrade_uses_recorded_manager() {
        let action = plan_upgrade(
            semver::Version::parse("0.1.4").unwrap(),
            semver::Version::parse("0.1.5").unwrap(),
            PathBuf::from("/usr/local/lib/node_modules/@murongg/aah-cli/vendor/aah"),
            DetectedInstall::PackageManager {
                package_manager: Some("pnpm".to_string()),
                source: DetectionSource::Metadata,
            },
            Some("pnpm".to_string()),
        )
        .expect("plan");

        assert_eq!(
            action,
            UpgradeAction::RunPackageManager {
                manager: "pnpm".to_string(),
                args: vec![
                    "add".to_string(),
                    "-g".to_string(),
                    "@murongg/aah-cli@latest".to_string()
                ],
                target_version: semver::Version::parse("0.1.5").unwrap(),
            }
        );
    }

    #[test]
    fn package_manager_command_for_yarn_uses_global_add() {
        let (manager, args) = package_manager_command("yarn");
        assert_eq!(manager, "yarn");
        assert_eq!(
            args,
            vec![
                "global".to_string(),
                "add".to_string(),
                "@murongg/aah-cli@latest".to_string()
            ]
        );
    }

    #[test]
    fn binary_upgrade_prints_install_script_when_target_dir_is_not_writable() {
        let action = UpgradeAction::PrintManual {
            reason: "Could not upgrade automatically because /usr/local/bin is not writable."
                .to_string(),
            command:
                "curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh"
                    .to_string(),
            target_version: semver::Version::parse("0.1.5").unwrap(),
        };

        let rendered = render_action_message(&action, &semver::Version::parse("0.1.4").unwrap());

        assert!(rendered
            .contains("Could not upgrade automatically because /usr/local/bin is not writable."));
        assert!(rendered.contains(
            "Run: curl -fsSL https://raw.githubusercontent.com/murongg/ai-accounts-hub/main/scripts/install-aah.sh | sh"
        ));
    }

    #[cfg(unix)]
    #[test]
    fn validate_downloaded_binary_accepts_version_flag() {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir().expect("temp dir");
        let candidate = temp.path().join("aah");
        std::fs::write(&candidate, "#!/bin/sh\necho 0.1.5\n").expect("write candidate");
        let mut permissions = std::fs::metadata(&candidate)
            .expect("metadata")
            .permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&candidate, permissions).expect("chmod");

        validate_downloaded_binary(&candidate).expect("valid binary");
    }
}
