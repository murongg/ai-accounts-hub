use std::fs;
use std::sync::{Arc, Mutex};

use serde_json::{Map, Value};
#[cfg(target_os = "macos")]
use sha2::{Digest, Sha256};

use super::keychain::ClaudeCredentialBundle;
use super::paths::{atomic_write, ClaudeAccountPaths};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeLiveCredentialSnapshot {
    pub credentials_json: Vec<u8>,
    pub oauth_account_json: Option<Vec<u8>>,
}

#[derive(Debug, Default)]
pub struct ClaudeLiveCredentialState {
    pub credentials_json: Option<Vec<u8>>,
    pub oauth_account_json: Option<Vec<u8>>,
}

pub trait ClaudeLiveCredentialStore: Send {
    fn capture(&self) -> Result<ClaudeLiveCredentialSnapshot, String>;
    fn capture_login_session(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        self.capture()
    }
    fn restore(&mut self, bundle: &ClaudeCredentialBundle) -> Result<(), String>;
}

#[derive(Clone, Default)]
pub struct InMemoryClaudeLiveCredentialStore {
    state: Arc<Mutex<Option<ClaudeLiveCredentialSnapshot>>>,
}

impl InMemoryClaudeLiveCredentialStore {
    pub fn new(snapshot: ClaudeLiveCredentialSnapshot) -> Self {
        Self {
            state: Arc::new(Mutex::new(Some(snapshot))),
        }
    }

    pub fn set_snapshot(&self, snapshot: ClaudeLiveCredentialSnapshot) {
        if let Ok(mut state) = self.state.lock() {
            *state = Some(snapshot);
        }
    }

    pub fn capture(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        ClaudeLiveCredentialStore::capture(self)
    }
}

impl ClaudeLiveCredentialStore for InMemoryClaudeLiveCredentialStore {
    fn capture(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        self.state
            .lock()
            .map_err(|_| "live Claude credential state lock poisoned".to_string())?
            .clone()
            .ok_or_else(|| "live Claude credentials are unavailable".to_string())
    }

    fn restore(&mut self, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| "live Claude credential state lock poisoned".to_string())?;
        *state = Some(ClaudeLiveCredentialSnapshot {
            credentials_json: bundle.credentials_json.clone(),
            oauth_account_json: bundle.oauth_account_json.clone(),
        });
        Ok(())
    }
}

pub struct FileSystemClaudeLiveCredentialStore {
    paths: ClaudeAccountPaths,
}

impl FileSystemClaudeLiveCredentialStore {
    pub fn new(paths: ClaudeAccountPaths) -> Self {
        Self { paths }
    }
}

impl ClaudeCredentialBundle {
    pub fn from_live_snapshot(
        email: &str,
        account_hint: Option<&str>,
        snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Self {
        Self {
            email: email.to_string(),
            credentials_json: snapshot.credentials_json.clone(),
            oauth_account_json: snapshot.oauth_account_json.clone(),
            account_hint: account_hint.map(str::to_string),
        }
    }
}

impl ClaudeLiveCredentialState {
    pub fn restore(&mut self, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        self.credentials_json = Some(bundle.credentials_json.clone());
        self.oauth_account_json = bundle.oauth_account_json.clone();
        Ok(())
    }
}

impl ClaudeLiveCredentialStore for FileSystemClaudeLiveCredentialStore {
    fn capture(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        let credentials_json = read_live_secure_storage(
            &self.paths.system_claude_dir,
            &self.paths.system_credentials_path,
            false,
        )?;

        Ok(ClaudeLiveCredentialSnapshot {
            credentials_json,
            oauth_account_json: read_live_oauth_account(&self.paths.system_global_config_path)?,
        })
    }

    fn capture_login_session(&self) -> Result<ClaudeLiveCredentialSnapshot, String> {
        let credentials_json = read_live_secure_storage(
            &self.paths.login_claude_dir,
            &self.paths.login_credentials_path,
            true,
        )?;

        Ok(ClaudeLiveCredentialSnapshot {
            credentials_json,
            oauth_account_json: read_live_oauth_account(&self.paths.login_global_config_path)?,
        })
    }

    fn restore(&mut self, bundle: &ClaudeCredentialBundle) -> Result<(), String> {
        self.paths.ensure_dirs()?;
        write_live_secure_storage(
            &self.paths.system_claude_dir,
            &self.paths.system_credentials_path,
            false,
            &bundle.credentials_json,
        )?;
        write_live_oauth_account(
            &self.paths.system_global_config_path,
            bundle.oauth_account_json.as_deref(),
        )?;
        Ok(())
    }
}

fn read_live_oauth_account(
    global_config_path: &std::path::Path,
) -> Result<Option<Vec<u8>>, String> {
    let bytes = match fs::read(global_config_path) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "failed to read Claude global config from {}: {error}",
                global_config_path.display()
            ))
        }
    };

    let config: Value = serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "failed to parse Claude global config from {}: {error}",
            global_config_path.display()
        )
    })?;

    config
        .get("oauthAccount")
        .map(|value| {
            serde_json::to_vec(value)
                .map_err(|error| format!("failed to serialize Claude oauthAccount json: {error}"))
        })
        .transpose()
}

fn write_live_oauth_account(
    global_config_path: &std::path::Path,
    oauth_account_json: Option<&[u8]>,
) -> Result<(), String> {
    let Some(oauth_account_json) = oauth_account_json else {
        return Ok(());
    };

    let oauth_account: Value = serde_json::from_slice(oauth_account_json)
        .map_err(|error| format!("failed to parse Claude oauthAccount json: {error}"))?;

    let mut config = match fs::read(global_config_path) {
        Ok(bytes) => serde_json::from_slice::<Value>(&bytes).map_err(|error| {
            format!(
                "failed to parse existing Claude global config from {}: {error}",
                global_config_path.display()
            )
        })?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Value::Object(Map::new()),
        Err(error) => {
            return Err(format!(
                "failed to read existing Claude global config from {}: {error}",
                global_config_path.display()
            ))
        }
    };

    let object = config
        .as_object_mut()
        .ok_or_else(|| "Claude global config is not a JSON object".to_string())?;
    object.insert("oauthAccount".to_string(), oauth_account);

    let bytes = serde_json::to_vec_pretty(&config)
        .map_err(|error| format!("failed to serialize Claude global config: {error}"))?;
    atomic_write(global_config_path, &bytes)
}

fn read_plaintext_secure_storage(path: &std::path::Path) -> Result<Option<Vec<u8>>, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!(
            "failed to read Claude secure storage fallback from {}: {error}",
            path.display()
        )),
    }
}

fn write_plaintext_secure_storage(path: &std::path::Path, payload: &[u8]) -> Result<(), String> {
    atomic_write(path, payload)
}

#[cfg(target_os = "macos")]
fn read_live_secure_storage(
    config_dir: &std::path::Path,
    fallback_path: &std::path::Path,
    is_custom_config_dir: bool,
) -> Result<Vec<u8>, String> {
    let service = claude_keychain_service_name(config_dir, is_custom_config_dir);
    let account = current_username();

    match security_framework::passwords::get_generic_password(&service, &account) {
        Ok(payload) => Ok(payload),
        Err(_) => read_plaintext_secure_storage(fallback_path)?.ok_or_else(|| {
            format!(
                "Claude secure storage is unavailable for {}",
                config_dir.display()
            )
        }),
    }
}

#[cfg(not(target_os = "macos"))]
fn read_live_secure_storage(
    _config_dir: &std::path::Path,
    fallback_path: &std::path::Path,
    _is_custom_config_dir: bool,
) -> Result<Vec<u8>, String> {
    read_plaintext_secure_storage(fallback_path)?.ok_or_else(|| {
        format!(
            "Claude secure storage is unavailable for {}",
            fallback_path.display()
        )
    })
}

#[cfg(target_os = "macos")]
fn write_live_secure_storage(
    config_dir: &std::path::Path,
    fallback_path: &std::path::Path,
    is_custom_config_dir: bool,
    payload: &[u8],
) -> Result<(), String> {
    let service = claude_keychain_service_name(config_dir, is_custom_config_dir);
    let account = current_username();

    match security_framework::passwords::set_generic_password(&service, &account, payload) {
        Ok(()) => {
            let _ = fs::remove_file(fallback_path);
            Ok(())
        }
        Err(_) => {
            write_plaintext_secure_storage(fallback_path, payload)?;
            let _ = security_framework::passwords::delete_generic_password(&service, &account);
            Ok(())
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn write_live_secure_storage(
    _config_dir: &std::path::Path,
    fallback_path: &std::path::Path,
    _is_custom_config_dir: bool,
    payload: &[u8],
) -> Result<(), String> {
    write_plaintext_secure_storage(fallback_path, payload)
}

#[cfg(target_os = "macos")]
fn claude_keychain_service_name(
    config_dir: &std::path::Path,
    is_custom_config_dir: bool,
) -> String {
    let mut service = "Claude Code-credentials".to_string();
    if is_custom_config_dir {
        let hash = Sha256::digest(config_dir.to_string_lossy().as_bytes());
        let mut suffix = String::new();
        for byte in &hash[..4] {
            suffix.push_str(&format!("{byte:02x}"));
        }
        service.push('-');
        service.push_str(&suffix);
    }
    service
}

#[cfg(target_os = "macos")]
fn current_username() -> String {
    std::env::var("USER")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "claude-code-user".to_string())
}

#[cfg(test)]
mod tests {
    use super::FileSystemClaudeLiveCredentialStore;
    use crate::claude_accounts::live_credentials::ClaudeLiveCredentialStore;
    use crate::claude_accounts::paths::ClaudeAccountPaths;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

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

    #[test]
    fn capture_reads_the_system_claude_credentials() {
        let temp = TempDir::new("claude-live-system");
        let paths =
            ClaudeAccountPaths::from_roots(temp.path().join("app-data"), temp.path().join("home"));

        fs::create_dir_all(&paths.login_claude_dir).expect("login dir");
        fs::write(
            &paths.login_credentials_path,
            br#"{"email":"login@example.com"}"#,
        )
        .expect("login credentials");
        fs::write(
            &paths.login_global_config_path,
            br#"{"oauthAccount":{"emailAddress":"login@example.com"}}"#,
        )
        .expect("login config");

        fs::create_dir_all(&paths.system_claude_dir).expect("system dir");
        fs::write(
            &paths.system_credentials_path,
            br#"{"email":"system@example.com"}"#,
        )
        .expect("system credentials");
        fs::write(
            &paths.system_global_config_path,
            br#"{"oauthAccount":{"emailAddress":"system@example.com"}}"#,
        )
        .expect("system config");

        let store = FileSystemClaudeLiveCredentialStore::new(paths);
        let snapshot = store.capture().expect("capture system snapshot");

        let credentials_json = String::from_utf8(snapshot.credentials_json).expect("utf8 json");
        let oauth_json = String::from_utf8(snapshot.oauth_account_json.expect("oauth json"))
            .expect("utf8 oauth json");
        assert!(!credentials_json.contains("login@example.com"));
        assert!(oauth_json.contains("system@example.com"));
        assert!(!oauth_json.contains("login@example.com"));
    }
}
