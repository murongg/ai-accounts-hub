use ai_accounts_hub_lib::gemini_accounts::auth::{extract_account_identity, match_active_identity};
use ai_accounts_hub_lib::gemini_accounts::models::{GeminiAccountIdentity, StoredGeminiAccount};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::json;

fn id_token(claims: serde_json::Value) -> String {
    let header = URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let payload = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&claims).expect("claims"));
    format!("{header}.{payload}.signature")
}

#[test]
fn extracts_identity_from_oauth_files() {
    let identity = extract_account_identity(
        &json!({
            "access_token": "access-token",
            "id_token": id_token(json!({
                "email": "gemini@example.com",
                "sub": "google-sub-123"
            })),
        }),
        Some(&json!({
            "security": {
                "auth": {
                    "selectedType": "oauth-personal"
                }
            }
        })),
    )
    .expect("identity");

    assert_eq!(identity.email, "gemini@example.com");
    assert_eq!(identity.subject.as_deref(), Some("google-sub-123"));
    assert_eq!(identity.auth_type, Some("oauth-personal".to_string()));
}

#[test]
fn rejects_oauth_credentials_without_email_claim() {
    let error = extract_account_identity(
        &json!({
            "id_token": id_token(json!({
                "email": "",
                "sub": "google-sub-123"
            })),
        }),
        None,
    )
    .expect_err("missing email should fail");

    assert!(error.contains("email"));
}

#[test]
fn matches_live_identity_by_email_before_subject() {
    let live = GeminiAccountIdentity {
        email: "gemini@example.com".into(),
        subject: Some("live-sub".into()),
        auth_type: Some("oauth-personal".into()),
    };
    let stored = vec![
        StoredGeminiAccount::new_for_tests("other@example.com", Some("live-sub")),
        StoredGeminiAccount::new_for_tests("gemini@example.com", Some("old-sub")),
    ];

    let matched = match_active_identity(&live, &stored).expect("match");
    assert_eq!(matched.email, "gemini@example.com");
}
