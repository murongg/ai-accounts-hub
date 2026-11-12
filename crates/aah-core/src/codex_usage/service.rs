use std::path::Path;

use crate::codex_accounts::auth::read_account_identity_from_path;
use crate::codex_accounts::paths::CodexAccountPaths;
use crate::codex_accounts::store::{auth_path_for_home, CodexAccountStore};

use super::models::FetchedCodexUsage;
use super::oauth::{resolve_usage_url, CodexUsageFetcher, ProcessCodexUsageFetcher};
use super::store::CodexUsageStore;
use super::warmup;

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

    pub fn refresh_account(&self, managed_account_id: &str) -> Result<(), String> {
        self.refresh_account_checked(managed_account_id).map(|_| ())
    }

    /// Returns whether the latest quota query succeeded after a completed warm-up.
    pub fn warm_account(&self, managed_account_id: &str) -> Result<bool, String> {
        self.warm_account_with(managed_account_id, warmup::run)
    }

    fn warm_account_with(
        &self,
        managed_account_id: &str,
        run: impl FnOnce(&Path) -> Result<(), String>,
    ) -> Result<bool, String> {
        let account_store = CodexAccountStore::load(&self.paths)?;
        let account = account_store
            .find_by_id(managed_account_id)
            .ok_or_else(|| "The selected Codex account no longer exists.".to_string())?;
        let managed_root = self
            .paths
            .managed_homes_dir
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let managed_home = Path::new(&account.managed_home_path)
            .canonicalize()
            .map_err(|error| error.to_string())?;
        let auth_path = auth_path_for_home(&managed_home);
        if managed_home == managed_root
            || !managed_home.starts_with(&managed_root)
            || !std::fs::symlink_metadata(&auth_path).is_ok_and(|meta| meta.is_file())
        {
            return Err("The selected account is missing an isolated Codex login.".into());
        }
        let identity = read_account_identity_from_path(&auth_path)?;
        if !identity.email.eq_ignore_ascii_case(&account.email)
            || account
                .account_id
                .as_ref()
                .is_some_and(|id| identity.account_id.as_ref() != Some(id))
        {
            return Err("The managed Codex login does not match the selected account.".into());
        }
        if resolve_usage_url(&managed_home) != "https://chatgpt.com/backend-api/wham/usage" {
            return Err("Warm-up requires the standard ChatGPT usage endpoint.".into());
        }

        let quota = self
            .fetcher
            .fetch_usage(&managed_home)
            .map_err(|error| error.to_string())?;
        check_warmup_quota(&quota)?;
        run(&managed_home)?;

        // The prompt already completed. A failed recheck must not suggest sending it again.
        Ok(self
            .refresh_account_checked(managed_account_id)
            .unwrap_or(false))
    }

    fn refresh_account_checked(&self, managed_account_id: &str) -> Result<bool, String> {
        let account_store = CodexAccountStore::load(&self.paths)?;
        let mut usage_store = CodexUsageStore::load(&self.paths)?;
        let account = account_store
            .accounts()
            .iter()
            .find(|account| account.id == managed_account_id)
            .ok_or_else(|| format!("codex account not found: {managed_account_id}"))?;

        let managed_home = Path::new(&account.managed_home_path);
        let fetched = self.fetcher.fetch_usage(managed_home);
        let succeeded = fetched.is_ok();
        match fetched {
            Ok(snapshot) => usage_store.upsert_success(&account.id, snapshot),
            Err(error) => usage_store.upsert_error(
                &account.id,
                account.plan.clone(),
                error.to_string(),
                error.needs_relogin(),
            ),
        }

        usage_store.persist(&self.paths)?;
        Ok(succeeded)
    }
}

fn check_warmup_quota(quota: &FetchedCodexUsage) -> Result<(), String> {
    let windows = [quota.five_hour.as_ref(), quota.weekly.as_ref()];
    if windows
        .iter()
        .flatten()
        .any(|window| window.remaining_percent == 0)
    {
        return Err("An included quota window is exhausted. Try after it resets.".into());
    }
    if !windows
        .iter()
        .flatten()
        .any(|window| window.used_percent == 0)
    {
        return Err("No unused quota window was reported for this account.".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::path::PathBuf;
    use std::sync::Mutex;

    use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
    use serde_json::json;

    use crate::codex_usage::models::RateWindowSnapshot;
    use crate::codex_usage::oauth::CodexUsageFetchError;

    fn window(used_percent: u8) -> RateWindowSnapshot {
        RateWindowSnapshot {
            remaining_percent: 100 - used_percent,
            used_percent,
            reset_at: "1234".into(),
        }
    }

    #[test]
    fn warmup_requires_an_unused_window_and_no_exhausted_window() {
        let mut quota = FetchedCodexUsage {
            plan: None,
            five_hour: None,
            weekly: None,
            credits_balance: None,
        };
        assert!(check_warmup_quota(&quota).is_err());
        quota.five_hour = Some(window(0));
        quota.weekly = Some(window(100));
        assert!(check_warmup_quota(&quota).is_err());
        quota.weekly = Some(window(1));
        assert!(check_warmup_quota(&quota).is_ok());
        quota.five_hour = Some(window(1));
        assert!(check_warmup_quota(&quota).is_err());
        quota.five_hour = None;
        quota.weekly = Some(window(0));
        assert!(check_warmup_quota(&quota).is_ok());
    }

    struct ExpectedAccountFetcher {
        home: PathBuf,
        results: Mutex<VecDeque<Result<FetchedCodexUsage, CodexUsageFetchError>>>,
    }

    impl CodexUsageFetcher for ExpectedAccountFetcher {
        fn fetch_usage(&self, home: &Path) -> Result<FetchedCodexUsage, CodexUsageFetchError> {
            assert_eq!(home.canonicalize().unwrap(), self.home);
            self.results.lock().unwrap().pop_front().unwrap()
        }
    }

    #[test]
    fn sends_once_for_selected_home_even_when_quota_recheck_fails() {
        let fixture = tempfile::tempdir().unwrap();
        let paths =
            CodexAccountPaths::for_test(fixture.path().join("app"), fixture.path().join("user"));
        let mut store = CodexAccountStore::load(&paths).unwrap();
        let home = paths.managed_homes_dir.join("selected");
        std::fs::create_dir(&home).unwrap();
        let home = home.canonicalize().unwrap();
        let claims = URL_SAFE_NO_PAD.encode(br#"{"email":"selected@example.test"}"#);
        let auth = json!({"tokens": {"id_token": format!("header.{claims}.signature")}});
        let auth_path = auth_path_for_home(&home);
        std::fs::write(&auth_path, auth.to_string()).unwrap();
        let identity = read_account_identity_from_path(&auth_path).unwrap();
        let account = store
            .upsert_identity(&paths, identity, home.clone())
            .unwrap();
        let quota = FetchedCodexUsage {
            plan: None,
            five_hour: Some(window(0)),
            weekly: None,
            credits_balance: None,
        };
        let service = CodexUsageService::new(
            paths.clone(),
            Box::new(ExpectedAccountFetcher {
                home: home.clone(),
                results: Mutex::new(VecDeque::from([
                    Ok(quota),
                    Err(CodexUsageFetchError::Unauthorized),
                ])),
            }),
        );
        let mut sends = 0;
        let refreshed = service
            .warm_account_with(&account.id, |selected_home| {
                assert_eq!(selected_home, home);
                sends += 1;
                Ok(())
            })
            .unwrap();
        assert_eq!(sends, 1);
        assert!(!refreshed);
        assert!(
            CodexUsageStore::load(&paths)
                .unwrap()
                .get(&account.id)
                .unwrap()
                .needs_relogin
        );
    }
}
