use ai_accounts_hub_lib::codex_accounts::auth::{extract_account_identity, match_active_identity};
use ai_accounts_hub_lib::codex_accounts::models::{CodexAccountIdentity, StoredCodexAccount};
use serde_json::json;

const HEADER: &str = "eyJhbGciOiJub25lIiwidHlwIjoiSldUIn0";
const WORK_PAYLOAD: &str = "eyJlbWFpbCI6IndvcmtAZXhhbXBsZS5jb20iLCJodHRwczovL2FwaS5vcGVuYWkuY29tL2F1dGgiOnsiY2hhdGdwdF9wbGFuX3R5cGUiOiJwbHVzIiwiY2hhdGdwdF9hY2NvdW50X2lkIjoiYWNjdF8xMjMifX0";
const EMPTY_EMAIL_PAYLOAD: &str = "eyJlbWFpbCI6IiIsImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6eyJjaGF0Z3B0X3BsYW5fdHlwZSI6InBsdXMiLCJjaGF0Z3B0X2FjY291bnRfaWQiOiJhY2N0XzEyMyJ9fQ";

fn id_token(payload: &str) -> String {
    format!("{HEADER}.{payload}.signature")
}

#[test]
fn extracts_identity_from_chatgpt_auth_json() {
    let auth = json!({
        "tokens": {
            "account_id": "acct_123",
            "id_token": id_token(WORK_PAYLOAD),
            "access_token": "access-token"
        }
    });

    let identity = extract_account_identity(&auth).expect("identity");
    assert_eq!(identity.email, "work@example.com");
    assert_eq!(identity.account_id.as_deref(), Some("acct_123"));
    assert_eq!(identity.plan.as_deref(), Some("Plus"));
}

#[test]
fn rejects_auth_without_email() {
    let auth = json!({
        "tokens": {
            "account_id": "acct_123",
            "id_token": id_token(EMPTY_EMAIL_PAYLOAD),
            "access_token": "access-token"
        }
    });

    let error = extract_account_identity(&auth).expect_err("missing email should fail");
    assert!(error.contains("email"));
}

#[test]
fn matches_live_identity_by_email_before_account_id() {
    let live = CodexAccountIdentity {
        email: "work@example.com".into(),
        account_id: Some("acct_live".into()),
        plan: Some("Plus".into()),
    };
    let stored = vec![
        StoredCodexAccount::new_for_tests("other@example.com", Some("acct_live")),
        StoredCodexAccount::new_for_tests("work@example.com", Some("acct_old")),
    ];

    let matched = match_active_identity(&live, &stored).expect("match");
    assert_eq!(matched.email, "work@example.com");
}
