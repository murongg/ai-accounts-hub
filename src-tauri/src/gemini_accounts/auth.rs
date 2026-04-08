use std::fs;
use std::path::Path;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::Value;

use super::models::{GeminiAccountIdentity, StoredGeminiAccount};

pub fn extract_account_identity(
    oauth_creds: &Value,
    settings: Option<&Value>,
) -> Result<GeminiAccountIdentity, String> {
    let id_token = oauth_creds
        .get("id_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "oauth_creds.json is missing id_token".to_string())?;
    let claims = decode_id_token_claims(id_token)?;
    let email = claims
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "oauth_creds.json is missing a usable email".to_string())?
        .to_lowercase();
    let subject = claims
        .get("sub")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let auth_type = settings
        .and_then(|value| value.get("security"))
        .and_then(|value| value.get("auth"))
        .and_then(|value| value.get("selectedType"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    Ok(GeminiAccountIdentity {
        email,
        subject,
        auth_type,
    })
}

pub fn read_account_identity_from_dir(gemini_dir: &Path) -> Result<GeminiAccountIdentity, String> {
    let oauth_creds = read_json_value(&gemini_dir.join("oauth_creds.json"))?;
    let settings = read_optional_json_value(&gemini_dir.join("settings.json"))?;
    extract_account_identity(&oauth_creds, settings.as_ref())
}

pub fn match_active_identity<'a>(
    live: &GeminiAccountIdentity,
    stored: &'a [StoredGeminiAccount],
) -> Option<&'a StoredGeminiAccount> {
    stored
        .iter()
        .find(|account| account.email.eq_ignore_ascii_case(&live.email))
        .or_else(|| {
            live.subject.as_deref().and_then(|subject| {
                stored
                    .iter()
                    .find(|account| account.subject.as_deref() == Some(subject))
            })
        })
}

fn decode_id_token_claims(token: &str) -> Result<Value, String> {
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| "id_token is missing payload".to_string())?;
    let decoded = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| format!("failed to decode id_token payload: {error}"))?;
    serde_json::from_slice(&decoded).map_err(|error| format!("invalid id_token json: {error}"))
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
