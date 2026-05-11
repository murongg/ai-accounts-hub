use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AppLanguage {
    #[serde(rename = "zh-CN")]
    ZhCn,
    #[serde(rename = "en-US")]
    EnUs,
}

impl Default for AppLanguage {
    fn default() -> Self {
        Self::ZhCn
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppTheme {
    Light,
    Dark,
    System,
}

impl Default for AppTheme {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppAccountsViewMode {
    Cards,
    List,
    Mini,
}

impl Default for AppAccountsViewMode {
    fn default() -> Self {
        Self::Cards
    }
}

fn default_auto_switch_enabled() -> bool {
    true
}

fn default_auto_switch_threshold_percent() -> u8 {
    0
}

fn default_relay_port() -> u16 {
    8765
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RelaySettings {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_relay_port")]
    pub port: u16,
}

impl Default for RelaySettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_relay_port(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    pub language: AppLanguage,
    pub theme: AppTheme,
    #[serde(default = "default_auto_switch_enabled")]
    pub auto_switch_enabled: bool,
    #[serde(default = "default_auto_switch_threshold_percent")]
    pub auto_switch_five_hour_threshold_percent: u8,
    #[serde(
        default = "default_auto_switch_threshold_percent",
        rename = "auto_switch_weekly_threshold_percent",
        alias = "auto_switch_one_hour_threshold_percent"
    )]
    pub auto_switch_weekly_threshold_percent: u8,
    #[serde(default)]
    pub accounts_view_mode: AppAccountsViewMode,
    #[serde(default)]
    pub email_privacy_enabled: bool,
    #[serde(default)]
    pub relay: RelaySettings,
}

impl AppSettings {
    pub fn sanitized(mut self) -> Self {
        self.auto_switch_five_hour_threshold_percent =
            self.auto_switch_five_hour_threshold_percent.min(99);
        self.auto_switch_weekly_threshold_percent =
            self.auto_switch_weekly_threshold_percent.min(99);
        self
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: AppLanguage::default(),
            theme: AppTheme::default(),
            auto_switch_enabled: default_auto_switch_enabled(),
            auto_switch_five_hour_threshold_percent: default_auto_switch_threshold_percent(),
            auto_switch_weekly_threshold_percent: default_auto_switch_threshold_percent(),
            accounts_view_mode: AppAccountsViewMode::default(),
            email_privacy_enabled: false,
            relay: RelaySettings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppDataDirectoryInfo {
    pub current_dir: String,
    pub default_dir: String,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClearAllDataResult {
    pub app_settings: AppSettings,
    pub refresh_settings: crate::codex_usage::models::CodexRefreshSettings,
    pub data_directory: AppDataDirectoryInfo,
}
