use std::fs;
use std::path::Path;

use crate::claude_accounts::live_credentials::{
    ClaudeLiveCredentialSnapshot, ClaudeLiveCredentialStore, FileSystemClaudeLiveCredentialStore,
};
use crate::claude_accounts::paths::ClaudeAccountPaths;
use crate::claude_usage::oauth::extract_oauth_credentials;
use crate::codex_accounts::auth::read_auth_value;
use crate::codex_accounts::paths::CodexAccountPaths;
use crate::gemini_accounts::paths::GeminiAccountPaths;
use crate::gemini_usage::oauth::read_fresh_gemini_oauth_credentials;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RelayProvider {
    Codex,
    Claude,
    Gemini,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RelayProviderCredential {
    pub provider: RelayProvider,
    pub upstream_base_url: String,
    pub bearer_token: String,
    pub extra_headers: Vec<(String, String)>,
}

pub trait RelayCredentialSource: Send + Sync + 'static {
    fn credential_for(&self, provider: RelayProvider) -> Result<RelayProviderCredential, String>;
}

pub struct EmptyRelayCredentialSource;

impl RelayCredentialSource for EmptyRelayCredentialSource {
    fn credential_for(&self, _provider: RelayProvider) -> Result<RelayProviderCredential, String> {
        Err("relay credentials are unavailable in this context".to_string())
    }
}

pub fn codex_credential_from_auth(
    auth: &Value,
    upstream_base_url: String,
) -> Result<RelayProviderCredential, String> {
    let tokens = auth
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or_else(|| "Codex auth.json is missing tokens".to_string())?;
    let bearer_token = tokens
        .get("access_token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Codex auth.json is missing tokens.access_token".to_string())?
        .to_string();
    let extra_headers = tokens
        .get("account_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| vec![("ChatGPT-Account-Id".to_string(), value.to_string())])
        .unwrap_or_default();

    Ok(RelayProviderCredential {
        provider: RelayProvider::Codex,
        upstream_base_url,
        bearer_token,
        extra_headers,
    })
}

pub fn claude_credential_from_snapshot(
    snapshot: &ClaudeLiveCredentialSnapshot,
) -> Result<RelayProviderCredential, String> {
    let credentials = extract_oauth_credentials(snapshot).map_err(|error| error.to_string())?;
    Ok(RelayProviderCredential {
        provider: RelayProvider::Claude,
        upstream_base_url: "https://api.anthropic.com".to_string(),
        bearer_token: credentials.access_token,
        extra_headers: vec![("anthropic-beta".to_string(), "oauth-2025-04-20".to_string())],
    })
}

pub fn gemini_credential_from_oauth(oauth: &Value) -> Result<RelayProviderCredential, String> {
    let bearer_token = oauth
        .get("access_token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "Gemini oauth_creds.json is missing access_token".to_string())?
        .to_string();

    Ok(RelayProviderCredential {
        provider: RelayProvider::Gemini,
        upstream_base_url: "https://cloudcode-pa.googleapis.com".to_string(),
        bearer_token,
        extra_headers: Vec::new(),
    })
}

pub fn normalize_chatgpt_base_url(base: &str) -> String {
    let mut normalized = base.trim().trim_end_matches('/').to_string();
    if normalized.is_empty() {
        normalized = "https://chatgpt.com/backend-api".into();
    }
    if (normalized.starts_with("https://chatgpt.com")
        || normalized.starts_with("https://chat.openai.com"))
        && !normalized.contains("/backend-api")
    {
        normalized.push_str("/backend-api");
    }
    normalized
}

pub fn parse_chatgpt_base_url(contents: &str) -> Option<String> {
    for raw_line in contents.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        let key = parts.next()?.trim();
        if key != "chatgpt_base_url" {
            continue;
        }
        let value = parts
            .next()?
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .trim();
        if !value.is_empty() {
            return Some(value.to_string());
        }
    }
    None
}

pub fn read_optional_chatgpt_base_url(config_path: &Path) -> String {
    fs::read_to_string(config_path)
        .ok()
        .and_then(|contents| parse_chatgpt_base_url(&contents))
        .map(|value| normalize_chatgpt_base_url(&value))
        .unwrap_or_else(|| "https://chatgpt.com/backend-api".to_string())
}

pub struct LiveRelayCredentialSource {
    codex_paths: CodexAccountPaths,
    claude_paths: ClaudeAccountPaths,
    gemini_paths: GeminiAccountPaths,
}

impl LiveRelayCredentialSource {
    pub fn new(
        codex_paths: CodexAccountPaths,
        claude_paths: ClaudeAccountPaths,
        gemini_paths: GeminiAccountPaths,
    ) -> Self {
        Self {
            codex_paths,
            claude_paths,
            gemini_paths,
        }
    }
}

impl RelayCredentialSource for LiveRelayCredentialSource {
    fn credential_for(&self, provider: RelayProvider) -> Result<RelayProviderCredential, String> {
        match provider {
            RelayProvider::Codex => {
                let auth = read_auth_value(&self.codex_paths.system_auth_path)?;
                let upstream_base_url = read_optional_chatgpt_base_url(
                    &self.codex_paths.system_codex_dir.join("config.toml"),
                );
                codex_credential_from_auth(&auth, upstream_base_url)
            }
            RelayProvider::Claude => {
                let store = FileSystemClaudeLiveCredentialStore::new(self.claude_paths.clone());
                let snapshot = store.capture()?;
                claude_credential_from_snapshot(&snapshot)
            }
            RelayProvider::Gemini => {
                let credentials =
                    read_fresh_gemini_oauth_credentials(&self.gemini_paths.system_gemini_dir)
                        .map_err(|error| error.to_string())?;
                gemini_credential_from_oauth(&credentials.oauth_creds)
            }
        }
    }
}
