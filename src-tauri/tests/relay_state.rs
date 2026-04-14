use ai_accounts_hub_lib::app_settings::models::RelaySettings;
use ai_accounts_hub_lib::relay::state::{RelayRuntimeStatus, RelayServerState};

#[tokio::test]
async fn disabled_relay_status_is_not_running() {
    let state = RelayServerState::default();

    let status = state
        .apply_settings_for_tests(RelaySettings {
            enabled: false,
            port: 8765,
        })
        .await;

    assert_eq!(
        status,
        RelayRuntimeStatus {
            running: false,
            bind_host: "127.0.0.1".to_string(),
            port: 8765,
            last_error: None,
            codex_base_url: "http://127.0.0.1:8765/codex".to_string(),
        }
    );
}

#[tokio::test]
async fn enabled_relay_binds_to_loopback() {
    let state = RelayServerState::default();

    let status = state
        .apply_settings_for_tests(RelaySettings {
            enabled: true,
            port: 0,
        })
        .await;

    assert!(status.running, "{status:?}");
    assert_eq!(status.bind_host, "127.0.0.1");
    assert!(status.port > 0);
    assert!(status.codex_base_url.starts_with("http://127.0.0.1:"));
    assert!(status.last_error.is_none());

    let stopped = state
        .apply_settings_for_tests(RelaySettings {
            enabled: false,
            port: status.port,
        })
        .await;
    assert!(!stopped.running);
}
