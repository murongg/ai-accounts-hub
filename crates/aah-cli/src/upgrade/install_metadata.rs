use std::io::ErrorKind;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InstallMethod {
    PackageManager,
    Binary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstallMetadata {
    pub schema_version: u8,
    pub install_method: InstallMethod,
    pub package_manager: Option<String>,
    pub package_name: Option<String>,
    pub binary_path: PathBuf,
    pub install_dir: Option<PathBuf>,
    pub repository: Option<String>,
    pub installed_at: Option<String>,
}

pub fn install_metadata_path_from(config_dir: Option<PathBuf>) -> Result<PathBuf, String> {
    let config_dir = config_dir
        .or_else(dirs::config_dir)
        .ok_or_else(|| "failed to resolve config directory".to_string())?;
    Ok(config_dir.join("aah/cli-install.json"))
}

pub fn load_install_metadata(
    config_dir: Option<PathBuf>,
) -> Result<Option<InstallMetadata>, String> {
    let path = install_metadata_path_from(config_dir)?;
    match std::fs::read_to_string(&path) {
        Ok(text) => serde_json::from_str(&text).map(Some).map_err(|error| {
            format!(
                "failed to parse install metadata {}: {error}",
                path.display()
            )
        }),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read install metadata {}: {error}",
            path.display()
        )),
    }
}
