use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeAccountIdentity {
    pub email: String,
    pub display_name: Option<String>,
    pub plan: Option<String>,
    pub account_hint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredClaudeAccount {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub plan: Option<String>,
    pub account_hint: Option<String>,
    pub credential_bundle_key: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_authenticated_at: String,
    pub last_used_at: Option<String>,
}

impl StoredClaudeAccount {
    pub fn new_for_tests(email: &str, account_hint: Option<&str>) -> Self {
        Self {
            id: format!("test-{}", email.to_lowercase()),
            email: email.to_lowercase(),
            display_name: Some("Test Claude".to_string()),
            plan: Some("Pro".to_string()),
            account_hint: account_hint.map(str::to_string),
            credential_bundle_key: "test-bundle".to_string(),
            created_at: "0".to_string(),
            updated_at: "0".to_string(),
            last_authenticated_at: "0".to_string(),
            last_used_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaudeAccountListItem {
    pub id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub plan: Option<String>,
    pub account_hint: Option<String>,
    pub is_active: bool,
    pub last_authenticated_at: String,
    pub session_remaining_percent: Option<u8>,
    pub session_refresh_at: Option<String>,
    pub weekly_remaining_percent: Option<u8>,
    pub weekly_refresh_at: Option<String>,
    pub model_weekly_label: Option<String>,
    pub model_weekly_remaining_percent: Option<u8>,
    pub model_weekly_refresh_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub needs_relogin: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredClaudeAccountIndex {
    pub version: u8,
    pub accounts: Vec<StoredClaudeAccount>,
}
