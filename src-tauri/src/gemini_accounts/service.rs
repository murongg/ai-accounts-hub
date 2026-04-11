use std::fs;
use std::path::Path;

use super::auth::{match_active_identity, read_account_identity_from_dir};
pub use super::cli::GeminiLoginRunner;
use super::cli::ProcessGeminiLoginRunner;
use super::models::{GeminiAccountListItem, StoredGeminiAccount};
use super::paths::{
    atomic_write, gemini_dir_for_home, google_accounts_path_for_home, oauth_creds_path_for_home,
    settings_path_for_home, GeminiAccountPaths,
};
use super::store::GeminiAccountStore;
use crate::gemini_usage::store::GeminiUsageStore;

pub struct GeminiAccountService {
    paths: GeminiAccountPaths,
    login_runner: Box<dyn GeminiLoginRunner>,
}

impl GeminiAccountService {
    pub fn new(paths: GeminiAccountPaths, login_runner: Box<dyn GeminiLoginRunner>) -> Self {
        Self {
            paths,
            login_runner,
        }
    }

    pub fn with_process_runner(paths: GeminiAccountPaths) -> Self {
        Self::new(paths, Box::new(ProcessGeminiLoginRunner))
    }

    pub fn start_login(&self) -> Result<StoredGeminiAccount, String> {
        self.paths.ensure_dirs()?;

        let managed_home = self
            .paths
            .managed_homes_dir
            .join(uuid::Uuid::new_v4().to_string());

        let result = (|| {
            self.login_runner.run_login(&managed_home)?;

            let identity = read_account_identity_from_dir(&gemini_dir_for_home(&managed_home))?;
            let mut store = GeminiAccountStore::load(&self.paths)?;
            let previous_home = store
                .find_matching_account(&identity)
                .map(|account| account.managed_home_path.clone());
            let saved = store.upsert_identity(&self.paths, identity, managed_home.clone())?;
            let mut usage_store = GeminiUsageStore::load(&self.paths)?;
            usage_store.clear_auth_error(&saved.id);
            usage_store.persist(&self.paths)?;

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

    pub fn list_accounts(&self) -> Result<Vec<GeminiAccountListItem>, String> {
        let store = GeminiAccountStore::load(&self.paths)?;
        let usage_store = GeminiUsageStore::load(&self.paths)?;
        let live_identity = read_account_identity_from_dir(&self.paths.system_gemini_dir).ok();
        let active_id = live_identity
            .as_ref()
            .and_then(|identity| match_active_identity(identity, store.accounts()))
            .map(|account| account.id.clone());

        Ok(store
            .accounts()
            .iter()
            .map(|account| {
                let usage = usage_store.get(&account.id);

                GeminiAccountListItem {
                    id: account.id.clone(),
                    email: account.email.clone(),
                    subject: account.subject.clone(),
                    auth_type: account.auth_type.clone(),
                    plan: usage.and_then(|snapshot| snapshot.plan.clone()),
                    is_active: active_id.as_deref() == Some(account.id.as_str()),
                    last_authenticated_at: account.last_authenticated_at.clone(),
                    pro_remaining_percent: usage.and_then(|snapshot| {
                        snapshot.pro.as_ref().map(|window| window.remaining_percent)
                    }),
                    flash_remaining_percent: usage.and_then(|snapshot| {
                        snapshot
                            .flash
                            .as_ref()
                            .map(|window| window.remaining_percent)
                    }),
                    flash_lite_remaining_percent: usage.and_then(|snapshot| {
                        snapshot
                            .flash_lite
                            .as_ref()
                            .map(|window| window.remaining_percent)
                    }),
                    pro_refresh_at: usage.and_then(|snapshot| {
                        snapshot.pro.as_ref().map(|window| window.reset_at.clone())
                    }),
                    flash_refresh_at: usage.and_then(|snapshot| {
                        snapshot
                            .flash
                            .as_ref()
                            .map(|window| window.reset_at.clone())
                    }),
                    flash_lite_refresh_at: usage.and_then(|snapshot| {
                        snapshot
                            .flash_lite
                            .as_ref()
                            .map(|window| window.reset_at.clone())
                    }),
                    last_synced_at: usage.and_then(|snapshot| snapshot.last_synced_at.clone()),
                    last_sync_error: usage.and_then(|snapshot| snapshot.last_sync_error.clone()),
                    needs_relogin: usage.map(|snapshot| snapshot.needs_relogin),
                }
            })
            .collect())
    }

    pub fn switch_account(&self, account_id: &str) -> Result<(), String> {
        let store = GeminiAccountStore::load(&self.paths)?;
        let account = store
            .find_by_id(account_id)
            .ok_or_else(|| format!("Gemini account {account_id} not found"))?;
        let managed_home = Path::new(&account.managed_home_path);

        self.copy_auth_file(
            &oauth_creds_path_for_home(managed_home),
            &self.paths.system_gemini_dir.join("oauth_creds.json"),
            true,
        )?;
        self.copy_auth_file(
            &google_accounts_path_for_home(managed_home),
            &self.paths.system_gemini_dir.join("google_accounts.json"),
            false,
        )?;
        self.copy_auth_file(
            &settings_path_for_home(managed_home),
            &self.paths.system_gemini_dir.join("settings.json"),
            false,
        )?;

        Ok(())
    }

    pub fn delete_account(&self, account_id: &str) -> Result<(), String> {
        let mut store = GeminiAccountStore::load(&self.paths)?;
        let managed_home = store
            .find_by_id(account_id)
            .map(|account| account.managed_home_path.clone());

        store.delete(&self.paths, account_id)?;

        if let Some(managed_home) = managed_home {
            let _ = fs::remove_dir_all(managed_home);
        }

        Ok(())
    }

    fn copy_auth_file(&self, source: &Path, target: &Path, required: bool) -> Result<(), String> {
        match fs::read(source) {
            Ok(bytes) => atomic_write(target, &bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound && !required => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Err(format!(
                "missing managed Gemini file {}: {error}",
                source.display()
            )),
            Err(error) => Err(format!("failed to read {}: {error}", source.display())),
        }
    }
}
