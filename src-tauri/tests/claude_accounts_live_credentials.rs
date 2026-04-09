use ai_accounts_hub_lib::claude_accounts::keychain::{
    ClaudeCredentialBundle, InMemoryClaudeKeychainStore,
};
use ai_accounts_hub_lib::claude_accounts::live_credentials::{
    ClaudeLiveCredentialSnapshot, ClaudeLiveCredentialState,
};

#[test]
fn creates_a_managed_bundle_from_live_claude_credentials() {
    let live = ClaudeLiveCredentialSnapshot {
        credentials_json: br#"{"claudeAiOauth":{"subscriptionType":"pro"}}"#.to_vec(),
        oauth_account_json: Some(
            br#"{"emailAddress":"murong@example.com","displayName":"Murong"}"#.to_vec(),
        ),
    };

    let bundle =
        ClaudeCredentialBundle::from_live_snapshot("murong@example.com", Some("owner-a"), &live);

    assert_eq!(bundle.email, "murong@example.com");
    assert_eq!(bundle.credentials_json, live.credentials_json);
    assert_eq!(bundle.oauth_account_json, live.oauth_account_json);
    assert_eq!(bundle.account_hint.as_deref(), Some("owner-a"));
}

#[test]
fn restoring_a_bundle_overwrites_live_state() {
    let mut live = ClaudeLiveCredentialState::default();
    let bundle = ClaudeCredentialBundle {
        email: "murong@example.com".into(),
        credentials_json: br#"{"claudeAiOauth":{"subscriptionType":"pro"}}"#.to_vec(),
        oauth_account_json: Some(br#"{"emailAddress":"murong@example.com"}"#.to_vec()),
        account_hint: Some("owner-a".into()),
    };

    live.restore(&bundle).expect("restore should succeed");

    assert_eq!(
        live.credentials_json.as_deref(),
        Some(bundle.credentials_json.as_slice())
    );
    assert_eq!(
        live.oauth_account_json.as_deref(),
        bundle.oauth_account_json.as_deref()
    );
}

#[test]
fn managed_keychain_store_round_trips_bundles_by_key() {
    let mut store = InMemoryClaudeKeychainStore::default();
    let bundle = ClaudeCredentialBundle {
        email: "murong@example.com".into(),
        credentials_json: br#"{"claudeAiOauth":{"subscriptionType":"pro"}}"#.to_vec(),
        oauth_account_json: Some(br#"{"emailAddress":"murong@example.com"}"#.to_vec()),
        account_hint: None,
    };

    store.save("bundle-1", &bundle).expect("save");
    let loaded = store.load("bundle-1").expect("load").expect("present");

    assert_eq!(loaded.email, "murong@example.com");
    assert_eq!(
        loaded.oauth_account_json.as_deref(),
        bundle.oauth_account_json.as_deref()
    );
}
