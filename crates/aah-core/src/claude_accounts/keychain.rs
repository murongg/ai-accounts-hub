use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};

use crate::fs_utils::atomic_write;

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

#[derive(Clone, Default)]
pub struct InMemoryClaudeKeychainStore(Arc<Mutex<BTreeMap<String, ClaudeCredentialBundle>>>);

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
        self.0
            .lock()
            .map_err(|_| "Claude in-memory keychain lock poisoned".to_string())?
            .insert(key.to_string(), bundle.clone());
        Ok(())
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        Ok(self
            .0
            .lock()
            .map_err(|_| "Claude in-memory keychain lock poisoned".to_string())?
            .get(key)
            .cloned())
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.0
            .lock()
            .map_err(|_| "Claude in-memory keychain lock poisoned".to_string())?
            .remove(key);
        Ok(())
    }
}

pub struct FileClaudeBundleStore {
    bundle_dir: PathBuf,
}

impl FileClaudeBundleStore {
    pub fn new(bundle_dir: PathBuf) -> Self {
        Self { bundle_dir }
    }

    fn bundle_path(&self, key: &str) -> Result<PathBuf, String> {
        if key.is_empty()
            || !key
                .chars()
                .all(|char| char.is_ascii_alphanumeric() || char == '-' || char == '_')
        {
            return Err(format!("invalid Claude credential bundle key: {key}"));
        }

        Ok(self.bundle_dir.join(format!("{key}.json")))
    }
}

impl ClaudeCredentialBundleStore for FileClaudeBundleStore {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        let path = self.bundle_path(key)?;
        fs::create_dir_all(&self.bundle_dir)
            .map_err(|error| format!("failed to create Claude bundle dir: {error}"))?;
        let payload = serde_json::to_vec(bundle)
            .map_err(|error| format!("failed to serialize Claude credential bundle: {error}"))?;
        atomic_write(&path, &payload)
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        let path = self.bundle_path(key)?;
        let payload = match fs::read(&path) {
            Ok(payload) => payload,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "failed to read Claude credential bundle from {}: {error}",
                    path.display()
                ))
            }
        };

        serde_json::from_slice(&payload).map(Some).map_err(|error| {
            format!(
                "failed to parse Claude credential bundle from {}: {error}",
                path.display()
            )
        })
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        let path = self.bundle_path(key)?;
        match fs::remove_file(&path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "failed to delete Claude credential bundle {}: {error}",
                path.display()
            )),
        }
    }
}

#[cfg(target_os = "macos")]
pub struct ManagedClaudeKeychainStore;

#[cfg(target_os = "macos")]
impl ManagedClaudeKeychainStore {
    pub fn new(_bundle_dir: PathBuf) -> Self {
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
    file_store: FileClaudeBundleStore,
}

#[cfg(not(target_os = "macos"))]
impl ManagedClaudeKeychainStore {
    pub fn new(bundle_dir: PathBuf) -> Self {
        Self {
            file_store: FileClaudeBundleStore::new(bundle_dir),
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl ClaudeCredentialBundleStore for ManagedClaudeKeychainStore {
    fn save(&mut self, key: &str, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        self.file_store.save(key, bundle)
    }

    fn load(&self, key: &str) -> Result<Option<ClaudeCredentialBundle>, String> {
        self.file_store.load(key)
    }

    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.file_store.delete(key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn file_bundle_store_persists_bundles_across_instances() {
        let temp = TempDir::new("claude-file-bundle-store");
        let bundle_dir = temp.path().join("managed-credential-bundles");
        let bundle = ClaudeCredentialBundle {
            email: "murong@example.com".to_string(),
            credentials_json: br#"{"claudeAiOauth":{"accessToken":"token"}}"#.to_vec(),
            oauth_account_json: Some(br#"{"emailAddress":"murong@example.com"}"#.to_vec()),
            account_hint: Some("owner-a".to_string()),
        };

        let mut writer = FileClaudeBundleStore::new(bundle_dir.clone());
        writer
            .save("claude-bundle-1", &bundle)
            .expect("save bundle");

        let reader = FileClaudeBundleStore::new(bundle_dir);
        assert_eq!(
            reader.load("claude-bundle-1").expect("load bundle"),
            Some(bundle),
        );
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!("aihub-{prefix}-{unique}"));
            fs::create_dir_all(&path).expect("temp dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }
}
