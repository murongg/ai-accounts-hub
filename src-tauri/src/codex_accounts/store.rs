use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use super::models::{CodexAccountIdentity, StoredCodexAccount, StoredCodexAccountIndex};
use super::paths::{atomic_write, CodexAccountPaths};

pub struct CodexAccountStore {
    index: StoredCodexAccountIndex,
}

impl CodexAccountStore {
    pub fn load(paths: &CodexAccountPaths) -> Result<Self, String> {
        paths.ensure_dirs()?;

        let index = match std::fs::read_to_string(&paths.account_index_path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|error| format!("failed to parse account index: {error}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => StoredCodexAccountIndex {
                version: 1,
                accounts: Vec::new(),
            },
            Err(error) => return Err(format!("failed to read account index: {error}")),
        };

        Ok(Self { index })
    }

    pub fn accounts(&self) -> &[StoredCodexAccount] {
        &self.index.accounts
    }

    pub fn find_matching_account(&self, identity: &CodexAccountIdentity) -> Option<&StoredCodexAccount> {
        self.index.accounts.iter().find(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.account_id.is_some() && account.account_id == identity.account_id)
        })
    }

    pub fn upsert_identity(
        &mut self,
        paths: &CodexAccountPaths,
        identity: CodexAccountIdentity,
        managed_home_path: PathBuf,
    ) -> Result<StoredCodexAccount, String> {
        let now = timestamp_string();
        let existing_index = self.index.accounts.iter().position(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.account_id.is_some() && account.account_id == identity.account_id)
        });

        let saved = if let Some(index) = existing_index {
            let existing = &self.index.accounts[index];
            StoredCodexAccount {
                id: existing.id.clone(),
                email: identity.email,
                account_id: identity.account_id,
                plan: identity.plan,
                managed_home_path: managed_home_path.display().to_string(),
                created_at: existing.created_at.clone(),
                updated_at: now.clone(),
                last_authenticated_at: now,
            }
        } else {
            StoredCodexAccount {
                id: Uuid::new_v4().to_string(),
                email: identity.email,
                account_id: identity.account_id,
                plan: identity.plan,
                managed_home_path: managed_home_path.display().to_string(),
                created_at: now.clone(),
                updated_at: now.clone(),
                last_authenticated_at: now,
            }
        };

        if let Some(index) = existing_index {
            self.index.accounts[index] = saved.clone();
        } else {
            self.index.accounts.push(saved.clone());
        }

        self.persist(paths)?;
        Ok(saved)
    }

    pub fn delete(&mut self, paths: &CodexAccountPaths, account_id: &str) -> Result<(), String> {
        self.index.accounts.retain(|account| account.id != account_id);
        self.persist(paths)
    }

    pub fn find_by_id(&self, account_id: &str) -> Option<&StoredCodexAccount> {
        self.index.accounts.iter().find(|account| account.id == account_id)
    }

    fn persist(&self, paths: &CodexAccountPaths) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&self.index)
            .map_err(|error| format!("failed to serialize account index: {error}"))?;
        atomic_write(&paths.account_index_path, &bytes)
    }
}

pub fn auth_path_for_home(managed_home_path: &Path) -> PathBuf {
    managed_home_path.join("auth.json")
}

fn timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
