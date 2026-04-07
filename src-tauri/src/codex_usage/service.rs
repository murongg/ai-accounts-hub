use std::path::Path;

use crate::codex_accounts::paths::CodexAccountPaths;
use crate::codex_accounts::store::CodexAccountStore;

use super::oauth::{CodexUsageFetcher, ProcessCodexUsageFetcher};
use super::store::CodexUsageStore;

pub struct CodexUsageService {
    paths: CodexAccountPaths,
    fetcher: Box<dyn CodexUsageFetcher>,
}

impl CodexUsageService {
    pub fn new(paths: CodexAccountPaths, fetcher: Box<dyn CodexUsageFetcher>) -> Self {
        Self { paths, fetcher }
    }

    pub fn with_process_fetcher(paths: CodexAccountPaths) -> Self {
        Self::new(paths, Box::new(ProcessCodexUsageFetcher))
    }

    pub fn refresh_all(&self) -> Result<(), String> {
        let account_store = CodexAccountStore::load(&self.paths)?;
        let mut usage_store = CodexUsageStore::load(&self.paths)?;
        let active_account_ids = account_store
            .accounts()
            .iter()
            .map(|account| account.id.clone())
            .collect::<Vec<_>>();

        for account in account_store.accounts() {
            let managed_home = Path::new(&account.managed_home_path);
            match self.fetcher.fetch_usage(managed_home) {
                Ok(snapshot) => usage_store.upsert_success(&account.id, snapshot),
                Err(error) => usage_store.upsert_error(
                    &account.id,
                    account.plan.clone(),
                    error.to_string(),
                    error.needs_relogin(),
                ),
            }
        }

        usage_store.retain_only(&active_account_ids);
        usage_store.persist(&self.paths)
    }
}
