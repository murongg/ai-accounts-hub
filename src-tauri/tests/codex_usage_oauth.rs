use ai_accounts_hub_lib::codex_usage::models::{
    CodexUsageApiResponse, CodexUsageApiWindow, CodexUsageCredits, CodexUsageRateLimit,
};
use ai_accounts_hub_lib::codex_usage::oauth::normalize_usage_response;
use serde_json::json;

#[test]
fn normalizes_primary_and_weekly_windows_into_remaining_snapshots() {
    let response = CodexUsageApiResponse {
        plan_type: Some("pro".into()),
        rate_limit: Some(CodexUsageRateLimit {
            primary_window: Some(CodexUsageApiWindow {
                used_percent: 18,
                reset_at: 1_800_000_000,
                limit_window_seconds: 18_000,
            }),
            secondary_window: Some(CodexUsageApiWindow {
                used_percent: 12,
                reset_at: 1_800_500_000,
                limit_window_seconds: 604_800,
            }),
        }),
        credits: Some(CodexUsageCredits {
            has_credits: true,
            unlimited: false,
            balance: Some(42.5),
        }),
    };

    let normalized = normalize_usage_response(response);

    assert_eq!(normalized.plan.as_deref(), Some("Pro"));
    assert_eq!(
        normalized
            .five_hour
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(82)
    );
    assert_eq!(
        normalized
            .weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(88)
    );
    assert_eq!(normalized.credits_balance, Some(42.5));
}

#[test]
fn recognizes_weekly_only_payloads() {
    let response = CodexUsageApiResponse {
        plan_type: Some("plus".into()),
        rate_limit: Some(CodexUsageRateLimit {
            primary_window: Some(CodexUsageApiWindow {
                used_percent: 35,
                reset_at: 1_800_500_000,
                limit_window_seconds: 604_800,
            }),
            secondary_window: None,
        }),
        credits: None,
    };

    let normalized = normalize_usage_response(response);

    assert!(normalized.five_hour.is_none());
    assert_eq!(
        normalized
            .weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(65)
    );
}

#[test]
fn parses_string_credit_balances_from_live_usage_payloads() {
    let response: CodexUsageApiResponse = serde_json::from_value(json!({
        "plan_type": "plus",
        "rate_limit": {
            "primary_window": {
                "used_percent": 0,
                "limit_window_seconds": 18000,
                "reset_after_seconds": 18000,
                "reset_at": 1775575738_u64
            },
            "secondary_window": {
                "used_percent": 71,
                "limit_window_seconds": 604800,
                "reset_after_seconds": 160709,
                "reset_at": 1775718446_u64
            }
        },
        "credits": {
            "has_credits": false,
            "unlimited": false,
            "balance": "0"
        }
    }))
    .expect("live usage payload should deserialize");

    let normalized = normalize_usage_response(response);

    assert_eq!(normalized.plan.as_deref(), Some("Plus"));
    assert_eq!(
        normalized
            .five_hour
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(100)
    );
    assert_eq!(
        normalized
            .weekly
            .as_ref()
            .map(|window| window.remaining_percent),
        Some(29)
    );
    assert_eq!(normalized.credits_balance, Some(0.0));
}
