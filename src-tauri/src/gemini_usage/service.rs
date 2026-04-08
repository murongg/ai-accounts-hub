use std::path::Path;

use crate::gemini_accounts::paths::GeminiAccountPaths;
use crate::gemini_accounts::store::GeminiAccountStore;

use super::oauth::{GeminiUsageFetcher, ProcessGeminiUsageFetcher};
use super::store::GeminiUsageStore;

pub struct GeminiUsageService {
    paths: GeminiAccountPaths,
    fetcher: Box<dyn GeminiUsageFetcher>,
}

impl GeminiUsageService {
    pub fn new(paths: GeminiAccountPaths, fetcher: Box<dyn GeminiUsageFetcher>) -> Self {
        Self { paths, fetcher }
    }

    pub fn with_process_fetcher(paths: GeminiAccountPaths) -> Self {
        Self::new(paths, Box::new(ProcessGeminiUsageFetcher))
    }

    pub fn refresh_all(&self) -> Result<(), String> {
        let account_store = GeminiAccountStore::load(&self.paths)?;
        let mut usage_store = GeminiUsageStore::load(&self.paths)?;
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
                    None,
                    error.to_string(),
                    error.needs_relogin(),
                ),
            }
        }

        usage_store.retain_only(&active_account_ids);
        usage_store.persist(&self.paths)
    }
}
