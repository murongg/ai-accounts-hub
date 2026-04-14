use std::sync::{Arc, Mutex};

use aah_core::app_settings::models::RelaySettings;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

use super::credentials::{EmptyRelayCredentialSource, RelayCredentialSource};
use super::proxy::{build_relay_router, RelayProxyState};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayRuntimeStatus {
    pub running: bool,
    pub bind_host: String,
    pub port: u16,
    pub last_error: Option<String>,
    pub codex_base_url: String,
}

pub struct RelayRuntime {
    port: u16,
    shutdown: Option<oneshot::Sender<()>>,
    handle: JoinHandle<()>,
}

#[derive(Default)]
pub struct RelayServerState {
    runtime: Mutex<Option<RelayRuntime>>,
    last_error: Mutex<Option<String>>,
}

impl RelayServerState {
    pub fn status(&self, settings: &RelaySettings) -> RelayRuntimeStatus {
        let runtime_port = self
            .runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .map(|runtime| runtime.port);
        let port = runtime_port.unwrap_or(settings.port);
        relay_status(runtime_port.is_some(), port, self.last_error())
    }

    pub async fn apply_settings(
        &self,
        settings: RelaySettings,
        credential_source: Arc<dyn RelayCredentialSource>,
    ) -> RelayRuntimeStatus {
        self.stop_current_runtime();
        if !settings.enabled {
            self.set_last_error(None);
            return relay_status(false, settings.port, None);
        }

        match self.start_runtime(settings.port, credential_source).await {
            Ok(port) => {
                self.set_last_error(None);
                relay_status(true, port, None)
            }
            Err(error) => {
                self.set_last_error(Some(error.clone()));
                relay_status(false, settings.port, Some(error))
            }
        }
    }

    pub async fn apply_settings_for_tests(&self, settings: RelaySettings) -> RelayRuntimeStatus {
        let source = Arc::new(EmptyRelayCredentialSource);
        self.apply_settings(settings, source).await
    }

    async fn start_runtime(
        &self,
        port: u16,
        credential_source: Arc<dyn RelayCredentialSource>,
    ) -> Result<u16, String> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|error| format!("failed to bind relay server: {error}"))?;
        let actual_port = listener
            .local_addr()
            .map_err(|error| format!("failed to read relay listener address: {error}"))?
            .port();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let router = build_relay_router(RelayProxyState::new(credential_source));
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
        });
        *self.runtime.lock().expect("relay runtime lock") = Some(RelayRuntime {
            port: actual_port,
            shutdown: Some(shutdown_tx),
            handle,
        });
        Ok(actual_port)
    }

    fn stop_current_runtime(&self) {
        if let Some(mut runtime) = self.runtime.lock().expect("relay runtime lock").take() {
            if let Some(shutdown) = runtime.shutdown.take() {
                let _ = shutdown.send(());
            }
            runtime.handle.abort();
        }
    }

    fn last_error(&self) -> Option<String> {
        self.last_error.lock().expect("relay error lock").clone()
    }

    fn set_last_error(&self, error: Option<String>) {
        *self.last_error.lock().expect("relay error lock") = error;
    }
}

fn relay_status(running: bool, port: u16, last_error: Option<String>) -> RelayRuntimeStatus {
    RelayRuntimeStatus {
        running,
        bind_host: "127.0.0.1".to_string(),
        port,
        last_error,
        codex_base_url: format!("http://127.0.0.1:{port}/codex"),
    }
}
