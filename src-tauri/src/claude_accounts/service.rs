use serde_json::Value;

use super::auth::{extract_account_identity, match_active_identity};
pub use super::cli::ClaudeLoginRunner;
use super::cli::ProcessClaudeLoginRunner;
use super::keychain::{
    ClaudeCredentialBundle, ClaudeCredentialBundleStore, ManagedClaudeKeychainStore,
};
use super::live_credentials::{ClaudeLiveCredentialStore, FileSystemClaudeLiveCredentialStore};
use super::models::{ClaudeAccountListItem, StoredClaudeAccount};
use super::paths::ClaudeAccountPaths;
use super::store::ClaudeAccountStore;
use crate::claude_usage::store::ClaudeUsageStore;

pub struct ClaudeAccountService<K: ClaudeCredentialBundleStore, L: ClaudeLiveCredentialStore> {
    paths: ClaudeAccountPaths,
    login_runner: Box<dyn ClaudeLoginRunner>,
    bundle_store: K,
    live_store: L,
}

impl<K: ClaudeCredentialBundleStore, L: ClaudeLiveCredentialStore> ClaudeAccountService<K, L> {
    pub fn new(
        paths: ClaudeAccountPaths,
        login_runner: Box<dyn ClaudeLoginRunner>,
        bundle_store: K,
        live_store: L,
    ) -> Self {
        Self {
            paths,
            login_runner,
            bundle_store,
            live_store,
        }
    }

    pub fn start_login(&mut self) -> Result<StoredClaudeAccount, String> {
        self.paths.ensure_dirs()?;
        self.login_runner.run_login(&self.paths.login_claude_dir)?;

        let live = self.live_store.capture()?;
        let identity = identity_from_snapshot(&live)?;
        let mut store = ClaudeAccountStore::load(&self.paths)?;
        let bundle_key = store
            .find_matching_account(&identity)
            .map(|account| account.credential_bundle_key.clone())
            .unwrap_or_else(|| format!("claude-bundle-{}", uuid::Uuid::new_v4()));
        let bundle = ClaudeCredentialBundle::from_live_snapshot(
            &identity.email,
            identity.account_hint.as_deref(),
            &live,
        );

        self.bundle_store.save(&bundle_key, &bundle)?;
        store.upsert_identity(&self.paths, identity, bundle_key)
    }

    pub fn list_accounts(&self) -> Result<Vec<ClaudeAccountListItem>, String> {
        let store = ClaudeAccountStore::load(&self.paths)?;
        let usage_store = ClaudeUsageStore::load(&self.paths)?;
        let live_identity = self
            .live_store
            .capture()
            .ok()
            .and_then(|snapshot| identity_from_snapshot(&snapshot).ok());
        let active_id = live_identity
            .as_ref()
            .and_then(|identity| match_active_identity(identity, store.accounts()))
            .map(|account| account.id.clone());

        Ok(store
            .accounts()
            .iter()
            .map(|account| {
                let usage = usage_store.get(&account.id);

                ClaudeAccountListItem {
                    id: account.id.clone(),
                    email: account.email.clone(),
                    display_name: account.display_name.clone(),
                    plan: account.plan.clone(),
                    account_hint: account.account_hint.clone(),
                    is_active: active_id.as_deref() == Some(account.id.as_str()),
                    last_authenticated_at: account.last_authenticated_at.clone(),
                    session_remaining_percent: usage
                        .and_then(|snapshot| snapshot.session.as_ref())
                        .map(|window| window.remaining_percent),
                    session_refresh_at: usage
                        .and_then(|snapshot| snapshot.session.as_ref())
                        .map(|window| window.reset_at.clone()),
                    weekly_remaining_percent: usage
                        .and_then(|snapshot| snapshot.weekly.as_ref())
                        .map(|window| window.remaining_percent),
                    weekly_refresh_at: usage
                        .and_then(|snapshot| snapshot.weekly.as_ref())
                        .map(|window| window.reset_at.clone()),
                    model_weekly_label: usage.and_then(|snapshot| snapshot.model_weekly_label.clone()),
                    model_weekly_remaining_percent: usage
                        .and_then(|snapshot| snapshot.model_weekly.as_ref())
                        .map(|window| window.remaining_percent),
                    model_weekly_refresh_at: usage
                        .and_then(|snapshot| snapshot.model_weekly.as_ref())
                        .map(|window| window.reset_at.clone()),
                    last_synced_at: usage.and_then(|snapshot| snapshot.last_synced_at.clone()),
                    last_sync_error: usage.and_then(|snapshot| snapshot.last_sync_error.clone()),
                    needs_relogin: usage.map(|snapshot| snapshot.needs_relogin),
                }
            })
            .collect())
    }

    pub fn switch_account(&mut self, account_id: &str) -> Result<(), String> {
        let store = ClaudeAccountStore::load(&self.paths)?;
        let account = store
            .find_by_id(account_id)
            .cloned()
            .ok_or_else(|| format!("Claude account {account_id} not found"))?;
        let bundle = self
            .bundle_store
            .load(&account.credential_bundle_key)?
            .ok_or_else(|| format!("Claude bundle {} not found", account.credential_bundle_key))?;

        self.live_store.restore(&bundle)?;

        let verified = self.live_store.capture()?;
        let verified_identity = identity_from_snapshot(&verified)?;
        if match_active_identity(&verified_identity, std::slice::from_ref(&account)).is_none() {
            return Err(
                "restored Claude credentials do not match the selected account".to_string(),
            );
        }

        Ok(())
    }

    pub fn delete_account(&mut self, account_id: &str) -> Result<(), String> {
        let mut store = ClaudeAccountStore::load(&self.paths)?;
        if let Some(account) = store.find_by_id(account_id).cloned() {
            self.bundle_store.delete(&account.credential_bundle_key)?;
        }
        store.delete(&self.paths, account_id)
    }
}

impl ClaudeAccountService<ManagedClaudeKeychainStore, FileSystemClaudeLiveCredentialStore> {
    pub fn with_process_runner(paths: ClaudeAccountPaths) -> Self {
        let live_paths = paths.clone();
        Self::new(
            paths,
            Box::new(ProcessClaudeLoginRunner),
            ManagedClaudeKeychainStore::new(),
            FileSystemClaudeLiveCredentialStore::new(live_paths),
        )
    }
}

fn identity_from_snapshot(
    snapshot: &super::live_credentials::ClaudeLiveCredentialSnapshot,
) -> Result<super::models::ClaudeAccountIdentity, String> {
    let credentials: Value = serde_json::from_slice(&snapshot.credentials_json)
        .map_err(|error| format!("failed to parse Claude credentials json: {error}"))?;
    let oauth_account = snapshot
        .oauth_account_json
        .as_deref()
        .map(|bytes| {
            serde_json::from_slice::<Value>(bytes)
                .map_err(|error| format!("failed to parse Claude oauth account json: {error}"))
        })
        .transpose()?;

    extract_account_identity(&credentials, oauth_account.as_ref())
}
