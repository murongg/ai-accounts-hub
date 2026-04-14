use std::fmt::{Display, Formatter};

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::Deserialize;
use serde_json::Value;

use crate::claude_accounts::live_credentials::ClaudeLiveCredentialSnapshot;

use super::models::{
    ClaudeRateWindowSnapshot, ClaudeUsageApiResponse, ClaudeUsageApiWindow, FetchedClaudeUsage,
};

const OAUTH_USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";
const OAUTH_REFRESH_URL: &str = "https://platform.claude.com/v1/oauth/token";
const OAUTH_BETA_HEADER: &str = "oauth-2025-04-20";
const OAUTH_CLIENT_ID: &str = "9d1c250a-e61b-44d9-88ed-5944d1962f5e";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaudeOAuthCredentials {
    pub access_token: String,
    pub refresh_token: Option<String>,
}

pub trait ClaudeOAuthHttpClient: Send + Sync {
    fn get_usage(
        &self,
        access_token: &str,
    ) -> Result<ClaudeUsageApiResponse, ClaudeUsageFetchError>;
    fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<ClaudeOAuthCredentials, ClaudeUsageFetchError>;
}

pub trait ClaudeUsageFetcher: Send + Sync {
    fn fetch_usage(
        &self,
        snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeUsageFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaudeUsageFetchError {
    Unauthorized,
    RefreshRejected(String),
    MissingCredentials(String),
    InvalidResponse(String),
    RequestFailed(String),
}

impl ClaudeUsageFetchError {
    pub fn needs_relogin(&self) -> bool {
        matches!(self, Self::Unauthorized | Self::RefreshRejected(_))
    }
}

impl Display for ClaudeUsageFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized => {
                write!(f, "Claude OAuth request unauthorized. Run `claude` again.")
            }
            Self::RefreshRejected(message)
            | Self::MissingCredentials(message)
            | Self::InvalidResponse(message)
            | Self::RequestFailed(message) => f.write_str(message),
        }
    }
}

#[derive(Debug, Deserialize)]
struct RawStoredOAuthEnvelope {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: Option<RawStoredOAuthCredentials>,
}

#[derive(Debug, Deserialize)]
struct RawStoredOAuthCredentials {
    #[serde(rename = "accessToken", alias = "access_token")]
    access_token: Option<String>,
    #[serde(rename = "refreshToken", alias = "refresh_token")]
    refresh_token: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RawRefreshTokenResponse {
    #[serde(rename = "access_token")]
    access_token: String,
    #[serde(rename = "refresh_token")]
    refresh_token: Option<String>,
}

pub struct ProcessClaudeOAuthHttpClient {
    client: Client,
}

impl Default for ProcessClaudeOAuthHttpClient {
    fn default() -> Self {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(USER_AGENT, HeaderValue::from_static("claude-code/2.1.0"));

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .default_headers(headers)
            .build()
            .expect("Claude OAuth HTTP client should build");

        Self { client }
    }
}

impl ClaudeOAuthHttpClient for ProcessClaudeOAuthHttpClient {
    fn get_usage(
        &self,
        access_token: &str,
    ) -> Result<ClaudeUsageApiResponse, ClaudeUsageFetchError> {
        let response = self
            .client
            .get(OAUTH_USAGE_URL)
            .header(AUTHORIZATION, format!("Bearer {access_token}"))
            .header("anthropic-beta", OAUTH_BETA_HEADER)
            .send()
            .map_err(|error| ClaudeUsageFetchError::RequestFailed(error.to_string()))?;
        let status = response.status();

        if should_require_relogin_for_oauth_status(status.as_u16()) {
            return Err(ClaudeUsageFetchError::Unauthorized);
        }
        if !status.is_success() {
            return Err(ClaudeUsageFetchError::RequestFailed(format!(
                "GET {OAUTH_USAGE_URL} failed: {}",
                status.as_u16()
            )));
        }

        response
            .json()
            .map_err(|error| ClaudeUsageFetchError::InvalidResponse(error.to_string()))
    }

    fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<ClaudeOAuthCredentials, ClaudeUsageFetchError> {
        let response = self
            .client
            .post(OAUTH_REFRESH_URL)
            .header(CONTENT_TYPE, "application/x-www-form-urlencoded")
            .body(format!(
                "grant_type=refresh_token&refresh_token={refresh_token}&client_id={OAUTH_CLIENT_ID}"
            ))
            .send()
            .map_err(|error| ClaudeUsageFetchError::RequestFailed(error.to_string()))?;
        let status = response.status();

        if status.as_u16() == 400 || status.as_u16() == 401 {
            let body = response.text().unwrap_or_default();
            if body.to_ascii_lowercase().contains("invalid_grant") || status.as_u16() == 401 {
                return Err(ClaudeUsageFetchError::RefreshRejected(format!(
                    "Claude OAuth refresh rejected. Run `claude` again."
                )));
            }
            return Err(ClaudeUsageFetchError::RequestFailed(format!(
                "POST {OAUTH_REFRESH_URL} failed: {}",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(ClaudeUsageFetchError::RequestFailed(format!(
                "POST {OAUTH_REFRESH_URL} failed: {}",
                status.as_u16()
            )));
        }

        let parsed: RawRefreshTokenResponse = response
            .json()
            .map_err(|error| ClaudeUsageFetchError::InvalidResponse(error.to_string()))?;

        Ok(ClaudeOAuthCredentials {
            access_token: parsed.access_token,
            refresh_token: parsed
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())),
        })
    }
}

pub struct OAuthClaudeUsageFetcher<C: ClaudeOAuthHttpClient> {
    client: C,
}

impl<C: ClaudeOAuthHttpClient> OAuthClaudeUsageFetcher<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn client(&self) -> &C {
        &self.client
    }
}

impl<C: ClaudeOAuthHttpClient> ClaudeUsageFetcher for OAuthClaudeUsageFetcher<C> {
    fn fetch_usage(
        &self,
        snapshot: &ClaudeLiveCredentialSnapshot,
    ) -> Result<FetchedClaudeUsage, ClaudeUsageFetchError> {
        let credentials = extract_oauth_credentials(snapshot)?;

        match self.client.get_usage(&credentials.access_token) {
            Ok(response) => normalize_usage_response(response),
            Err(ClaudeUsageFetchError::Unauthorized) => {
                let refresh_token = credentials.refresh_token.as_deref().ok_or_else(|| {
                    ClaudeUsageFetchError::RefreshRejected(
                        "Claude OAuth access token expired and refresh token is unavailable. Run `claude` again."
                            .to_string(),
                    )
                })?;

                let refreshed = self.client.refresh_access_token(refresh_token)?;
                let response = self.client.get_usage(&refreshed.access_token)?;
                normalize_usage_response(response)
            }
            Err(other) => Err(other),
        }
    }
}

pub fn extract_oauth_credentials(
    snapshot: &ClaudeLiveCredentialSnapshot,
) -> Result<ClaudeOAuthCredentials, ClaudeUsageFetchError> {
    let envelope: RawStoredOAuthEnvelope = serde_json::from_slice(&snapshot.credentials_json)
        .map_err(|error| {
            ClaudeUsageFetchError::MissingCredentials(format!(
                "failed to parse Claude secure storage payload: {error}"
            ))
        })?;
    let Some(oauth) = envelope.claude_ai_oauth else {
        return Err(ClaudeUsageFetchError::MissingCredentials(
            "Claude secure storage payload is missing claudeAiOauth".to_string(),
        ));
    };
    let access_token = oauth
        .access_token
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            ClaudeUsageFetchError::MissingCredentials(
                "Claude secure storage payload is missing claudeAiOauth.accessToken".to_string(),
            )
        })?;

    Ok(ClaudeOAuthCredentials {
        access_token,
        refresh_token: oauth
            .refresh_token
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
    })
}

pub fn normalize_usage_response(
    response: ClaudeUsageApiResponse,
) -> Result<FetchedClaudeUsage, ClaudeUsageFetchError> {
    let session = response.five_hour.and_then(window_from_api);
    let weekly = response.seven_day.and_then(window_from_api);
    let (model_weekly_label, model_weekly) =
        if let Some(window) = response.seven_day_opus.and_then(window_from_api) {
            (Some("Opus Weekly".to_string()), Some(window))
        } else if let Some(window) = response.seven_day_sonnet.and_then(window_from_api) {
            (Some("Sonnet Weekly".to_string()), Some(window))
        } else {
            (None, None)
        };

    if session.is_none() && weekly.is_none() && model_weekly.is_none() {
        return Err(ClaudeUsageFetchError::InvalidResponse(
            "Claude OAuth response did not contain any usage windows".to_string(),
        ));
    }

    Ok(FetchedClaudeUsage {
        session,
        weekly,
        model_weekly_label,
        model_weekly,
    })
}

pub fn should_require_relogin_for_oauth_status(status: u16) -> bool {
    status == 401
}

fn window_from_api(window: ClaudeUsageApiWindow) -> Option<ClaudeRateWindowSnapshot> {
    let utilization = window.utilization?;
    let reset_at = window.resets_at?;
    let normalized_percent = if utilization <= 1.0 {
        utilization * 100.0
    } else {
        utilization
    };
    let used_percent = normalized_percent.round().clamp(0.0, 100.0) as u8;

    Some(ClaudeRateWindowSnapshot {
        remaining_percent: 100_u8.saturating_sub(used_percent),
        used_percent,
        reset_at,
    })
}

pub fn oauth_account_email(snapshot: &ClaudeLiveCredentialSnapshot) -> Option<String> {
    let raw = snapshot.oauth_account_json.as_deref()?;
    let json: Value = serde_json::from_slice(raw).ok()?;
    json.get("emailAddress")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}
