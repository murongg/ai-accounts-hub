use crate::codex_accounts::paths::{atomic_write, CodexAccountPaths};
use crate::time_utils::timestamp_string;

use super::models::{
    CodexRefreshSettings, CodexUsageSnapshot, CodexUsageSnapshotIndex, FetchedCodexUsage,
};

pub fn load_refresh_settings(paths: &CodexAccountPaths) -> Result<CodexRefreshSettings, String> {
    paths.ensure_dirs()?;
    match std::fs::read_to_string(&paths.refresh_settings_path) {
        Ok(text) => serde_json::from_str::<CodexRefreshSettings>(&text)
            .map(|settings| settings.sanitized())
            .map_err(|error| format!("failed to parse refresh settings: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(CodexRefreshSettings::default())
        }
        Err(error) => Err(format!("failed to read refresh settings: {error}")),
    }
}

pub fn save_refresh_settings(
    paths: &CodexAccountPaths,
    settings: CodexRefreshSettings,
) -> Result<CodexRefreshSettings, String> {
    let sanitized = settings.sanitized();
    let bytes = serde_json::to_vec_pretty(&sanitized)
        .map_err(|error| format!("failed to serialize refresh settings: {error}"))?;
    atomic_write(&paths.refresh_settings_path, &bytes)?;
    Ok(sanitized)
}

pub fn load_usage_snapshots(paths: &CodexAccountPaths) -> Result<Vec<CodexUsageSnapshot>, String> {
    paths.ensure_dirs()?;
    match std::fs::read_to_string(&paths.usage_snapshot_path) {
        Ok(text) => serde_json::from_str::<CodexUsageSnapshotIndex>(&text)
            .map(|index| index.snapshots)
            .map_err(|error| format!("failed to parse usage snapshots: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("failed to read usage snapshots: {error}")),
    }
}

pub fn save_usage_snapshots(
    paths: &CodexAccountPaths,
    snapshots: &[CodexUsageSnapshot],
) -> Result<(), String> {
    let index = CodexUsageSnapshotIndex {
        version: 1,
        snapshots: snapshots.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&index)
        .map_err(|error| format!("failed to serialize usage snapshots: {error}"))?;
    atomic_write(&paths.usage_snapshot_path, &bytes)
}

pub struct CodexUsageStore {
    snapshots: Vec<CodexUsageSnapshot>,
}

impl CodexUsageStore {
    pub fn load(paths: &CodexAccountPaths) -> Result<Self, String> {
        Ok(Self {
            snapshots: load_usage_snapshots(paths)?,
        })
    }

    pub fn get(&self, managed_account_id: &str) -> Option<&CodexUsageSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.managed_account_id == managed_account_id)
    }

    pub fn upsert_success(&mut self, managed_account_id: &str, fetched: FetchedCodexUsage) {
        let now = timestamp_string();
        let snapshot = CodexUsageSnapshot {
            managed_account_id: managed_account_id.to_string(),
            plan: fetched.plan,
            five_hour: fetched.five_hour,
            weekly: fetched.weekly,
            credits_balance: fetched.credits_balance,
            last_synced_at: Some(now),
            last_sync_error: None,
            needs_relogin: false,
        };
        self.replace_snapshot(snapshot);
    }

    pub fn upsert_error(
        &mut self,
        managed_account_id: &str,
        fallback_plan: Option<String>,
        error: String,
        needs_relogin: bool,
    ) {
        let now = timestamp_string();
        let mut snapshot = self
            .get(managed_account_id)
            .cloned()
            .unwrap_or(CodexUsageSnapshot {
                managed_account_id: managed_account_id.to_string(),
                plan: fallback_plan.clone(),
                five_hour: None,
                weekly: None,
                credits_balance: None,
                last_synced_at: None,
                last_sync_error: None,
                needs_relogin: false,
            });
        if snapshot.plan.is_none() {
            snapshot.plan = fallback_plan;
        }
        snapshot.last_synced_at = Some(now);
        snapshot.last_sync_error = Some(error);
        snapshot.needs_relogin = needs_relogin;
        self.replace_snapshot(snapshot);
    }

    pub fn delete(&mut self, managed_account_id: &str) {
        self.snapshots
            .retain(|snapshot| snapshot.managed_account_id != managed_account_id);
    }

    pub fn retain_only(&mut self, managed_account_ids: &[String]) {
        self.snapshots.retain(|snapshot| {
            managed_account_ids
                .iter()
                .any(|id| id == &snapshot.managed_account_id)
        });
    }

    pub fn persist(&self, paths: &CodexAccountPaths) -> Result<(), String> {
        save_usage_snapshots(paths, &self.snapshots)
    }

    fn replace_snapshot(&mut self, snapshot: CodexUsageSnapshot) {
        if let Some(existing) = self
            .snapshots
            .iter_mut()
            .find(|existing| existing.managed_account_id == snapshot.managed_account_id)
        {
            *existing = snapshot;
        } else {
            self.snapshots.push(snapshot);
        }
    }
}
