use std::fs;
use std::path::Path;

use crate::codex_usage::store::CodexUsageStore;

use super::auth::{match_active_identity, read_account_identity_from_path};
pub use super::cli::CodexLoginRunner;
use super::cli::ProcessCodexLoginRunner;
use super::models::{CodexAccountListItem, StoredCodexAccount};
use super::paths::{atomic_write, CodexAccountPaths};
use super::store::{auth_path_for_home, CodexAccountStore};

pub struct CodexAccountService {
    paths: CodexAccountPaths,
    login_runner: Box<dyn CodexLoginRunner>,
}

impl CodexAccountService {
    pub fn new(paths: CodexAccountPaths, login_runner: Box<dyn CodexLoginRunner>) -> Self {
        Self {
            paths,
            login_runner,
        }
    }

    pub fn with_process_runner(paths: CodexAccountPaths) -> Self {
        Self::new(paths, Box::new(ProcessCodexLoginRunner))
    }

    pub fn start_login(&self) -> Result<StoredCodexAccount, String> {
        self.paths.ensure_dirs()?;

        let managed_home = self
            .paths
            .managed_homes_dir
            .join(uuid::Uuid::new_v4().to_string());

        let result = (|| {
            self.login_runner.run_login(&managed_home)?;

            let managed_auth_path = auth_path_for_home(&managed_home);
            let identity = read_account_identity_from_path(&managed_auth_path)?;
            let mut store = CodexAccountStore::load(&self.paths)?;
            let previous_home = store
                .find_matching_account(&identity)
                .map(|account| account.managed_home_path.clone());
            let saved = store.upsert_identity(&self.paths, identity, managed_home.clone())?;

            if let Some(previous_home) = previous_home {
                if previous_home != saved.managed_home_path {
                    let _ = fs::remove_dir_all(previous_home);
                }
            }

            Ok(saved)
        })();

        if result.is_err() {
            let _ = fs::remove_dir_all(&managed_home);
        }

        result
    }

    pub fn import_current_account_if_missing(&self) -> Result<Option<StoredCodexAccount>, String> {
        self.paths.ensure_dirs()?;

        let live_bytes = match fs::read(&self.paths.system_auth_path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "failed to read live auth.json from {}: {error}",
                    self.paths.system_auth_path.display()
                ))
            }
        };
        let identity = read_account_identity_from_path(&self.paths.system_auth_path)?;
        let mut store = CodexAccountStore::load(&self.paths)?;
        if store.find_matching_account(&identity).is_some() {
            return Ok(None);
        }

        let managed_home = self
            .paths
            .managed_homes_dir
            .join(uuid::Uuid::new_v4().to_string());
        let managed_auth_path = auth_path_for_home(&managed_home);
        let result = (|| {
            fs::create_dir_all(&managed_home)
                .map_err(|error| format!("failed to create managed Codex home: {error}"))?;
            atomic_write(&managed_auth_path, &live_bytes)?;
            store
                .upsert_identity(&self.paths, identity, managed_home.clone())
                .map(Some)
        })();

        if result.is_err() {
            let _ = fs::remove_dir_all(&managed_home);
        }

        result
    }

    pub fn list_accounts(&self) -> Result<Vec<CodexAccountListItem>, String> {
        let store = CodexAccountStore::load(&self.paths)?;
        let usage_store = CodexUsageStore::load(&self.paths)?;
        let live_identity = read_account_identity_from_path(&self.paths.system_auth_path).ok();
        let active_id = live_identity
            .as_ref()
            .and_then(|identity| match_active_identity(identity, store.accounts()))
            .map(|account| account.id.clone());

        Ok(store
            .accounts()
            .iter()
            .map(|account| {
                let usage = usage_store.get(&account.id);

                CodexAccountListItem {
                    id: account.id.clone(),
                    email: account.email.clone(),
                    plan: usage
                        .and_then(|snapshot| snapshot.plan.clone())
                        .or_else(|| account.plan.clone()),
                    account_id: account.account_id.clone(),
                    is_active: active_id.as_deref() == Some(account.id.as_str()),
                    last_authenticated_at: account.last_authenticated_at.clone(),
                    five_hour_remaining_percent: usage.and_then(|snapshot| {
                        snapshot
                            .five_hour
                            .as_ref()
                            .map(|window| window.remaining_percent)
                    }),
                    weekly_remaining_percent: usage.and_then(|snapshot| {
                        snapshot
                            .weekly
                            .as_ref()
                            .map(|window| window.remaining_percent)
                    }),
                    five_hour_refresh_at: usage.and_then(|snapshot| {
                        snapshot
                            .five_hour
                            .as_ref()
                            .map(|window| window.reset_at.clone())
                    }),
                    weekly_refresh_at: usage.and_then(|snapshot| {
                        snapshot
                            .weekly
                            .as_ref()
                            .map(|window| window.reset_at.clone())
                    }),
                    last_synced_at: usage.and_then(|snapshot| snapshot.last_synced_at.clone()),
                    last_sync_error: usage.and_then(|snapshot| snapshot.last_sync_error.clone()),
                    credits_balance: usage.and_then(|snapshot| snapshot.credits_balance),
                    needs_relogin: usage.map(|snapshot| snapshot.needs_relogin),
                }
            })
            .collect())
    }

    pub fn switch_account(&self, account_id: &str) -> Result<(), String> {
        let store = CodexAccountStore::load(&self.paths)?;
        let account = store
            .find_by_id(account_id)
            .ok_or_else(|| format!("account {account_id} not found"))?;
        let managed_auth_path = auth_path_for_home(Path::new(&account.managed_home_path));
        let bytes = fs::read(&managed_auth_path)
            .map_err(|error| format!("failed to read managed auth.json: {error}"))?;
        atomic_write(&self.paths.system_auth_path, &bytes)
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), String> {
        let mut store = CodexAccountStore::load(&self.paths)?;
        let mut usage_store = CodexUsageStore::load(&self.paths)?;
        let managed_home = store
            .find_by_id(account_id)
            .map(|account| account.managed_home_path.clone());

        store.delete(&self.paths, account_id)?;
        usage_store.delete(account_id);
        usage_store.persist(&self.paths)?;

        if let Some(managed_home) = managed_home {
            let _ = fs::remove_dir_all(managed_home);
        }

        Ok(())
    }
}
