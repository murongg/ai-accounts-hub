use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiAccountIdentity {
    pub email: String,
    pub subject: Option<String>,
    pub auth_type: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredGeminiAccount {
    pub id: String,
    pub email: String,
    pub subject: Option<String>,
    pub auth_type: Option<String>,
    pub managed_home_path: String,
    pub created_at: String,
    pub updated_at: String,
    pub last_authenticated_at: String,
}

impl StoredGeminiAccount {
    pub fn new_for_tests(email: &str, subject: Option<&str>) -> Self {
        Self {
            id: format!("test-{}", email.to_lowercase()),
            email: email.to_lowercase(),
            subject: subject.map(str::to_string),
            auth_type: Some("oauth-personal".to_string()),
            managed_home_path: "/tmp/test-home".to_string(),
            created_at: "0".to_string(),
            updated_at: "0".to_string(),
            last_authenticated_at: "0".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GeminiAccountListItem {
    pub id: String,
    pub email: String,
    pub subject: Option<String>,
    pub auth_type: Option<String>,
    pub plan: Option<String>,
    pub is_active: bool,
    pub last_authenticated_at: String,
    pub pro_remaining_percent: Option<u8>,
    pub flash_remaining_percent: Option<u8>,
    pub flash_lite_remaining_percent: Option<u8>,
    pub pro_refresh_at: Option<String>,
    pub flash_refresh_at: Option<String>,
    pub flash_lite_refresh_at: Option<String>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub needs_relogin: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGeminiAccountIndex {
    pub version: u8,
    pub accounts: Vec<StoredGeminiAccount>,
}
