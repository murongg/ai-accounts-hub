use ai_accounts_hub_lib::gemini_usage::models::{
    GeminiTierId, GeminiUsageApiBucket, GeminiUsageApiResponse,
};
use ai_accounts_hub_lib::gemini_usage::oauth::{
    normalize_usage_response, should_require_relogin_for_quota_status,
};

#[test]
fn normalizes_paid_plan_and_maps_pro_and_flash_quotas() {
    let response = GeminiUsageApiResponse {
        buckets: Some(vec![
            GeminiUsageApiBucket {
                remaining_fraction: Some(0.82),
                reset_time: Some("2026-04-09T02:30:00Z".into()),
                model_id: Some("gemini-2.5-pro".into()),
                remaining_amount: Some("82".into()),
                token_type: None,
            },
            GeminiUsageApiBucket {
                remaining_fraction: Some(0.65),
                reset_time: Some("2026-04-09T03:45:00Z".into()),
                model_id: Some("gemini-2.5-flash".into()),
                remaining_amount: Some("65".into()),
                token_type: None,
            },
        ]),
    };

    let normalized = normalize_usage_response(response, Some(GeminiTierId::Standard), None)
        .expect("normalized");

    assert_eq!(normalized.plan.as_deref(), Some("Paid"));
    assert_eq!(normalized.pro.as_ref().map(|window| window.remaining_percent), Some(82));
    assert_eq!(normalized.flash.as_ref().map(|window| window.remaining_percent), Some(65));
    assert_eq!(normalized.pro.as_ref().map(|window| window.reset_at.as_str()), Some("2026-04-09T02:30:00Z"));
}

#[test]
fn maps_free_workspace_accounts_when_hosted_domain_exists() {
    let response = GeminiUsageApiResponse {
        buckets: Some(vec![GeminiUsageApiBucket {
            remaining_fraction: Some(0.91),
            reset_time: Some("2026-04-09T00:15:00Z".into()),
            model_id: Some("gemini-2.5-pro".into()),
            remaining_amount: Some("91".into()),
            token_type: None,
        }]),
    };

    let normalized = normalize_usage_response(
        response,
        Some(GeminiTierId::Free),
        Some("workspace.example.com"),
    )
    .expect("normalized");

    assert_eq!(normalized.plan.as_deref(), Some("Workspace"));
    assert_eq!(normalized.pro.as_ref().map(|window| window.remaining_percent), Some(91));
    assert!(normalized.flash.is_none());
}

#[test]
fn rejects_quota_payloads_without_buckets() {
    let error = normalize_usage_response(GeminiUsageApiResponse { buckets: Some(vec![]) }, None, None)
        .expect_err("empty buckets should fail");

    assert!(error.contains("buckets"));
}

#[test]
fn falls_back_to_no_plan_when_tier_detection_is_unavailable() {
    let response = GeminiUsageApiResponse {
        buckets: Some(vec![GeminiUsageApiBucket {
            remaining_fraction: Some(0.73),
            reset_time: Some("2026-04-09T03:45:00Z".into()),
            model_id: Some("gemini-2.5-flash".into()),
            remaining_amount: Some("73".into()),
            token_type: None,
        }]),
    };

    let normalized = normalize_usage_response(response, None, None).expect("normalized");

    assert_eq!(normalized.plan, None);
    assert_eq!(normalized.flash.as_ref().map(|window| window.remaining_percent), Some(73));
}

#[test]
fn maps_flash_lite_quota_into_tertiary_window() {
    let response = GeminiUsageApiResponse {
        buckets: Some(vec![GeminiUsageApiBucket {
            remaining_fraction: Some(0.58),
            reset_time: Some("2026-04-09T05:15:00Z".into()),
            model_id: Some("gemini-2.5-flash-lite".into()),
            remaining_amount: Some("58".into()),
            token_type: None,
        }]),
    };

    let normalized = normalize_usage_response(response, Some(GeminiTierId::Standard), None)
        .expect("normalized");

    assert_eq!(
        normalized.flash_lite.as_ref().map(|window| window.remaining_percent),
        Some(58)
    );
}

#[test]
fn only_401_from_quota_endpoint_requires_relogin() {
    assert!(should_require_relogin_for_quota_status(401));
    assert!(!should_require_relogin_for_quota_status(403));
    assert!(!should_require_relogin_for_quota_status(500));
}
