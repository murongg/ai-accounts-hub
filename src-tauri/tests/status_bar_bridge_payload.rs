use ai_accounts_hub_lib::codex_accounts::models::CodexAccountListItem;
use ai_accounts_hub_lib::gemini_accounts::models::GeminiAccountListItem;
use ai_accounts_hub_lib::status_bar::bridge_payload::{
    build_bridge_payload, BridgeMetricPayload, BridgeProviderPayload, StatusBarTab,
};

fn codex_account(
    id: &str,
    email: &str,
    is_active: bool,
    five_hour_remaining_percent: Option<u8>,
    weekly_remaining_percent: Option<u8>,
) -> CodexAccountListItem {
    CodexAccountListItem {
        id: id.to_string(),
        email: email.to_string(),
        plan: Some("Plus".to_string()),
        account_id: Some(format!("acct-{id}")),
        is_active,
        last_authenticated_at: "1775640000".to_string(),
        five_hour_remaining_percent,
        weekly_remaining_percent,
        five_hour_refresh_at: Some("1775643600".to_string()),
        weekly_refresh_at: Some("1776248400".to_string()),
        last_synced_at: Some("1775643000".to_string()),
        last_sync_error: None,
        credits_balance: None,
        needs_relogin: Some(false),
    }
}

fn gemini_account(
    id: &str,
    email: &str,
    is_active: bool,
    pro_remaining_percent: Option<u8>,
    flash_remaining_percent: Option<u8>,
    flash_lite_remaining_percent: Option<u8>,
) -> GeminiAccountListItem {
    GeminiAccountListItem {
        id: id.to_string(),
        email: email.to_string(),
        subject: Some(format!("sub-{id}")),
        auth_type: Some("oauth-personal".to_string()),
        plan: Some("Paid".to_string()),
        is_active,
        last_authenticated_at: "1775640000".to_string(),
        pro_remaining_percent,
        flash_remaining_percent,
        flash_lite_remaining_percent,
        pro_refresh_at: Some("2026-04-09T00:00:00Z".to_string()),
        flash_refresh_at: Some("2026-04-09T00:00:00Z".to_string()),
        flash_lite_refresh_at: Some("2026-04-09T00:00:00Z".to_string()),
        last_synced_at: Some("1775643000".to_string()),
        last_sync_error: None,
        needs_relogin: Some(false),
    }
}

#[test]
fn overview_payload_uses_active_accounts_from_both_providers() {
    let payload = build_bridge_payload(
        StatusBarTab::Overview,
        vec![
            codex_account("idle", "idle@example.com", false, Some(63), Some(58)),
            codex_account("active", "active@example.com", true, Some(82), Some(64)),
        ],
        vec![
            gemini_account("idle-g", "idle-g@example.com", false, Some(88), Some(70), Some(52)),
            gemini_account("active-g", "active-g@example.com", true, Some(100), Some(90), Some(75)),
        ],
        1_775_643_000_000,
    );

    assert_eq!(payload.selected_tab, StatusBarTab::Overview);
    assert_eq!(payload.sections.len(), 2);
    assert_eq!(payload.sections[0].provider_id, "codex");
    assert_eq!(payload.sections[0].email, "active@example.com");
    assert_eq!(payload.sections[1].provider_id, "gemini");
    assert_eq!(payload.sections[1].email, "active-g@example.com");
}

#[test]
fn codex_payload_includes_session_and_weekly_metrics() {
    let payload = build_bridge_payload(
        StatusBarTab::Codex,
        vec![codex_account("active", "active@example.com", true, Some(82), Some(64))],
        Vec::new(),
        1_775_640_000_000,
    );

    assert_eq!(payload.sections.len(), 1);
    assert_eq!(
        payload.sections[0].metrics,
        vec![
            BridgeMetricPayload {
                title: "Session".to_string(),
                percent: 82,
                left_text: "82% left".to_string(),
                reset_text: "Resets in 1h".to_string(),
            },
            BridgeMetricPayload {
                title: "Weekly".to_string(),
                percent: 64,
                left_text: "64% left".to_string(),
                reset_text: "Resets in 7d 1h".to_string(),
            },
        ]
    );
}

#[test]
fn relogin_payload_clears_metrics_and_marks_status() {
    let mut broken = gemini_account("bad", "broken@example.com", false, Some(100), Some(90), Some(75));
    broken.needs_relogin = Some(true);

    let payload = build_bridge_payload(StatusBarTab::Gemini, Vec::new(), vec![broken], 1_775_643_000_000);

    assert_eq!(
        payload.sections[0],
        BridgeProviderPayload {
            id: "gemini:bad".to_string(),
            provider_id: "gemini".to_string(),
            provider_title: "Gemini".to_string(),
            email: "broken@example.com".to_string(),
            subtitle: "Re-login required".to_string(),
            plan: Some("Paid".to_string()),
            is_active: false,
            needs_relogin: true,
            metrics: Vec::new(),
            switch_account_id: Some("bad".to_string()),
        }
    );
}
