use serde_json::Value;

use super::models::{ClaudeAccountIdentity, StoredClaudeAccount};

pub fn extract_account_identity(
    credentials: &Value,
    oauth_account: Option<&Value>,
) -> Result<ClaudeAccountIdentity, String> {
    let oauth = credentials.get("claudeAiOauth");

    let email = first_non_empty_string(&[
        oauth_account.and_then(|value| value.get("emailAddress")),
        credentials.get("email"),
        credentials.get("account_email"),
        credentials.get("primary_email"),
        oauth.and_then(|value| value.get("email")),
    ])
    .ok_or_else(|| "Claude credentials are missing a usable email".to_string())?
    .to_lowercase();

    Ok(ClaudeAccountIdentity {
        email,
        display_name: first_non_empty_string(&[
            oauth_account.and_then(|value| value.get("displayName")),
            credentials.get("display_name"),
            credentials.get("displayName"),
            credentials.get("name"),
        ]),
        plan: first_non_empty_string(&[
            oauth.and_then(|value| value.get("subscriptionType")),
            credentials.get("plan"),
            credentials.get("subscriptionType"),
            credentials.get("rate_limit_tier"),
            credentials.get("rateLimitTier"),
            oauth.and_then(|value| value.get("rate_limit_tier")),
            oauth.and_then(|value| value.get("rateLimitTier")),
        ])
        .as_deref()
        .map(format_plan),
        account_hint: first_non_empty_string(&[
            oauth_account.and_then(|value| value.get("accountUuid")),
            oauth_account.and_then(|value| value.get("organizationUuid")),
            credentials.get("account_hint"),
            credentials.get("accountHint"),
            credentials.get("account"),
            oauth.and_then(|value| value.get("account_hint")),
        ]),
    })
}

pub fn match_active_identity<'a>(
    live: &ClaudeAccountIdentity,
    stored: &'a [StoredClaudeAccount],
) -> Option<&'a StoredClaudeAccount> {
    stored
        .iter()
        .find(|account| account.email.eq_ignore_ascii_case(&live.email))
        .or_else(|| {
            live.account_hint.as_deref().and_then(|account_hint| {
                stored
                    .iter()
                    .find(|account| account.account_hint.as_deref() == Some(account_hint))
            })
        })
}

fn first_non_empty_string(values: &[Option<&Value>]) -> Option<String> {
    values
        .iter()
        .copied()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::trim)
        .find(|value| !value.is_empty())
        .map(str::to_string)
}

fn format_plan(raw: &str) -> String {
    let normalized = raw.trim().replace('_', " ").replace('-', " ");
    let lowered = normalized.to_ascii_lowercase();

    match lowered.as_str() {
        "free" => "Free".to_string(),
        "pro" => "Pro".to_string(),
        "max" => "Max".to_string(),
        "team" => "Team".to_string(),
        "enterprise" => "Enterprise".to_string(),
        other if !other.is_empty() => {
            let mut chars = other.chars();
            match chars.next() {
                Some(first) => {
                    let mut formatted = first.to_uppercase().collect::<String>();
                    formatted.push_str(chars.as_str());
                    formatted
                }
                None => "Unknown".to_string(),
            }
        }
        _ => "Unknown".to_string(),
    }
}
