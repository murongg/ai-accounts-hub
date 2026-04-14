use uuid::Uuid;

use super::models::{ClaudeAccountIdentity, StoredClaudeAccount, StoredClaudeAccountIndex};
use super::paths::{atomic_write, ClaudeAccountPaths};
use crate::time_utils::timestamp_string;

pub struct ClaudeAccountStore {
    index: StoredClaudeAccountIndex,
}

impl ClaudeAccountStore {
    pub fn load(paths: &ClaudeAccountPaths) -> Result<Self, String> {
        paths.ensure_dirs()?;

        let index = match std::fs::read_to_string(&paths.metadata_index_path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|error| format!("failed to parse Claude account index: {error}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                StoredClaudeAccountIndex {
                    version: 1,
                    accounts: Vec::new(),
                }
            }
            Err(error) => return Err(format!("failed to read Claude account index: {error}")),
        };

        Ok(Self { index })
    }

    pub fn accounts(&self) -> &[StoredClaudeAccount] {
        &self.index.accounts
    }

    pub fn find_matching_account(
        &self,
        identity: &ClaudeAccountIdentity,
    ) -> Option<&StoredClaudeAccount> {
        self.index.accounts.iter().find(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.account_hint.is_some()
                    && account.account_hint == identity.account_hint)
        })
    }

    pub fn upsert_identity(
        &mut self,
        paths: &ClaudeAccountPaths,
        identity: ClaudeAccountIdentity,
        credential_bundle_key: String,
    ) -> Result<StoredClaudeAccount, String> {
        let now = timestamp_string();
        let existing_index = self.index.accounts.iter().position(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.account_hint.is_some()
                    && account.account_hint == identity.account_hint)
        });

        let saved = if let Some(index) = existing_index {
            let existing = &self.index.accounts[index];
            StoredClaudeAccount {
                id: existing.id.clone(),
                email: identity.email,
                display_name: identity.display_name,
                plan: identity.plan,
                account_hint: identity.account_hint,
                credential_bundle_key,
                created_at: existing.created_at.clone(),
                updated_at: now.clone(),
                last_authenticated_at: now,
                last_used_at: existing.last_used_at.clone(),
            }
        } else {
            StoredClaudeAccount {
                id: Uuid::new_v4().to_string(),
                email: identity.email,
                display_name: identity.display_name,
                plan: identity.plan,
                account_hint: identity.account_hint,
                credential_bundle_key,
                created_at: now.clone(),
                updated_at: now.clone(),
                last_authenticated_at: now,
                last_used_at: None,
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

    pub fn delete(&mut self, paths: &ClaudeAccountPaths, account_id: &str) -> Result<(), String> {
        self.index
            .accounts
            .retain(|account| account.id != account_id);
        self.persist(paths)
    }

    pub fn find_by_id(&self, account_id: &str) -> Option<&StoredClaudeAccount> {
        self.index
            .accounts
            .iter()
            .find(|account| account.id == account_id)
    }

    fn persist(&self, paths: &ClaudeAccountPaths) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&self.index)
            .map_err(|error| format!("failed to serialize Claude account index: {error}"))?;
        atomic_write(&paths.metadata_index_path, &bytes)
    }
}
