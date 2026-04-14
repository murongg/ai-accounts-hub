use ai_accounts_hub_lib::claude_accounts::live_credentials::ClaudeLiveCredentialSnapshot;
use ai_accounts_hub_lib::relay::credentials::{
    claude_credential_from_snapshot, codex_credential_from_auth, gemini_credential_from_oauth,
    normalize_chatgpt_base_url, RelayProvider, RelayProviderCredential,
};
use serde_json::json;

#[test]
fn codex_credential_uses_access_token_and_account_id() {
    let auth = json!({
        "tokens": {
            "access_token": "codex-token",
            "account_id": "acct_123"
        }
    });

    let credential =
        codex_credential_from_auth(&auth, "https://chatgpt.com/backend-api".to_string())
            .expect("codex credential");

    assert_eq!(
        credential,
        RelayProviderCredential {
            provider: RelayProvider::Codex,
            upstream_base_url: "https://chatgpt.com/backend-api".to_string(),
            bearer_token: "codex-token".to_string(),
            extra_headers: vec![("ChatGPT-Account-Id".to_string(), "acct_123".to_string())],
        }
    );
}

#[test]
fn codex_base_url_normalizes_chatgpt_hosts_to_backend_api() {
    assert_eq!(
        normalize_chatgpt_base_url("https://chatgpt.com"),
        "https://chatgpt.com/backend-api"
    );
    assert_eq!(
        normalize_chatgpt_base_url("https://chat.openai.com/"),
        "https://chat.openai.com/backend-api"
    );
    assert_eq!(
        normalize_chatgpt_base_url("https://example.test/custom/"),
        "https://example.test/custom"
    );
}

#[test]
fn claude_credential_uses_oauth_access_token() {
    let snapshot = ClaudeLiveCredentialSnapshot {
        credentials_json: serde_json::to_vec(&json!({
            "claudeAiOauth": {
                "accessToken": "claude-token"
            }
        }))
        .expect("credentials json"),
        oauth_account_json: None,
    };

    let credential = claude_credential_from_snapshot(&snapshot).expect("claude credential");

    assert_eq!(credential.provider, RelayProvider::Claude);
    assert_eq!(credential.upstream_base_url, "https://api.anthropic.com");
    assert_eq!(credential.bearer_token, "claude-token");
    assert!(credential
        .extra_headers
        .iter()
        .any(|(name, value)| name == "anthropic-beta" && value == "oauth-2025-04-20"));
}

#[test]
fn gemini_credential_uses_code_assist_oauth_access_token() {
    let oauth = json!({
        "access_token": "gemini-token"
    });

    let credential = gemini_credential_from_oauth(&oauth).expect("gemini credential");

    assert_eq!(
        credential,
        RelayProviderCredential {
            provider: RelayProvider::Gemini,
            upstream_base_url: "https://cloudcode-pa.googleapis.com".to_string(),
            bearer_token: "gemini-token".to_string(),
            extra_headers: Vec::new(),
        }
    );
}
