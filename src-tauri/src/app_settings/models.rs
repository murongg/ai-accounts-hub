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

fn default_auto_switch_enabled() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppSettings {
    pub language: AppLanguage,
    pub theme: AppTheme,
    #[serde(default = "default_auto_switch_enabled")]
    pub auto_switch_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: AppLanguage::default(),
            theme: AppTheme::default(),
            auto_switch_enabled: default_auto_switch_enabled(),
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
