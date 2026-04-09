use ai_accounts_hub_lib::claude_accounts::auth::{extract_account_identity, match_active_identity};
use ai_accounts_hub_lib::claude_accounts::models::{ClaudeAccountIdentity, StoredClaudeAccount};
use serde_json::json;

#[test]
fn extracts_identity_from_real_claude_storage_shapes() {
    let credentials = json!({
        "claudeAiOauth": {
            "subscriptionType": "pro",
            "rateLimitTier": "default"
        }
    });
    let oauth_account = json!({
        "emailAddress": "murong@example.com",
        "displayName": "Murong",
        "accountUuid": "owner-a"
    });

    let identity = extract_account_identity(&credentials, Some(&oauth_account))
        .expect("identity should parse");

    assert_eq!(identity.email, "murong@example.com");
    assert_eq!(identity.display_name.as_deref(), Some("Murong"));
    assert_eq!(identity.plan.as_deref(), Some("Pro"));
    assert_eq!(identity.account_hint.as_deref(), Some("owner-a"));
}

#[test]
fn matches_live_identity_by_email_before_hint() {
    let live = ClaudeAccountIdentity {
        email: "murong@example.com".into(),
        display_name: Some("Murong".into()),
        plan: Some("Pro".into()),
        account_hint: Some("owner-a".into()),
    };
    let stored = vec![
        StoredClaudeAccount::new_for_tests("other@example.com", Some("owner-a")),
        StoredClaudeAccount::new_for_tests("murong@example.com", Some("owner-b")),
    ];

    let matched = match_active_identity(&live, &stored).expect("should match");
    assert_eq!(matched.email, "murong@example.com");
}
