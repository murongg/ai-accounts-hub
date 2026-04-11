use serde::{Deserialize, Serialize};

pub const DEFAULT_REFRESH_INTERVAL_SECONDS: u64 = 300;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexRefreshSettings {
    pub enabled: bool,
    pub interval_seconds: u64,
}

impl Default for CodexRefreshSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            interval_seconds: DEFAULT_REFRESH_INTERVAL_SECONDS,
        }
    }
}

impl CodexRefreshSettings {
    pub fn sanitized(self) -> Self {
        Self {
            enabled: self.enabled,
            interval_seconds: self.interval_seconds.max(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RateWindowSnapshot {
    pub remaining_percent: u8,
    pub used_percent: u8,
    pub reset_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodexUsageSnapshot {
    pub managed_account_id: String,
    pub plan: Option<String>,
    pub five_hour: Option<RateWindowSnapshot>,
    pub weekly: Option<RateWindowSnapshot>,
    pub credits_balance: Option<f64>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodexUsageSnapshotIndex {
    pub version: u8,
    pub snapshots: Vec<CodexUsageSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchedCodexUsage {
    pub plan: Option<String>,
    pub five_hour: Option<RateWindowSnapshot>,
    pub weekly: Option<RateWindowSnapshot>,
    pub credits_balance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodexUsageApiResponse {
    pub plan_type: Option<String>,
    pub rate_limit: Option<CodexUsageRateLimit>,
    pub credits: Option<CodexUsageCredits>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodexUsageRateLimit {
    pub primary_window: Option<CodexUsageApiWindow>,
    pub secondary_window: Option<CodexUsageApiWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CodexUsageApiWindow {
    pub used_percent: u8,
    pub reset_at: u64,
    pub limit_window_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CodexUsageCredits {
    pub has_credits: bool,
    pub unlimited: bool,
    #[serde(default, deserialize_with = "deserialize_optional_f64")]
    pub balance: Option<f64>,
}

fn deserialize_optional_f64<'de, D>(deserializer: D) -> Result<Option<f64>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::Number(number)) => number
            .as_f64()
            .ok_or_else(|| serde::de::Error::custom("invalid numeric credits balance"))
            .map(Some),
        Some(serde_json::Value::String(raw)) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                trimmed.parse::<f64>().map(Some).map_err(|error| {
                    serde::de::Error::custom(format!("invalid string credits balance: {error}"))
                })
            }
        }
        Some(other) => Err(serde::de::Error::custom(format!(
            "unsupported credits balance type: {other}"
        ))),
    }
}
