use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeRateWindowSnapshot {
    pub remaining_percent: u8,
    pub used_percent: u8,
    pub reset_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeUsageSnapshot {
    pub managed_account_id: String,
    pub session: Option<ClaudeRateWindowSnapshot>,
    pub weekly: Option<ClaudeRateWindowSnapshot>,
    pub model_weekly_label: Option<String>,
    pub model_weekly: Option<ClaudeRateWindowSnapshot>,
    pub last_synced_at: Option<String>,
    pub last_sync_error: Option<String>,
    pub needs_relogin: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClaudeUsageSnapshotIndex {
    pub version: u8,
    pub snapshots: Vec<ClaudeUsageSnapshot>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedClaudeUsage {
    pub session: Option<ClaudeRateWindowSnapshot>,
    pub weekly: Option<ClaudeRateWindowSnapshot>,
    pub model_weekly_label: Option<String>,
    pub model_weekly: Option<ClaudeRateWindowSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeUsageApiResponse {
    #[serde(rename = "five_hour")]
    pub five_hour: Option<ClaudeUsageApiWindow>,
    #[serde(rename = "seven_day")]
    pub seven_day: Option<ClaudeUsageApiWindow>,
    #[serde(rename = "seven_day_oauth_apps")]
    pub seven_day_oauth_apps: Option<ClaudeUsageApiWindow>,
    #[serde(rename = "seven_day_opus")]
    pub seven_day_opus: Option<ClaudeUsageApiWindow>,
    #[serde(rename = "seven_day_sonnet")]
    pub seven_day_sonnet: Option<ClaudeUsageApiWindow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ClaudeUsageApiWindow {
    pub utilization: Option<f64>,
    #[serde(rename = "resets_at", alias = "reset_at")]
    pub resets_at: Option<String>,
}
