use serde::{Deserialize, Serialize};

use crate::codex_usage::models::RateWindowSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeminiUsageSnapshot {
    pub managed_account_id: String,
    pub plan: Option<String>,
    pub pro: Option<RateWindowSnapshot>,
    pub flash: Option<RateWindowSnapshot>,
    pub flash_lite: Option<RateWindowSnapshot>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiUsageSnapshotIndex {
    pub version: u8,
    pub snapshots: Vec<GeminiUsageSnapshot>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FetchedGeminiUsage {
    pub plan: Option<String>,
    pub pro: Option<RateWindowSnapshot>,
    pub flash: Option<RateWindowSnapshot>,
    pub flash_lite: Option<RateWindowSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeminiUsageApiResponse {
    pub buckets: Option<Vec<GeminiUsageApiBucket>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeminiUsageApiBucket {
    #[serde(rename = "remainingFraction")]
    pub remaining_fraction: Option<f64>,
    #[serde(rename = "resetTime")]
    pub reset_time: Option<String>,
    #[serde(rename = "modelId")]
    pub model_id: Option<String>,
    #[serde(rename = "remainingAmount")]
    pub remaining_amount: Option<String>,
    #[serde(rename = "tokenType")]
    pub token_type: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeminiTierId {
    Free,
    Legacy,
    Standard,
}

impl GeminiTierId {
    pub fn from_api_id(raw: &str) -> Option<Self> {
        match raw.trim() {
            "free-tier" => Some(Self::Free),
            "legacy-tier" => Some(Self::Legacy),
            "standard-tier" => Some(Self::Standard),
            _ => None,
        }
    }
}
