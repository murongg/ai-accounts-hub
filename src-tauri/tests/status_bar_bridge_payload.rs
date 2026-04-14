use ai_accounts_hub_lib::claude_accounts::models::ClaudeAccountListItem;
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

fn claude_account(
    id: &str,
    email: &str,
    is_active: bool,
    session_remaining_percent: Option<u8>,
    weekly_remaining_percent: Option<u8>,
    model_weekly_label: Option<&str>,
    model_weekly_remaining_percent: Option<u8>,
) -> ClaudeAccountListItem {
    ClaudeAccountListItem {
        id: id.to_string(),
        email: email.to_string(),
        display_name: Some(format!("Claude {id}")),
        plan: Some("Pro".to_string()),
        account_hint: Some(format!("org-{id}")),
        is_active,
        last_authenticated_at: "1775640000".to_string(),
        session_remaining_percent,
        session_refresh_at: Some("1775650800".to_string()),
        weekly_remaining_percent,
        weekly_refresh_at: Some("1776248400".to_string()),
        model_weekly_label: model_weekly_label.map(str::to_string),
        model_weekly_remaining_percent,
        model_weekly_refresh_at: Some("1776248400".to_string()),
        last_synced_at: Some("1775642700".to_string()),
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
            claude_account(
                "idle-c",
                "idle-c@example.com",
                false,
                None,
                None,
                None,
                None,
            ),
            claude_account(
                "active-c",
                "active-c@example.com",
                true,
                None,
                None,
                None,
                None,
            ),
        ],
        vec![
            gemini_account(
                "idle-g",
                "idle-g@example.com",
                false,
                Some(88),
                Some(70),
                Some(52),
            ),
            gemini_account(
                "active-g",
                "active-g@example.com",
                true,
                Some(100),
                Some(90),
                Some(75),
            ),
        ],
        1_775_643_000_000,
    );

    assert_eq!(payload.selected_tab, StatusBarTab::Overview);
    assert_eq!(payload.sections.len(), 3);
    assert_eq!(payload.sections[0].provider_id, "codex");
    assert_eq!(payload.sections[0].email, "active@example.com");
    assert_eq!(payload.sections[1].provider_id, "claude");
    assert_eq!(payload.sections[1].email, "active-c@example.com");
    assert_eq!(payload.sections[2].provider_id, "gemini");
    assert_eq!(payload.sections[2].email, "active-g@example.com");
}

#[test]
fn codex_payload_includes_session_and_weekly_metrics() {
    let payload = build_bridge_payload(
        StatusBarTab::Codex,
        vec![codex_account(
            "active",
            "active@example.com",
            true,
            Some(82),
            Some(64),
        )],
        Vec::new(),
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
fn codex_payload_keeps_reset_countdown_at_minute_precision() {
    let mut account = codex_account("active", "active@example.com", true, Some(82), Some(64));
    account.five_hour_refresh_at = Some("1775645400".to_string());
    account.weekly_refresh_at = Some("1776249300".to_string());

    let payload = build_bridge_payload(
        StatusBarTab::Codex,
        vec![account],
        Vec::new(),
        Vec::new(),
        1_775_640_000_000,
    );

    assert_eq!(payload.sections.len(), 1);
    assert_eq!(
        payload.sections[0].metrics[0].reset_text,
        "Resets in 1h 30m"
    );
    assert_eq!(
        payload.sections[0].metrics[1].reset_text,
        "Resets in 7d 1h 15m"
    );
}

#[test]
fn provider_payload_uses_weekly_before_five_hour_and_then_credits_as_tie_breakers() {
    let mut relogin = codex_account(
        "relogin",
        "relogin@example.com",
        false,
        Some(100),
        Some(100),
    );
    relogin.needs_relogin = Some(true);

    let payload = build_bridge_payload(
        StatusBarTab::Codex,
        vec![
            codex_account(
                "weekly-low",
                "weekly-low@example.com",
                false,
                Some(50),
                Some(40),
            ),
            codex_account("missing", "missing@example.com", false, None, Some(99)),
            codex_account("active", "active@example.com", true, Some(3), Some(4)),
            {
                let mut account = codex_account(
                    "weekly-high-credits-high",
                    "weekly-high-credits-high@example.com",
                    false,
                    Some(50),
                    Some(90),
                );
                account.credits_balance = Some(200.0);
                account
            },
            {
                let mut account = codex_account(
                    "weekly-high-credits-low",
                    "weekly-high-credits-low@example.com",
                    false,
                    Some(50),
                    Some(90),
                );
                account.credits_balance = Some(10.0);
                account
            },
            {
                let mut account = codex_account(
                    "lower-primary",
                    "lower-primary@example.com",
                    false,
                    Some(49),
                    Some(100),
                );
                account.credits_balance = Some(999.0);
                account
            },
            relogin,
        ],
        Vec::new(),
        Vec::new(),
        1_775_640_000_000,
    );

    let ids: Vec<&str> = payload
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();

    assert_eq!(
        ids,
        vec![
            "codex:active",
            "codex:lower-primary",
            "codex:weekly-high-credits-high",
            "codex:weekly-high-credits-low",
            "codex:weekly-low",
            "codex:missing",
            "codex:relogin",
        ]
    );
}

#[test]
fn provider_payload_uses_all_provider_quotas_for_sorting() {
    let claude_payload = build_bridge_payload(
        StatusBarTab::Claude,
        Vec::new(),
        vec![
            claude_account(
                "weekly-high-session-low",
                "weekly-high-session-low@example.com",
                false,
                Some(10),
                Some(90),
                None,
                Some(10),
            ),
            claude_account(
                "weekly-high-session-high",
                "weekly-high-session-high@example.com",
                false,
                Some(70),
                Some(90),
                None,
                Some(70),
            ),
            claude_account(
                "weekly-low-session-high",
                "weekly-low-session-high@example.com",
                false,
                Some(99),
                Some(80),
                None,
                Some(99),
            ),
        ],
        Vec::new(),
        1_775_640_000_000,
    );
    let gemini_payload = build_bridge_payload(
        StatusBarTab::Gemini,
        Vec::new(),
        Vec::new(),
        vec![
            gemini_account(
                "flash-high-lite-low",
                "flash-high-lite-low@example.com",
                false,
                Some(88),
                Some(90),
                Some(10),
            ),
            gemini_account(
                "flash-high-lite-high",
                "flash-high-lite-high@example.com",
                false,
                Some(88),
                Some(90),
                Some(70),
            ),
            gemini_account(
                "flash-low",
                "flash-low@example.com",
                false,
                Some(88),
                Some(80),
                Some(99),
            ),
        ],
        1_775_640_000_000,
    );

    let claude_ids: Vec<&str> = claude_payload
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();
    let gemini_ids: Vec<&str> = gemini_payload
        .sections
        .iter()
        .map(|section| section.id.as_str())
        .collect();

    assert_eq!(
        claude_ids,
        vec![
            "claude:weekly-high-session-high",
            "claude:weekly-high-session-low",
            "claude:weekly-low-session-high"
        ]
    );
    assert_eq!(
        gemini_ids,
        vec![
            "gemini:flash-high-lite-high",
            "gemini:flash-high-lite-low",
            "gemini:flash-low"
        ]
    );
}

#[test]
fn relogin_payload_clears_metrics_and_marks_status() {
    let mut broken = gemini_account(
        "bad",
        "broken@example.com",
        false,
        Some(100),
        Some(90),
        Some(75),
    );
    broken.needs_relogin = Some(true);

    let payload = build_bridge_payload(
        StatusBarTab::Gemini,
        Vec::new(),
        Vec::new(),
        vec![broken],
        1_775_643_000_000,
    );

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
            primary_quota_percent: None,
            metrics: Vec::new(),
            switch_account_id: Some("bad".to_string()),
        }
    );
}

#[test]
fn claude_payload_includes_session_weekly_and_model_metrics() {
    let payload = build_bridge_payload(
        StatusBarTab::Claude,
        Vec::new(),
        vec![claude_account(
            "active-c",
            "active-c@example.com",
            true,
            Some(82),
            Some(74),
            Some("Opus Weekly"),
            Some(61),
        )],
        Vec::new(),
        1_775_643_000_000,
    );

    assert_eq!(payload.sections.len(), 1);
    assert_eq!(
        payload.sections[0],
        BridgeProviderPayload {
            id: "claude:active-c".to_string(),
            provider_id: "claude".to_string(),
            provider_title: "Claude".to_string(),
            email: "active-c@example.com".to_string(),
            subtitle: "Updated 5m ago".to_string(),
            plan: Some("Pro".to_string()),
            is_active: true,
            needs_relogin: false,
            primary_quota_percent: Some(82),
            metrics: vec![
                BridgeMetricPayload {
                    title: "Session".to_string(),
                    percent: 82,
                    left_text: "82% left".to_string(),
                    reset_text: "Resets in 2h 10m".to_string(),
                },
                BridgeMetricPayload {
                    title: "Weekly".to_string(),
                    percent: 74,
                    left_text: "74% left".to_string(),
                    reset_text: "Resets in 7d 10m".to_string(),
                },
                BridgeMetricPayload {
                    title: "Opus Weekly".to_string(),
                    percent: 61,
                    left_text: "61% left".to_string(),
                    reset_text: "Resets in 7d 10m".to_string(),
                },
            ],
            switch_account_id: None,
        }
    );
}
