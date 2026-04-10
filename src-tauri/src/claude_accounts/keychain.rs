use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "macos")]
const MANAGED_CLAUDE_BUNDLE_SERVICE: &str = "ai-accounts-hub.claude-account-bundle";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeCredentialBundle {
    pub email: String,
    pub credentials_json: Vec<u8>,
    pub oauth_account_json: Option<Vec<u8>>,
    pub account_hint: Option<String>,
}

pub trait ClaudeCredentialBundleStore: Send + Sync {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String>;
    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

#[derive(Default)]
pub struct InMemoryClaudeKeychainStore(BTreeMap<String, ClaudeCredentialBundle>);

impl InMemoryClaudeKeychainStore {
    pub fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        ClaudeCredentialBundleStore::save(self, key, bundle)
    }

    pub fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        ClaudeCredentialBundleStore::load(self, key)
    }

    pub fn delete(&mut self, key: &str) -> Result<(), String> {
        ClaudeCredentialBundleStore::delete(self, key)
    }
}

impl ClaudeCredentialBundleStore for InMemoryClaudeKeychainStore {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        self.0.insert(key.to_string(), bundle.clone());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        Ok(self.0.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.0.remove(key);
        Ok(())
    }
}

#[cfg(target_os = "macos")]
pub struct ManagedClaudeKeychainStore;

#[cfg(target_os = "macos")]
impl ManagedClaudeKeychainStore {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "macos")]
impl ClaudeCredentialBundleStore for ManagedClaudeKeychainStore {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        let payload = serde_json::to_vec(bundle)
            .map_err(|error| format!("failed to serialize Claude credential bundle: {error}"))?;
        security_framework::passwords::set_generic_password(
            MANAGED_CLAUDE_BUNDLE_SERVICE,
            key,
            &payload,
        )
        .map_err(|error| format!("failed to save Claude credential bundle in Keychain: {error}"))
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        match security_framework::passwords::get_generic_password(
            MANAGED_CLAUDE_BUNDLE_SERVICE,
            key,
        ) {
            Ok(payload) => serde_json::from_slice(&payload).map(Some).map_err(|error| {
                format!("failed to parse Claude credential bundle from Keychain: {error}")
            }),
            Err(error) => {
                if error.code() == -25300 {
                    Ok(None)
                } else {
                    Err(format!(
                        "failed to load Claude credential bundle from Keychain: {error}"
                    ))
                }
            }
        }
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        match security_framework::passwords::delete_generic_password(
            MANAGED_CLAUDE_BUNDLE_SERVICE,
            key,
        ) {
            Ok(()) => Ok(()),
            Err(error) => {
                if error.code() == -25300 {
                    Ok(())
                } else {
                    Err(format!(
                        "failed to delete Claude credential bundle from Keychain: {error}"
                    ))
                }
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub struct ManagedClaudeKeychainStore {
    fallback: BTreeMap<String, ClaudeCredentialBundle>,
}

#[cfg(not(target_os = "macos"))]
impl ManagedClaudeKeychainStore {
    pub fn new() -> Self {
        Self {
            fallback: BTreeMap::new(),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl ClaudeCredentialBundleStore for ManagedClaudeKeychainStore {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        self.fallback.insert(key.to_string(), bundle.clone());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        Ok(self.fallback.get(key).cloned())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.fallback.remove(key);
        Ok(())
    }
}
