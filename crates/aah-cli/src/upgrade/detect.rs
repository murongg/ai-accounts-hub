use std::path::{Path, PathBuf};

use super::install_metadata::{InstallMetadata, InstallMethod};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectionSource {
    Metadata,
    Heuristic,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DetectedInstall {
    PackageManager {
        package_manager: Option<String>,
        source: DetectionSource,
    },
    Binary {
        install_dir: Option<PathBuf>,
        source: DetectionSource,
    },
    Unknown,
}

pub fn detect_installation(
    current_exe: &Path,
    metadata: Option<&InstallMetadata>,
) -> DetectedInstall {
    if let Some(metadata) = metadata.filter(|metadata| metadata.binary_path == current_exe) {
        return match metadata.install_method {
            InstallMethod::PackageManager => DetectedInstall::PackageManager {
                package_manager: metadata.package_manager.clone(),
                source: DetectionSource::Metadata,
            },
            InstallMethod::Binary => DetectedInstall::Binary {
                install_dir: metadata.install_dir.clone(),
                source: DetectionSource::Metadata,
            },
        };
    }

    let normalized = current_exe.to_string_lossy().replace('\\', "/");
    if normalized.contains("node_modules/@murongg/aah-cli/vendor/aah") {
        return DetectedInstall::PackageManager {
            package_manager: None,
            source: DetectionSource::Heuristic,
        };
    }

    let file_name = current_exe
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    let parent_name = current_exe
        .parent()
        .and_then(|value| value.file_name())
        .and_then(|value| value.to_str())
        .unwrap_or_default();

    if matches!(file_name, "aah" | "aah.exe") && parent_name == "bin" {
        return DetectedInstall::Binary {
            install_dir: current_exe.parent().map(|value| value.to_path_buf()),
            source: DetectionSource::Heuristic,
        };
    }

    DetectedInstall::Unknown
}
