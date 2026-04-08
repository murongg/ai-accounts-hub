use std::fmt::{Display, Formatter};
use std::fs;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use reqwest::blocking::Client;
use reqwest::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use reqwest::blocking::RequestBuilder;
use serde_json::{json, Value};

use crate::codex_usage::models::RateWindowSnapshot;
use crate::gemini_accounts::cli::resolve_gemini_binary;
use crate::gemini_accounts::paths::{atomic_write, gemini_dir_for_home};

use super::models::{FetchedGeminiUsage, GeminiTierId, GeminiUsageApiBucket, GeminiUsageApiResponse};

const LOAD_CODE_ASSIST_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:loadCodeAssist";
const QUOTA_URL: &str = "https://cloudcode-pa.googleapis.com/v1internal:retrieveUserQuota";
const PROJECTS_URL: &str = "https://cloudresourcemanager.googleapis.com/v1/projects";
const TOKEN_REFRESH_URL: &str = "https://oauth2.googleapis.com/token";

pub trait GeminiUsageFetcher: Send + Sync {
    fn fetch_usage(&self, managed_home: &Path) -> Result<FetchedGeminiUsage, GeminiUsageFetchError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeminiUsageFetchError {
    Unauthorized(String),
    UnsupportedAuthType(String),
    MissingCredentials(String),
    InvalidResponse(String),
    RequestFailed(String),
}

impl GeminiUsageFetchError {
    pub fn unauthorized(reason: impl Into<String>) -> Self {
        Self::Unauthorized(reason.into())
    }

    pub fn needs_relogin(&self) -> bool {
        matches!(self, Self::Unauthorized(_))
    }
}

impl Display for GeminiUsageFetchError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unauthorized(reason) => {
                write!(
                    f,
                    "Gemini OAuth token expired or invalid ({reason}). Run `gemini` again."
                )
            }
            Self::UnsupportedAuthType(auth_type) => {
                write!(f, "Gemini {auth_type} auth is not supported. Sign in with Google instead.")
            }
            Self::MissingCredentials(message)
            | Self::InvalidResponse(message)
            | Self::RequestFailed(message) => f.write_str(message),
        }
    }
}

pub struct ProcessGeminiUsageFetcher;

impl GeminiUsageFetcher for ProcessGeminiUsageFetcher {
    fn fetch_usage(&self, managed_home: &Path) -> Result<FetchedGeminiUsage, GeminiUsageFetchError> {
        let gemini_dir = gemini_dir_for_home(managed_home);
        let settings = read_optional_json_value(&gemini_dir.join("settings.json"))
            .map_err(GeminiUsageFetchError::MissingCredentials)?;
        validate_auth_type(settings.as_ref())?;

        let mut oauth_creds = read_json_value(&gemini_dir.join("oauth_creds.json"))
            .map_err(GeminiUsageFetchError::MissingCredentials)?;
        let mut access_token = oauth_creds
            .get("access_token")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                GeminiUsageFetchError::MissingCredentials(
                    "oauth_creds.json is missing access_token".to_string(),
                )
            })?
            .to_string();

        if token_is_expired(&oauth_creds) {
            let refresh_token = oauth_creds
                .get("refresh_token")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| GeminiUsageFetchError::unauthorized("access token expired and refresh_token is missing"))?;
            let refreshed = refresh_access_token(refresh_token, &gemini_dir)?;
            update_stored_credentials(&gemini_dir.join("oauth_creds.json"), &mut oauth_creds, &refreshed)
                .map_err(GeminiUsageFetchError::RequestFailed)?;
            access_token = refreshed.access_token.clone();
            if let Some(id_token) = refreshed.id_token {
                oauth_creds["id_token"] = Value::String(id_token);
            }
        }

        let claims = extract_claims_from_token(
            oauth_creds
                .get("id_token")
                .and_then(Value::as_str),
        );
        let client = build_http_client()?;
        let code_assist = load_code_assist_status(&client, &access_token)?;
        let project_id = code_assist
            .project_id
            .or_else(|| discover_gemini_project_id(&client, &access_token).ok().flatten());

        let response = quota_request(&client, &access_token, project_id.as_deref())
            .send()
            .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))?;

        let status = response.status();
        if should_require_relogin_for_quota_status(status.as_u16()) {
            return Err(GeminiUsageFetchError::unauthorized(format!(
                "quota endpoint returned {}",
                status.as_u16()
            )));
        }
        if !status.is_success() {
            return Err(GeminiUsageFetchError::RequestFailed(format!(
                "POST {QUOTA_URL} failed: {}",
                status.as_u16()
            )));
        }

        let parsed: GeminiUsageApiResponse = response
            .json()
            .map_err(|error| GeminiUsageFetchError::InvalidResponse(error.to_string()))?;
        normalize_usage_response(parsed, code_assist.tier, claims.hosted_domain.as_deref())
            .map_err(GeminiUsageFetchError::InvalidResponse)
    }
}

fn quota_request<'a>(
    client: &'a Client,
    access_token: &'a str,
    project_id: Option<&'a str>,
) -> RequestBuilder {
    client
        .post(QUOTA_URL)
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .json(&match project_id {
            Some(project) => json!({ "project": project }),
            None => json!({}),
        })
}

pub fn normalize_usage_response(
    response: GeminiUsageApiResponse,
    tier: Option<GeminiTierId>,
    hosted_domain: Option<&str>,
) -> Result<FetchedGeminiUsage, String> {
    let buckets = response
        .buckets
        .filter(|buckets| !buckets.is_empty())
        .ok_or_else(|| "No quota buckets in response".to_string())?;

    let mut pro_bucket: Option<GeminiUsageApiBucket> = None;
    let mut flash_bucket: Option<GeminiUsageApiBucket> = None;
    let mut flash_lite_bucket: Option<GeminiUsageApiBucket> = None;

    for bucket in buckets {
        let Some(model_id) = bucket.model_id.as_deref().map(str::to_ascii_lowercase) else {
            continue;
        };
        let Some(_remaining_fraction) = bucket.remaining_fraction else {
            continue;
        };
        if model_id.contains("flash-lite") {
            if lower_fraction_than(
                bucket.remaining_fraction,
                flash_lite_bucket.as_ref().and_then(|b| b.remaining_fraction),
            ) {
                flash_lite_bucket = Some(bucket.clone());
            }
            continue;
        }
        if model_id.contains("pro") {
            if lower_fraction_than(bucket.remaining_fraction, pro_bucket.as_ref().and_then(|b| b.remaining_fraction)) {
                pro_bucket = Some(bucket.clone());
            }
            continue;
        }
        if model_id.contains("flash") {
            if lower_fraction_than(bucket.remaining_fraction, flash_bucket.as_ref().and_then(|b| b.remaining_fraction)) {
                flash_bucket = Some(bucket.clone());
            }
        }
    }

    Ok(FetchedGeminiUsage {
        plan: map_plan(tier, hosted_domain),
        pro: pro_bucket.and_then(map_bucket_to_window),
        flash: flash_bucket.and_then(map_bucket_to_window),
        flash_lite: flash_lite_bucket.and_then(map_bucket_to_window),
    })
}

fn lower_fraction_than(candidate: Option<f64>, current: Option<f64>) -> bool {
    match (candidate, current) {
        (Some(candidate), Some(current)) => candidate < current,
        (Some(_), None) => true,
        _ => false,
    }
}

fn map_bucket_to_window(bucket: GeminiUsageApiBucket) -> Option<RateWindowSnapshot> {
    let remaining_fraction = bucket.remaining_fraction?;
    let remaining_percent = (remaining_fraction * 100.0).round().clamp(0.0, 100.0) as u8;
    Some(RateWindowSnapshot {
        remaining_percent,
        used_percent: 100_u8.saturating_sub(remaining_percent),
        reset_at: bucket.reset_time.unwrap_or_default(),
    })
}

fn map_plan(tier: Option<GeminiTierId>, hosted_domain: Option<&str>) -> Option<String> {
    match (tier, hosted_domain.filter(|value| !value.trim().is_empty())) {
        (Some(GeminiTierId::Standard), _) => Some("Paid".to_string()),
        (Some(GeminiTierId::Free), Some(_)) => Some("Workspace".to_string()),
        (Some(GeminiTierId::Free), None) => Some("Free".to_string()),
        (Some(GeminiTierId::Legacy), _) => Some("Legacy".to_string()),
        (None, _) => None,
    }
}

fn validate_auth_type(settings: Option<&Value>) -> Result<(), GeminiUsageFetchError> {
    let auth_type = settings
        .and_then(|value| value.get("security"))
        .and_then(|value| value.get("auth"))
        .and_then(|value| value.get("selectedType"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty());

    match auth_type {
        Some("api-key") => Err(GeminiUsageFetchError::UnsupportedAuthType("API key".to_string())),
        Some("vertex-ai") => Err(GeminiUsageFetchError::UnsupportedAuthType("Vertex AI".to_string())),
        _ => Ok(()),
    }
}

fn build_http_client() -> Result<Client, GeminiUsageFetchError> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static("gemini-cli"));
    headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    Client::builder()
        .timeout(Duration::from_secs(30))
        .default_headers(headers)
        .build()
        .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))
}

struct CodeAssistStatus {
    tier: Option<GeminiTierId>,
    project_id: Option<String>,
}

fn load_code_assist_status(
    client: &Client,
    access_token: &str,
) -> Result<CodeAssistStatus, GeminiUsageFetchError> {
    let response = client
        .post(LOAD_CODE_ASSIST_URL)
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .json(&json!({
            "metadata": {
                "ideType": "GEMINI_CLI",
                "pluginType": "GEMINI"
            }
        }))
        .send()
        .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))?;

    if !response.status().is_success() {
        return Ok(CodeAssistStatus {
            tier: None,
            project_id: None,
        });
    }

    let json: Value = match response.json() {
        Ok(json) => json,
        Err(_) => {
            return Ok(CodeAssistStatus {
                tier: None,
                project_id: None,
            });
        }
    };

    let project_id = json
        .get("cloudaicompanionProject")
        .and_then(|value| {
            value
                .as_str()
                .map(str::to_string)
                .or_else(|| value.get("id").and_then(Value::as_str).map(str::to_string))
                .or_else(|| value.get("projectId").and_then(Value::as_str).map(str::to_string))
        })
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    let tier = json
        .get("currentTier")
        .and_then(|value| value.get("id"))
        .and_then(Value::as_str)
        .and_then(GeminiTierId::from_api_id);

    Ok(CodeAssistStatus { tier, project_id })
}

pub fn should_require_relogin_for_quota_status(status_code: u16) -> bool {
    status_code == 401
}

fn discover_gemini_project_id(
    client: &Client,
    access_token: &str,
) -> Result<Option<String>, GeminiUsageFetchError> {
    let response = client
        .get(PROJECTS_URL)
        .header(AUTHORIZATION, format!("Bearer {access_token}"))
        .send()
        .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let json: Value = response
        .json()
        .map_err(|error| GeminiUsageFetchError::InvalidResponse(error.to_string()))?;
    let Some(projects) = json.get("projects").and_then(Value::as_array) else {
        return Ok(None);
    };

    for project in projects {
        let project_id = project
            .get("projectId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let Some(project_id) = project_id else {
            continue;
        };

        if project_id.starts_with("gen-lang-client") {
            return Ok(Some(project_id.to_string()));
        }

        if project
            .get("labels")
            .and_then(Value::as_object)
            .is_some_and(|labels| labels.contains_key("generative-language"))
        {
            return Ok(Some(project_id.to_string()));
        }
    }

    Ok(None)
}

struct RefreshedTokens {
    access_token: String,
    expires_in_seconds: Option<u64>,
    id_token: Option<String>,
}

fn refresh_access_token(refresh_token: &str, gemini_dir: &Path) -> Result<RefreshedTokens, GeminiUsageFetchError> {
    let credentials = extract_oauth_client_credentials()?;
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))?;
    let response = client
        .post(TOKEN_REFRESH_URL)
        .form(&[
            ("client_id", credentials.client_id.as_str()),
            ("client_secret", credentials.client_secret.as_str()),
            ("refresh_token", refresh_token),
            ("grant_type", "refresh_token"),
        ])
        .send()
        .map_err(|error| GeminiUsageFetchError::RequestFailed(error.to_string()))?;

    let status = response.status();
    if status.as_u16() == 400 || status.as_u16() == 401 || status.as_u16() == 403 {
        return Err(GeminiUsageFetchError::unauthorized(format!(
            "token refresh returned {}",
            status.as_u16()
        )));
    }
    if !status.is_success() {
        return Err(GeminiUsageFetchError::RequestFailed(format!(
            "POST {TOKEN_REFRESH_URL} failed: {}",
            status.as_u16()
        )));
    }

    let json: Value = response
        .json()
        .map_err(|error| GeminiUsageFetchError::InvalidResponse(error.to_string()))?;
    let access_token = json
        .get("access_token")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| GeminiUsageFetchError::InvalidResponse("missing refresh access token".into()))?
        .to_string();
    let expires_in_seconds = json.get("expires_in").and_then(Value::as_u64);
    let id_token = json.get("id_token").and_then(Value::as_str).map(str::to_string);

    let _ = gemini_dir;
    Ok(RefreshedTokens {
        access_token,
        expires_in_seconds,
        id_token,
    })
}

struct OAuthClientCredentials {
    client_id: String,
    client_secret: String,
}

fn extract_oauth_client_credentials() -> Result<OAuthClientCredentials, GeminiUsageFetchError> {
    let binary = resolve_gemini_binary().ok_or_else(|| {
        GeminiUsageFetchError::RequestFailed("failed to resolve gemini binary".to_string())
    })?;
    let resolved_binary = fs::canonicalize(&binary).unwrap_or(binary);
    let bin_dir = resolved_binary
        .parent()
        .ok_or_else(|| GeminiUsageFetchError::RequestFailed("invalid gemini binary path".to_string()))?;
    let base_dir = bin_dir
        .parent()
        .ok_or_else(|| GeminiUsageFetchError::RequestFailed("invalid gemini base path".to_string()))?;
    let possible_paths = [
        base_dir.join("libexec/lib/node_modules/@google/gemini-cli/node_modules/@google/gemini-cli-core/dist/src/code_assist/oauth2.js"),
        base_dir.join("lib/node_modules/@google/gemini-cli/node_modules/@google/gemini-cli-core/dist/src/code_assist/oauth2.js"),
        base_dir.join("share/gemini-cli/node_modules/@google/gemini-cli-core/dist/src/code_assist/oauth2.js"),
        bin_dir.join("../gemini-cli-core/dist/src/code_assist/oauth2.js"),
        base_dir.join("node_modules/@google/gemini-cli-core/dist/src/code_assist/oauth2.js"),
    ];

    for path in possible_paths {
        if let Ok(contents) = fs::read_to_string(&path) {
            if let (Some(client_id), Some(client_secret)) = (
                extract_assignment(&contents, "OAUTH_CLIENT_ID"),
                extract_assignment(&contents, "OAUTH_CLIENT_SECRET"),
            ) {
                return Ok(OAuthClientCredentials {
                    client_id,
                    client_secret,
                });
            }
        }
    }

    Err(GeminiUsageFetchError::RequestFailed(
        "Could not find Gemini CLI OAuth configuration".to_string(),
    ))
}

fn extract_assignment(contents: &str, key: &str) -> Option<String> {
    contents.lines().find_map(|line| {
        if !line.contains(key) || !line.contains('=') {
            return None;
        }
        let (_, value) = line.split_once('=')?;
        extract_quoted_value(value)
    })
}

fn extract_quoted_value(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let quote = if trimmed.contains('\'') { '\'' } else { '"' };
    let start = trimmed.find(quote)?;
    let remainder = &trimmed[start + 1..];
    let end = remainder.find(quote)?;
    Some(remainder[..end].to_string())
}

fn update_stored_credentials(
    oauth_path: &Path,
    oauth_creds: &mut Value,
    refreshed: &RefreshedTokens,
) -> Result<(), String> {
    oauth_creds["access_token"] = Value::String(refreshed.access_token.clone());
    if let Some(id_token) = refreshed.id_token.clone() {
        oauth_creds["id_token"] = Value::String(id_token);
    }
    if let Some(expires_in) = refreshed.expires_in_seconds {
        let expiry_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis() as u64 + expires_in * 1000)
            .unwrap_or(expires_in * 1000);
        oauth_creds["expiry_date"] = Value::Number(serde_json::Number::from(expiry_ms));
    }
    let bytes = serde_json::to_vec_pretty(oauth_creds)
        .map_err(|error| format!("failed to serialize refreshed Gemini creds: {error}"))?;
    atomic_write(oauth_path, &bytes)
}

fn token_is_expired(oauth_creds: &Value) -> bool {
    let expiry_date = oauth_creds.get("expiry_date").and_then(parse_expiry_ms);
    match expiry_date {
        Some(expiry_ms) => {
            let now_ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|duration| duration.as_millis() as u64)
                .unwrap_or(0);
            expiry_ms <= now_ms
        }
        None => false,
    }
}

fn parse_expiry_ms(value: &Value) -> Option<u64> {
    value
        .as_u64()
        .or_else(|| value.as_i64().and_then(|v| u64::try_from(v).ok()))
        .or_else(|| value.as_f64().map(|v| v.max(0.0) as u64))
        .or_else(|| value.as_str().and_then(|v| v.trim().parse::<u64>().ok()))
}

struct TokenClaims {
    hosted_domain: Option<String>,
}

fn extract_claims_from_token(id_token: Option<&str>) -> TokenClaims {
    let Some(token) = id_token else {
        return TokenClaims { hosted_domain: None };
    };
    let Some(payload) = token.split('.').nth(1) else {
        return TokenClaims { hosted_domain: None };
    };
    let decoded = URL_SAFE_NO_PAD.decode(payload);
    let Ok(decoded) = decoded else {
        return TokenClaims { hosted_domain: None };
    };
    let Ok(json) = serde_json::from_slice::<Value>(&decoded) else {
        return TokenClaims { hosted_domain: None };
    };
    TokenClaims {
        hosted_domain: json
            .get("hd")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
    }
}

fn read_json_value(path: &Path) -> Result<Value, String> {
    let bytes = fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn read_optional_json_value(path: &Path) -> Result<Option<Value>, String> {
    match fs::read(path) {
        Ok(bytes) => serde_json::from_slice(&bytes)
            .map(Some)
            .map_err(|error| format!("failed to parse {}: {error}", path.display())),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("failed to read {}: {error}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use super::quota_request;
    use reqwest::blocking::Client;
    use reqwest::header::AUTHORIZATION;

    #[test]
    fn quota_request_includes_bearer_authorization_header() {
        let client = Client::new();
        let request = quota_request(&client, "access-token", Some("project-123"))
            .build()
            .expect("request");

        assert_eq!(
            request.headers().get(AUTHORIZATION).and_then(|value| value.to_str().ok()),
            Some("Bearer access-token")
        );
    }
}
