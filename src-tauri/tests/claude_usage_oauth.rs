use ai_accounts_hub_lib::claude_accounts::live_credentials::ClaudeLiveCredentialSnapshot;
use ai_accounts_hub_lib::claude_usage::models::ClaudeUsageApiResponse;
use ai_accounts_hub_lib::claude_usage::oauth::{
    extract_oauth_credentials, normalize_usage_response, should_require_relogin_for_oauth_status,
    ClaudeOAuthCredentials, ClaudeOAuthHttpClient, ClaudeUsageFetchError, ClaudeUsageFetcher,
    OAuthClaudeUsageFetcher,
};
use serde_json::json;
use std::sync::Mutex;

#[test]
fn normalizes_session_weekly_and_prefers_opus_weekly() {
    let response: ClaudeUsageApiResponse = serde_json::from_value(json!({
        "five_hour": {
            "utilization": 0.18,
            "resets_at": "2026-04-10T08:00:00Z"
        },
        "seven_day": {
            "utilization": 0.26,
            "resets_at": "2026-04-14T00:00:00Z"
        },
        "seven_day_opus": {
            "utilization": 0.39,
            "resets_at": "2026-04-14T00:00:00Z"
        },
        "seven_day_sonnet": {
            "utilization": 0.12,
            "resets_at": "2026-04-14T00:00:00Z"
        }
    }))
    .expect("response");

    let usage = normalize_usage_response(response).expect("normalized");

    assert_eq!(
        usage
            .session
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(82)
    );
    assert_eq!(
        usage.weekly.as_ref().map(|window| window.remaining_percent),
        Some(74)
    );
    assert_eq!(usage.model_weekly_label.as_deref(), Some("Opus Weekly"));
    assert_eq!(
        usage
            .model_weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(61)
    );
}

#[test]
fn falls_back_to_sonnet_weekly_when_opus_is_missing() {
    let response: ClaudeUsageApiResponse = serde_json::from_value(json!({
        "five_hour": {
            "utilization": 0.05,
            "resets_at": "2026-04-10T08:00:00Z"
        },
        "seven_day": {
            "utilization": 0.40,
            "resets_at": "2026-04-14T00:00:00Z"
        },
        "seven_day_sonnet": {
            "utilization": 0.11,
            "resets_at": "2026-04-14T00:00:00Z"
        }
    }))
    .expect("response");

    let usage = normalize_usage_response(response).expect("normalized");

    assert_eq!(usage.model_weekly_label.as_deref(), Some("Sonnet Weekly"));
    assert_eq!(
        usage
            .model_weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(89)
    );
}

#[test]
fn normalizes_percentage_style_utilization_values_without_multiplying_again() {
    let response: ClaudeUsageApiResponse = serde_json::from_value(json!({
        "five_hour": {
            "utilization": 2.0,
            "resets_at": "2026-04-10T08:00:00Z"
        },
        "seven_day": {
            "utilization": 0.0,
            "resets_at": "2026-04-17T03:00:00Z"
        }
    }))
    .expect("response");

    let usage = normalize_usage_response(response).expect("normalized");

    assert_eq!(
        usage.session.as_ref().map(|window| window.used_percent),
        Some(2)
    );
    assert_eq!(
        usage
            .session
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(98)
    );
    assert_eq!(
        usage.weekly.as_ref().map(|window| window.used_percent),
        Some(0)
    );
    assert_eq!(
        usage.weekly.as_ref().map(|window| window.remaining_percent),
        Some(100)
    );
}

#[test]
fn extracts_refreshable_oauth_credentials_from_live_snapshot() {
    let snapshot = ClaudeLiveCredentialSnapshot {
        credentials_json: serde_json::to_vec(&json!({
            "claudeAiOauth": {
                "accessToken": "access-token",
                "refreshToken": "refresh-token"
            }
        }))
        .expect("json"),
        oauth_account_json: Some(
            serde_json::to_vec(&json!({
                "emailAddress": "murong@example.com"
            }))
            .expect("json"),
        ),
    };

    let credentials = extract_oauth_credentials(&snapshot).expect("credentials");
    assert_eq!(credentials.access_token, "access-token");
    assert_eq!(credentials.refresh_token.as_deref(), Some("refresh-token"));
}

#[test]
fn only_401_requires_relogin_for_oauth_failures() {
    assert!(should_require_relogin_for_oauth_status(401));
    assert!(!should_require_relogin_for_oauth_status(403));
    assert!(!should_require_relogin_for_oauth_status(500));
}

#[derive(Default)]
struct FakeClaudeOAuthHttpClient {
    usage_calls: Mutex<Vec<String>>,
    refresh_calls: Mutex<Vec<String>>,
}

impl ClaudeOAuthHttpClient for FakeClaudeOAuthHttpClient {
    fn get_usage(
        &self,
        access_token: &str,
    ) -> Result<ClaudeUsageApiResponse, ClaudeUsageFetchError> {
        self.usage_calls
            .lock()
            .expect("usage lock")
            .push(access_token.to_string());
        if access_token == "expired-token" {
            return Err(ClaudeUsageFetchError::Unauthorized);
        }

        Ok(serde_json::from_value(json!({
            "five_hour": {
                "utilization": 0.40,
                "resets_at": "2026-04-10T08:00:00Z"
            }
        }))
        .expect("response"))
    }

    fn refresh_access_token(
        &self,
        refresh_token: &str,
    ) -> Result<ClaudeOAuthCredentials, ClaudeUsageFetchError> {
        self.refresh_calls
            .lock()
            .expect("refresh lock")
            .push(refresh_token.to_string());
        Ok(ClaudeOAuthCredentials {
            access_token: "fresh-token".into(),
            refresh_token: Some(refresh_token.to_string()),
        })
    }
}

#[test]
fn fetcher_refreshes_token_once_after_401() {
    let client = FakeClaudeOAuthHttpClient::default();
    let fetcher = OAuthClaudeUsageFetcher::new(client);
    let snapshot = ClaudeLiveCredentialSnapshot {
        credentials_json: serde_json::to_vec(&json!({
            "claudeAiOauth": {
                "accessToken": "expired-token",
                "refreshToken": "refresh-token"
            }
        }))
        .expect("json"),
        oauth_account_json: Some(br#"{"emailAddress":"murong@example.com"}"#.to_vec()),
    };

    let usage = fetcher.fetch_usage(&snapshot).expect("usage");

    assert_eq!(
        usage
            .session
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(60)
    );
    assert_eq!(
        fetcher
            .client()
            .refresh_calls
            .lock()
            .expect("refresh lock")
            .as_slice(),
        ["refresh-token"]
    );
    assert_eq!(
        fetcher
            .client()
            .usage_calls
            .lock()
            .expect("usage lock")
            .as_slice(),
        ["expired-token", "fresh-token"]
    );
}

#[test]
fn unauthorized_without_refresh_token_requires_relogin() {
    let fetcher = OAuthClaudeUsageFetcher::new(FakeClaudeOAuthHttpClient::default());
    let snapshot = ClaudeLiveCredentialSnapshot {
        credentials_json: serde_json::to_vec(&json!({
            "claudeAiOauth": {
                "accessToken": "expired-token"
            }
        }))
        .expect("json"),
        oauth_account_json: Some(br#"{"emailAddress":"murong@example.com"}"#.to_vec()),
    };

    let error = fetcher.fetch_usage(&snapshot).expect_err("error");

    assert!(error.needs_relogin());
}
