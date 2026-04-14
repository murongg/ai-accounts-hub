use crate::gemini_accounts::paths::{atomic_write, GeminiAccountPaths};
use crate::time_utils::timestamp_string;

use super::models::{FetchedGeminiUsage, GeminiUsageSnapshot, GeminiUsageSnapshotIndex};

pub fn load_usage_snapshots(
    paths: &GeminiAccountPaths,
) -> Result<Vec<GeminiUsageSnapshot>, String> {
    paths.ensure_dirs()?;
    match std::fs::read_to_string(&paths.usage_snapshot_path) {
        Ok(text) => serde_json::from_str::<GeminiUsageSnapshotIndex>(&text)
            .map(|index| index.snapshots)
            .map_err(|error| format!("failed to parse Gemini usage snapshots: {error}")),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(error) => Err(format!("failed to read Gemini usage snapshots: {error}")),
    }
}

pub fn save_usage_snapshots(
    paths: &GeminiAccountPaths,
    snapshots: &[GeminiUsageSnapshot],
) -> Result<(), String> {
    let index = GeminiUsageSnapshotIndex {
        version: 1,
        snapshots: snapshots.to_vec(),
    };
    let bytes = serde_json::to_vec_pretty(&index)
        .map_err(|error| format!("failed to serialize Gemini usage snapshots: {error}"))?;
    atomic_write(&paths.usage_snapshot_path, &bytes)
}

pub struct GeminiUsageStore {
    snapshots: Vec<GeminiUsageSnapshot>,
}

impl GeminiUsageStore {
    pub fn load(paths: &GeminiAccountPaths) -> Result<Self, String> {
        Ok(Self {
            snapshots: load_usage_snapshots(paths)?,
        })
    }

    pub fn get(&self, managed_account_id: &str) -> Option<&GeminiUsageSnapshot> {
        self.snapshots
            .iter()
            .find(|snapshot| snapshot.managed_account_id == managed_account_id)
    }

    pub fn upsert_success(&mut self, managed_account_id: &str, fetched: FetchedGeminiUsage) {
        let snapshot = GeminiUsageSnapshot {
            managed_account_id: managed_account_id.to_string(),
            plan: fetched.plan,
            pro: fetched.pro,
            flash: fetched.flash,
            flash_lite: fetched.flash_lite,
            last_synced_at: Some(timestamp_string()),
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
        let mut snapshot = self
            .get(managed_account_id)
            .cloned()
            .unwrap_or(GeminiUsageSnapshot {
                managed_account_id: managed_account_id.to_string(),
                plan: fallback_plan.clone(),
                pro: None,
                flash: None,
                flash_lite: None,
                last_synced_at: None,
                last_sync_error: None,
                needs_relogin: false,
            });
        if snapshot.plan.is_none() {
            snapshot.plan = fallback_plan;
        }
        snapshot.last_synced_at = Some(timestamp_string());
        snapshot.last_sync_error = Some(error);
        snapshot.needs_relogin = needs_relogin;
        self.replace_snapshot(snapshot);
    }

    pub fn clear_auth_error(&mut self, managed_account_id: &str) {
        let Some(snapshot) = self
            .snapshots
            .iter_mut()
            .find(|snapshot| snapshot.managed_account_id == managed_account_id)
        else {
            return;
        };

        snapshot.needs_relogin = false;
        if snapshot
            .last_sync_error
            .as_deref()
            .is_some_and(|message| message.contains("Run `gemini` again"))
        {
            snapshot.last_sync_error = None;
        }
    }

    pub fn retain_only(&mut self, managed_account_ids: &[String]) {
        self.snapshots.retain(|snapshot| {
            managed_account_ids
                .iter()
                .any(|id| id == &snapshot.managed_account_id)
        });
    }

    pub fn persist(&self, paths: &GeminiAccountPaths) -> Result<(), String> {
        save_usage_snapshots(paths, &self.snapshots)
    }

    fn replace_snapshot(&mut self, snapshot: GeminiUsageSnapshot) {
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
