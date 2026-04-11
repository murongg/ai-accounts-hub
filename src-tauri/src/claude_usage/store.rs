use crate::claude_accounts::paths::{atomic_write, ClaudeAccountPaths};
use crate::time_utils::timestamp_string;

use super::models::{ClaudeUsageSnapshot, ClaudeUsageSnapshotIndex, FetchedClaudeUsage};

pub fn load_usage_snapshots(
    paths: &ClaudeAccountPaths,
) -> Result<Vec<ClaudeUsageSnapshot>, String> {
    paths.ensure_dirs()?;
    match std::fs::read_to_string(&paths.usage_snapshot_path) {
        Ok(text) => serde_json::from_str::<ClaudeUsageSnapshotIndex>(&text)
            .map(|index| index.snapshots)
            .map_err(|error| format!("failed to parse Claude usage snapshots: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("failed to read Claude usage snapshots: {error}")),
    }
}

pub fn save_usage_snapshots(
    paths: &ClaudeAccountPaths,
    snapshots: &[ClaudeUsageSnapshot],
) -> Result<(), String> {
    let index = ClaudeUsageSnapshotIndex {
        version: 1,
        snapshots: snapshots.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&index)
        .map_err(|error| format!("failed to serialize Claude usage snapshots: {error}"))?;
    atomic_write(&paths.usage_snapshot_path, &bytes)
}

pub struct ClaudeUsageStore {
    snapshots: Vec<ClaudeUsageSnapshot>,
}

impl ClaudeUsageStore {
    pub fn load(paths: &ClaudeAccountPaths) -> Result<Self, String> {
        Ok(Self {
            snapshots: load_usage_snapshots(paths)?,
        })
    }

    pub fn get(&self, managed_account_id: &str) -> Option<&ClaudeUsageSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.managed_account_id == managed_account_id)
    }

    pub fn upsert_success(&mut self, managed_account_id: &str, fetched: FetchedClaudeUsage) {
        self.replace_snapshot(ClaudeUsageSnapshot {
            managed_account_id: managed_account_id.to_string(),
            session: fetched.session,
            weekly: fetched.weekly,
            model_weekly_label: fetched.model_weekly_label,
            model_weekly: fetched.model_weekly,
            last_synced_at: Some(timestamp_string()),
            last_sync_error: None,
            needs_relogin: false,
        });
    }

    pub fn upsert_error(&mut self, managed_account_id: &str, error: String, needs_relogin: bool) {
        let mut snapshot = self
            .get(managed_account_id)
            .cloned()
            .unwrap_or(ClaudeUsageSnapshot {
                managed_account_id: managed_account_id.to_string(),
                session: None,
                weekly: None,
                model_weekly_label: None,
                model_weekly: None,
                last_synced_at: None,
                last_sync_error: None,
                needs_relogin: false,
            });
        snapshot.last_synced_at = Some(timestamp_string());
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

    pub fn persist(&self, paths: &ClaudeAccountPaths) -> Result<(), String> {
        save_usage_snapshots(paths, &self.snapshots)
    }

    fn replace_snapshot(&mut self, snapshot: ClaudeUsageSnapshot) {
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
