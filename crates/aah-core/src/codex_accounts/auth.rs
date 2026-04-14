use std::fs;
use std::path::Path;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::Value;

use super::models::{CodexAccountIdentity, StoredCodexAccount};

pub fn extract_account_identity(auth: &Value) -> Result<CodexAccountIdentity, String> {
    let tokens = auth
        .get("tokens")
        .and_then(Value::as_object)
        .ok_or_else(|| "auth.json is missing tokens".to_string())?;

    let account_id = tokens
        .get("account_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let id_token = tokens
        .get("id_token")
        .and_then(Value::as_str)
        .ok_or_else(|| "auth.json is missing tokens.id_token".to_string())?;
    let claims = decode_id_token_claims(id_token)?;
    let email = claims
        .get("email")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| "auth.json is missing a usable email".to_string())?
        .to_lowercase();
    let plan = claims
        .get("https://api.openai.com/auth")
        .and_then(|value| value.get("chatgpt_plan_type"))
        .and_then(Value::as_str)
        .map(format_plan);

    Ok(CodexAccountIdentity {
        email,
        account_id,
        plan,
    })
}

pub fn read_account_identity_from_path(path: &Path) -> Result<CodexAccountIdentity, String> {
    let auth = read_auth_value(path)?;
    extract_account_identity(&auth)
}

pub fn read_auth_value(path: &Path) -> Result<Value, String> {
    let bytes =
        fs::read(path).map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

pub fn match_active_identity<'a>(
    live: &CodexAccountIdentity,
    stored: &'a [StoredCodexAccount],
) -> Option<&'a StoredCodexAccount> {
    stored
        .iter()
        .find(|account| account.email.eq_ignore_ascii_case(&live.email))
        .or_else(|| {
            live.account_id.as_deref().and_then(|account_id| {
                stored
                    .iter()
                    .find(|account| account.account_id.as_deref() == Some(account_id))
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

pub fn format_plan(raw: &str) -> String {
    match raw.trim().to_ascii_lowercase().as_str() {
        "plus" => "Plus".to_string(),
        "free" => "Free".to_string(),
        "pro" => "Pro".to_string(),
        other if !other.is_empty() => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => {
                    let mut normalized = first.to_uppercase().collect::<String>();
                    normalized.push_str(chars.as_str());
                    normalized
                }
                None => "Unknown".to_string(),
            }
        }
        _ => "Unknown".to_string(),
    }
}
