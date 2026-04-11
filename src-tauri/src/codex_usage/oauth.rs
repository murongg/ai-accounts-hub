use std::fmt::{Display, Formatter};
use std::path::Path;

use crate::codex_accounts::auth::{format_plan, read_auth_value};
use crate::codex_accounts::store::auth_path_for_home;
use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

use super::models::{
    CodexUsageApiResponse, CodexUsageApiWindow, CodexUsageRateLimit, FetchedCodexUsage,
    RateWindowSnapshot,
};

pub trait CodexUsageFetcher: Send + Sync {
    fn fetch_usage(&self, managed_home: &Path) -> Result<FetchedCodexUsage, CodexUsageFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CodexUsageFetchError {
    Unauthorized,
    MissingCredentials(String),
    InvalidResponse(String),
    RequestFailed(String),
}

impl CodexUsageFetchError {
    pub fn needs_relogin(&self) -> bool {
        matches!(self, Self::Unauthorized)
    }
}

impl Display for CodexUsageFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => {
                write!(
                    f,
                    "Codex OAuth token expired or invalid. Run `codex login` again."
                )
            }
            Self::MissingCredentials(message)
            | Self::InvalidResponse(message)
            | Self::RequestFailed(message) => f.write_str(message),
        }
    }
}

pub struct ProcessCodexUsageFetcher;

impl CodexUsageFetcher for ProcessCodexUsageFetcher {
    fn fetch_usage(&self, managed_home: &Path) -> Result<FetchedCodexUsage, CodexUsageFetchError> {
        let auth = read_auth_value(&auth_path_for_home(managed_home))
            .map_err(CodexUsageFetchError::MissingCredentials)?;
        let tokens = auth
            .get("tokens")
            .and_then(|value| value.as_object())
            .ok_or_else(|| {
                CodexUsageFetchError::MissingCredentials("auth.json is missing tokens".into())
            })?;
        let access_token = tokens
            .get("access_token")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                CodexUsageFetchError::MissingCredentials(
                    "auth.json is missing tokens.access_token".into(),
                )
            })?;
        let account_id = tokens
            .get("account_id")
            .and_then(|value| value.as_str())
            .map(str::trim)
            .filter(|value| !value.is_empty());

        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|error| CodexUsageFetchError::RequestFailed(error.to_string()))?,
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("codex-cli"));
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        if let Some(account_id) = account_id {
            headers.insert(
                "ChatGPT-Account-Id",
                HeaderValue::from_str(account_id)
                    .map_err(|error| CodexUsageFetchError::RequestFailed(error.to_string()))?,
            );
        }

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .map_err(|error| CodexUsageFetchError::RequestFailed(error.to_string()))?;
        let response = client
            .get(resolve_usage_url(managed_home))
            .send()
            .map_err(|error| CodexUsageFetchError::RequestFailed(error.to_string()))?;
        let status = response.status();
        if status.as_u16() == 401 || status.as_u16() == 403 {
            return Err(CodexUsageFetchError::Unauthorized);
        }
        if !status.is_success() {
            return Err(CodexUsageFetchError::RequestFailed(format!(
                "GET {} failed: {}",
                resolve_usage_url(managed_home),
                status.as_u16()
            )));
        }

        let parsed: CodexUsageApiResponse = response
            .json()
            .map_err(|error| CodexUsageFetchError::InvalidResponse(error.to_string()))?;
        Ok(normalize_usage_response(parsed))
    }
}

pub fn normalize_usage_response(response: CodexUsageApiResponse) -> FetchedCodexUsage {
    let (five_hour, weekly) = normalize_rate_limit(response.rate_limit);

    FetchedCodexUsage {
        plan: response.plan_type.as_deref().map(format_plan),
        five_hour,
        weekly,
        credits_balance: response.credits.and_then(|credits| {
            if credits.unlimited {
                None
            } else {
                credits.balance
            }
        }),
    }
}

fn normalize_rate_limit(
    rate_limit: Option<CodexUsageRateLimit>,
) -> (Option<RateWindowSnapshot>, Option<RateWindowSnapshot>) {
    let Some(rate_limit) = rate_limit else {
        return (None, None);
    };

    let mut five_hour = None;
    let mut weekly = None;

    for window in [rate_limit.primary_window, rate_limit.secondary_window]
        .into_iter()
        .flatten()
    {
        let (snapshot, role) = map_window(window);
        match role {
            WindowRole::FiveHour => {
                if five_hour.is_none() {
                    five_hour = Some(snapshot);
                } else if weekly.is_none() {
                    weekly = Some(snapshot);
                }
            }
            WindowRole::Weekly => {
                if weekly.is_none() {
                    weekly = Some(snapshot);
                } else if five_hour.is_none() {
                    five_hour = Some(snapshot);
                }
            }
            WindowRole::Unknown => {
                if five_hour.is_none() {
                    five_hour = Some(snapshot);
                } else if weekly.is_none() {
                    weekly = Some(snapshot);
                }
            }
        }
    }

    (five_hour, weekly)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowRole {
    FiveHour,
    Weekly,
    Unknown,
}

fn map_window(window: CodexUsageApiWindow) -> (RateWindowSnapshot, WindowRole) {
    let used_percent = window.used_percent.min(100);
    let role = match window.limit_window_seconds {
        18_000 => WindowRole::FiveHour,
        604_800 => WindowRole::Weekly,
        _ => WindowRole::Unknown,
    };

    (
        RateWindowSnapshot {
            remaining_percent: 100_u8.saturating_sub(used_percent),
            used_percent,
            reset_at: window.reset_at.to_string(),
        },
        role,
    )
}

fn resolve_usage_url(managed_home: &Path) -> String {
    let config_path = managed_home.join("config.toml");
    let config_contents = std::fs::read_to_string(config_path).ok();
    let base = config_contents
        .as_deref()
        .and_then(parse_chatgpt_base_url)
        .unwrap_or_else(|| "https://chatgpt.com/backend-api".to_string());

    let normalized = normalize_chatgpt_base_url(&base);
    if normalized.contains("/backend-api") {
        format!("{normalized}/wham/usage")
    } else {
        format!("{normalized}/api/codex/usage")
    }
}

fn parse_chatgpt_base_url(contents: &str) -> Option<String> {
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

fn normalize_chatgpt_base_url(base: &str) -> String {
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
