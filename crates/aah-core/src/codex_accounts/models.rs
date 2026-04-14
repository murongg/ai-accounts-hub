use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodexAccountIdentity {
    pub email: String,
    pub account_id: Option<String>,
    pub plan: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredCodexAccount {
    pub id: String,
    pub email: String,
    pub account_id: Option<String>,
    pub plan: Option<String>,
    pub managed_home_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_authenticated_at: String,
}

impl StoredCodexAccount {
    pub fn new_for_tests(email: &str, account_id: Option<&str>) -> Self {
        Self {
            id: format!("test-{}", email.to_lowercase()),
            email: email.to_lowercase(),
            account_id: account_id.map(str::to_string),
            plan: Some("Plus".to_string()),
            managed_home_path: "/tmp/test-home".to_string(),
            created_at: "0".to_string(),
            updated_at: "0".to_string(),
            last_authenticated_at: "0".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CodexAccountListItem {
    pub id: String,
    pub email: String,
    pub plan: Option<String>,
    pub account_id: Option<String>,
    pub is_active: bool,
    pub last_authenticated_at: String,
    pub five_hour_remaining_percent: Option<u8>,
    pub weekly_remaining_percent: Option<u8>,
    pub five_hour_refresh_at: Option<String>,
    pub weekly_refresh_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub credits_balance: Option<f64>,
    pub needs_relogin: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredCodexAccountIndex {
    pub version: u8,
    pub accounts: Vec<StoredCodexAccount>,
}
