use ai_accounts_hub_lib::claude_accounts::models::ClaudeAccountListItem;
use ai_accounts_hub_lib::codex_accounts::models::CodexAccountListItem;
use ai_accounts_hub_lib::gemini_accounts::models::GeminiAccountListItem;
use ai_accounts_hub_lib::status_bar::menu_model::{
    build_provider_menu_state, parse_menu_action, MenuAction, MenuProvider,
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
        last_authenticated_at: "0".to_string(),
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
        last_authenticated_at: "0".to_string(),
        pro_remaining_percent,
        flash_remaining_percent,
        flash_lite_remaining_percent,
        pro_refresh_at: Some("2026-04-08T10:31:46Z".to_string()),
        flash_refresh_at: Some("2026-04-08T10:31:46Z".to_string()),
        flash_lite_refresh_at: Some("2026-04-08T10:31:46Z".to_string()),
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
        last_authenticated_at: "0".to_string(),
        session_remaining_percent,
        session_refresh_at: Some("1775643600".to_string()),
        weekly_remaining_percent,
        weekly_refresh_at: Some("1776248400".to_string()),
        model_weekly_label: model_weekly_label.map(str::to_string),
        model_weekly_remaining_percent,
        model_weekly_refresh_at: Some("1776248400".to_string()),
        last_synced_at: Some("1775643000".to_string()),
        last_sync_error: None,
        needs_relogin: Some(false),
    }
}

#[test]
fn codex_menu_state_puts_active_account_first_and_formats_quota_summary() {
    let state = build_provider_menu_state(
        MenuProvider::Codex,
        vec![
            codex_account("idle", "idle@example.com", false, Some(63), Some(58)),
            codex_account("active", "active@example.com", true, Some(82), Some(64)),
        ],
        Vec::new(),
        Vec::new(),
    );

    assert_eq!(state.selected_provider, MenuProvider::Codex);
    assert_eq!(state.accounts.len(), 2);
    assert_eq!(state.accounts[0].id, "active");
    assert_eq!(
        state.accounts[0].quota_summary.as_deref(),
        Some("5h 82% • Week 64%")
    );
    assert!(state.accounts[0].is_active);
}

#[test]
fn codex_menu_state_uses_weekly_before_five_hour_and_then_credits_as_tie_breakers() {
    let mut relogin = codex_account(
        "relogin",
        "relogin@example.com",
        false,
        Some(100),
        Some(100),
    );
    relogin.needs_relogin = Some(true);

    let state = build_provider_menu_state(
        MenuProvider::Codex,
        vec![
            {
                let mut account = codex_account(
                    "weekly-low",
                    "weekly-low@example.com",
                    false,
                    Some(50),
                    Some(40),
                );
                account.credits_balance = Some(999.0);
                account
            },
            codex_account("missing", "missing@example.com", false, None, Some(99)),
            codex_account("active", "active@example.com", true, Some(3), Some(4)),
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
    );

    let ids: Vec<&str> = state
        .accounts
        .iter()
        .map(|account| account.id.as_str())
        .collect();

    assert_eq!(
        ids,
        vec![
            "active",
            "lower-primary",
            "weekly-high-credits-high",
            "weekly-high-credits-low",
            "weekly-low",
            "missing",
            "relogin",
        ]
    );
}

#[test]
fn provider_menu_state_uses_all_provider_quotas_for_sorting() {
    let claude_state = build_provider_menu_state(
        MenuProvider::Claude,
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
                "weekly-low-session-high",
                "weekly-low-session-high@example.com",
                false,
                Some(99),
                Some(80),
                None,
                Some(99),
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
        ],
        Vec::new(),
    );
    let gemini_state = build_provider_menu_state(
        MenuProvider::Gemini,
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
                "flash-low",
                "flash-low@example.com",
                false,
                Some(88),
                Some(80),
                Some(99),
            ),
            gemini_account(
                "flash-high-lite-high",
                "flash-high-lite-high@example.com",
                false,
                Some(88),
                Some(90),
                Some(70),
            ),
        ],
    );

    let claude_ids: Vec<&str> = claude_state
        .accounts
        .iter()
        .map(|account| account.id.as_str())
        .collect();
    let gemini_ids: Vec<&str> = gemini_state
        .accounts
        .iter()
        .map(|account| account.id.as_str())
        .collect();

    assert_eq!(
        claude_ids,
        vec![
            "weekly-high-session-high",
            "weekly-high-session-low",
            "weekly-low-session-high"
        ]
    );
    assert_eq!(
        gemini_ids,
        vec!["flash-high-lite-high", "flash-high-lite-low", "flash-low"]
    );
}

#[test]
fn gemini_menu_state_formats_three_quota_buckets() {
    let state = build_provider_menu_state(
        MenuProvider::Gemini,
        Vec::new(),
        Vec::new(),
        vec![gemini_account(
            "gem",
            "gemini@example.com",
            false,
            Some(100),
            Some(90),
            Some(75),
        )],
    );

    assert_eq!(state.accounts.len(), 1);
    assert_eq!(
        state.accounts[0].quota_summary.as_deref(),
        Some("Pro 100% • Flash 90% • Lite 75%")
    );
}

#[test]
fn menu_state_marks_accounts_needing_relogin() {
    let mut account = codex_account("bad", "broken@example.com", false, None, None);
    account.needs_relogin = Some(true);

    let state =
        build_provider_menu_state(MenuProvider::Codex, vec![account], Vec::new(), Vec::new());

    assert_eq!(state.accounts[0].status_label, "Re-login required");
    assert_eq!(state.accounts[0].quota_summary, None);
}

#[test]
fn claude_menu_state_shows_status_without_quota_summary() {
    let state = build_provider_menu_state(
        MenuProvider::Claude,
        Vec::new(),
        vec![claude_account(
            "claude",
            "claude@example.com",
            true,
            Some(82),
            Some(74),
            Some("Opus Weekly"),
            Some(61),
        )],
        Vec::new(),
    );

    assert_eq!(state.selected_provider, MenuProvider::Claude);
    assert_eq!(state.accounts.len(), 1);
    assert_eq!(state.accounts[0].id, "claude");
    assert_eq!(
        state.accounts[0].quota_summary.as_deref(),
        Some("Session 82% • Week 74% • Opus 61%")
    );
    assert_eq!(state.accounts[0].status_label, "Healthy");
}

#[test]
fn parse_menu_action_understands_provider_switch_and_account_actions() {
    assert_eq!(
        parse_menu_action("provider:codex"),
        Some(MenuAction::SelectProvider(MenuProvider::Codex))
    );
    assert_eq!(
        parse_menu_action("provider:claude"),
        Some(MenuAction::SelectProvider(MenuProvider::Claude))
    );
    assert_eq!(
        parse_menu_action("provider:gemini"),
        Some(MenuAction::SelectProvider(MenuProvider::Gemini))
    );
    assert_eq!(
        parse_menu_action("switch:codex:acct-1"),
        Some(MenuAction::SwitchAccount(
            MenuProvider::Codex,
            "acct-1".to_string()
        ))
    );
    assert_eq!(
        parse_menu_action("switch:claude:acct-3"),
        Some(MenuAction::SwitchAccount(
            MenuProvider::Claude,
            "acct-3".to_string()
        ))
    );
    assert_eq!(
        parse_menu_action("switch:gemini:acct-2"),
        Some(MenuAction::SwitchAccount(
            MenuProvider::Gemini,
            "acct-2".to_string()
        ))
    );
    assert_eq!(
        parse_menu_action("refresh"),
        Some(MenuAction::RefreshSelectedProvider)
    );
    assert_eq!(
        parse_menu_action("open-main"),
        Some(MenuAction::OpenMainWindow)
    );
    assert_eq!(parse_menu_action("quit"), Some(MenuAction::Quit));
    assert_eq!(parse_menu_action("switch:codex:"), None);
    assert_eq!(parse_menu_action("ignored"), None);
}
