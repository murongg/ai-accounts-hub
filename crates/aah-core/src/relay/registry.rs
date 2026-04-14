use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::app_settings::models::RelaySettings;
use crate::codex_accounts::paths::atomic_write;

use super::proxy::{
    RelayHealthResponse, RELAY_ADMIN_TOKEN_HEADER, RELAY_HEALTH_PATH, RELAY_STOP_PATH,
};
use super::state::{relay_status, RelayOwnerKind, RelayRuntimeStatus};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayRegistryPaths {
    pub relay_dir: PathBuf,
    pub runtime_path: PathBuf,
}

impl RelayRegistryPaths {
    pub fn from_managed_root(managed_root: &Path) -> Self {
        let relay_dir = managed_root.join("relay");
        let runtime_path = relay_dir.join("runtime.json");
        Self {
            relay_dir,
            runtime_path,
        }
    }

    pub fn ensure_dirs(&self) -> Result<(), String> {
        fs::create_dir_all(&self.relay_dir).map_err(|error| {
            format!(
                "failed to create relay dir {}: {error}",
                self.relay_dir.display()
            )
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayRuntimeRecord {
    pub owner_kind: RelayOwnerKind,
    pub pid: u32,
    pub port: u16,
    pub admin_token: String,
}

impl RelayRuntimeRecord {
    pub fn codex_base_url(&self) -> String {
        format!("http://127.0.0.1:{}/codex", self.port)
    }
}

pub fn load_runtime_record(
    paths: &RelayRegistryPaths,
) -> Result<Option<RelayRuntimeRecord>, String> {
    match fs::read_to_string(&paths.runtime_path) {
        Ok(text) => serde_json::from_str::<RelayRuntimeRecord>(&text)
            .map(Some)
            .map_err(|error| format!("failed to parse relay runtime record: {error}")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read relay runtime record {}: {error}",
            paths.runtime_path.display()
        )),
    }
}

pub fn save_runtime_record(
    paths: &RelayRegistryPaths,
    record: &RelayRuntimeRecord,
) -> Result<(), String> {
    paths.ensure_dirs()?;
    let bytes = serde_json::to_vec_pretty(record)
        .map_err(|error| format!("failed to encode relay runtime record: {error}"))?;
    atomic_write(&paths.runtime_path, &bytes)
}

pub fn remove_runtime_record(paths: &RelayRegistryPaths) -> Result<(), String> {
    match fs::remove_file(&paths.runtime_path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "failed to remove relay runtime record {}: {error}",
            paths.runtime_path.display()
        )),
    }
}

pub fn shared_runtime_status(
    settings: &RelaySettings,
    paths: &RelayRegistryPaths,
    last_error: Option<String>,
) -> RelayRuntimeStatus {
    match load_runtime_record(paths) {
        Ok(Some(record)) => match probe_health(record.port) {
            Some(health) => relay_status(true, health.port, last_error),
            None => {
                let _ = remove_runtime_record(paths);
                relay_status(false, settings.port, last_error)
            }
        },
        Ok(None) => relay_status(false, settings.port, last_error),
        Err(error) => relay_status(false, settings.port, Some(error)),
    }
}

pub fn stop_shared_runtime(paths: &RelayRegistryPaths) -> Result<bool, String> {
    let Some(record) = load_runtime_record(paths)? else {
        return Ok(false);
    };
    if probe_health(record.port).is_none() {
        remove_runtime_record(paths)?;
        return Ok(false);
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|error| format!("failed to build relay admin client: {error}"))?;
    client
        .post(format!(
            "http://127.0.0.1:{}{}",
            record.port, RELAY_STOP_PATH
        ))
        .header(RELAY_ADMIN_TOKEN_HEADER, record.admin_token)
        .send()
        .map_err(|error| format!("failed to stop relay runtime: {error}"))?
        .error_for_status()
        .map_err(|error| format!("relay stop request failed: {error}"))?;
    Ok(true)
}

pub fn probe_health(port: u16) -> Option<RelayHealthResponse> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(500))
        .build()
        .ok()?;
    let response = client
        .get(format!("http://127.0.0.1:{port}{RELAY_HEALTH_PATH}"))
        .send()
        .ok()?
        .error_for_status()
        .ok()?;
    response.json::<RelayHealthResponse>().ok()
}
