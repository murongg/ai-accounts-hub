use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use super::models::{GeminiAccountIdentity, StoredGeminiAccount, StoredGeminiAccountIndex};
use super::paths::{atomic_write, GeminiAccountPaths};

pub struct GeminiAccountStore {
    index: StoredGeminiAccountIndex,
}

impl GeminiAccountStore {
    pub fn load(paths: &GeminiAccountPaths) -> Result<Self, String> {
        paths.ensure_dirs()?;

        let index = match std::fs::read_to_string(&paths.account_index_path) {
            Ok(text) => serde_json::from_str(&text)
                .map_err(|error| format!("failed to parse Gemini account index: {error}"))?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                StoredGeminiAccountIndex {
                    version: 1,
                    accounts: Vec::new(),
                }
            }
            Err(error) => return Err(format!("failed to read Gemini account index: {error}")),
        };

        Ok(Self { index })
    }

    pub fn accounts(&self) -> &[StoredGeminiAccount] {
        &self.index.accounts
    }

    pub fn find_matching_account(
        &self,
        identity: &GeminiAccountIdentity,
    ) -> Option<&StoredGeminiAccount> {
        self.index.accounts.iter().find(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.subject.is_some() && account.subject == identity.subject)
        })
    }

    pub fn upsert_identity(
        &mut self,
        paths: &GeminiAccountPaths,
        identity: GeminiAccountIdentity,
        managed_home_path: PathBuf,
    ) -> Result<StoredGeminiAccount, String> {
        let now = timestamp_string();
        let existing_index = self.index.accounts.iter().position(|account| {
            account.email.eq_ignore_ascii_case(&identity.email)
                || (identity.subject.is_some() && account.subject == identity.subject)
        });

        let saved = if let Some(index) = existing_index {
            let existing = &self.index.accounts[index];
            StoredGeminiAccount {
                id: existing.id.clone(),
                email: identity.email,
                subject: identity.subject,
                auth_type: identity.auth_type,
                managed_home_path: managed_home_path.display().to_string(),
                created_at: existing.created_at.clone(),
                updated_at: now.clone(),
                last_authenticated_at: now,
            }
        } else {
            StoredGeminiAccount {
                id: Uuid::new_v4().to_string(),
                email: identity.email,
                subject: identity.subject,
                auth_type: identity.auth_type,
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

    pub fn delete(&mut self, paths: &GeminiAccountPaths, account_id: &str) -> Result<(), String> {
        self.index
            .accounts
            .retain(|account| account.id != account_id);
        self.persist(paths)
    }

    pub fn find_by_id(&self, account_id: &str) -> Option<&StoredGeminiAccount> {
        self.index
            .accounts
            .iter()
            .find(|account| account.id == account_id)
    }

    fn persist(&self, paths: &GeminiAccountPaths) -> Result<(), String> {
        let bytes = serde_json::to_vec_pretty(&self.index)
            .map_err(|error| format!("failed to serialize Gemini account index: {error}"))?;
        atomic_write(&paths.account_index_path, &bytes)
    }
}

fn timestamp_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}
