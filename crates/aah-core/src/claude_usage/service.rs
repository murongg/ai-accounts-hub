use crate::claude_accounts::keychain::{ClaudeCredentialBundleStore, ManagedClaudeKeychainStore};
use crate::claude_accounts::live_credentials::ClaudeLiveCredentialSnapshot;
use crate::claude_accounts::paths::ClaudeAccountPaths;
use crate::claude_accounts::store::ClaudeAccountStore;

use super::cli_probe::{ClaudeCliUsageProbe, ProcessClaudeCliUsageProbe};
use super::oauth::{
    ClaudeUsageFetchError, ClaudeUsageFetcher, OAuthClaudeUsageFetcher,
    ProcessClaudeOAuthHttpClient,
};
use super::store::ClaudeUsageStore;

pub struct ClaudeUsageService<K: ClaudeCredentialBundleStore> {
    paths: ClaudeAccountPaths,
    bundle_store: K,
    oauth_fetcher: Box<dyn ClaudeUsageFetcher>,
    cli_probe: Box<dyn ClaudeCliUsageProbe>,
}

impl<K: ClaudeCredentialBundleStore> ClaudeUsageService<K> {
    pub fn new(
        paths: ClaudeAccountPaths,
        bundle_store: K,
        oauth_fetcher: Box<dyn ClaudeUsageFetcher>,
        cli_probe: Box<dyn ClaudeCliUsageProbe>,
    ) -> Self {
        Self {
            paths,
            bundle_store,
            oauth_fetcher,
            cli_probe,
        }
    }

    pub fn refresh_all(&self) -> Result<(), String> {
        let account_store = ClaudeAccountStore::load(&self.paths)?;
        let mut usage_store = ClaudeUsageStore::load(&self.paths)?;
        let active_account_ids = account_store
            .accounts()
            .iter()
            .map(|account| account.id.clone())
            .collect::<Vec<_>>();

        for account in account_store.accounts() {
            let bundle = match self.bundle_store.load(&account.credential_bundle_key)? {
                Some(bundle) => bundle,
                None => {
                    usage_store.upsert_error(
                        &account.id,
                        format!(
                            "Claude credential bundle {} is missing.",
                            account.credential_bundle_key
                        ),
                        true,
                    );
                    continue;
                }
            };
            let snapshot = ClaudeLiveCredentialSnapshot {
                credentials_json: bundle.credentials_json,
                oauth_account_json: bundle.oauth_account_json,
            };

            match self.oauth_fetcher.fetch_usage(&snapshot) {
                Ok(fetched) => usage_store.upsert_success(&account.id, fetched),
                Err(oauth_error) => match self.cli_probe.probe_usage(&snapshot) {
                    Ok(fetched) => usage_store.upsert_success(&account.id, fetched),
                    Err(cli_error) => usage_store.upsert_error(
                        &account.id,
                        cli_error.to_string(),
                        oauth_error.needs_relogin() || cli_error.needs_relogin(),
                    ),
                },
            }
        }

        usage_store.retain_only(&active_account_ids);
        usage_store.persist(&self.paths)
    }
}

impl ClaudeUsageService<ManagedClaudeKeychainStore> {
    pub fn with_process_fetchers(paths: ClaudeAccountPaths) -> Self {
        Self::new(
            paths,
            ManagedClaudeKeychainStore::new(),
            Box::new(OAuthClaudeUsageFetcher::new(
                ProcessClaudeOAuthHttpClient::default(),
            )),
            Box::new(ProcessClaudeCliUsageProbe),
        )
    }
}

pub fn combine_usage_errors(
    oauth_error: ClaudeUsageFetchError,
    cli_error: &super::cli_probe::ClaudeCliUsageProbeError,
) -> (String, bool) {
    let message = format!("{}; {}", oauth_error, cli_error);
    let needs_relogin = oauth_error.needs_relogin() || cli_error.needs_relogin();
    (message, needs_relogin)
}
