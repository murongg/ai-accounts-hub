use std::sync::{Arc, Mutex};
use std::{process, time::Duration};

use crate::app_settings::models::RelaySettings;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::credentials::{EmptyRelayCredentialSource, RelayCredentialSource};
use super::proxy::{build_relay_router, RelayAdminControl, RelayProxyState};
use super::registry::{
    remove_runtime_record, save_runtime_record, shared_runtime_status, shared_runtime_status_async,
    stop_shared_runtime, stop_shared_runtime_async, RelayRegistryPaths, RelayRuntimeRecord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RelayOwnerKind {
    Cli,
    Tauri,
}

impl RelayOwnerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cli => "cli",
            Self::Tauri => "tauri",
        }
    }
}

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
    shutdown: Arc<Mutex<Option<oneshot::Sender<()>>>>,
    handle: JoinHandle<()>,
    registry_paths: RelayRegistryPaths,
}

#[derive(Default)]
pub struct RelayServerState {
    runtime: Mutex<Option<RelayRuntime>>,
    last_error: Mutex<Option<String>>,
}

impl RelayServerState {
    pub fn status(
        &self,
        settings: &RelaySettings,
        registry_paths: &RelayRegistryPaths,
    ) -> RelayRuntimeStatus {
        let runtime_port = self
            .runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .map(|runtime| runtime.port);
        match runtime_port {
            Some(port) => relay_status(true, port, self.last_error()),
            None => shared_runtime_status(settings, registry_paths, self.last_error()),
        }
    }

    pub async fn status_async(
        &self,
        settings: &RelaySettings,
        registry_paths: &RelayRegistryPaths,
    ) -> RelayRuntimeStatus {
        let runtime_port = self
            .runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .map(|runtime| runtime.port);
        match runtime_port {
            Some(port) => relay_status(true, port, self.last_error()),
            None => shared_runtime_status_async(settings, registry_paths, self.last_error()).await,
        }
    }

    pub async fn apply_settings(
        &self,
        settings: RelaySettings,
        credential_source: Arc<dyn RelayCredentialSource>,
        registry_paths: &RelayRegistryPaths,
        owner_kind: RelayOwnerKind,
    ) -> RelayRuntimeStatus {
        self.stop_current_runtime_async().await;
        if !settings.enabled {
            match stop_shared_runtime_async(registry_paths).await {
                Ok(_) => self.set_last_error(None),
                Err(error) => self.set_last_error(Some(error)),
            }
            return relay_status(false, settings.port, None);
        }

        let shared_status = shared_runtime_status_async(&settings, registry_paths, None).await;
        if shared_status.running {
            self.set_last_error(None);
            return shared_status;
        }

        match self
            .start_runtime(settings.port, credential_source, registry_paths, owner_kind)
            .await
        {
            Ok(port) => {
                self.set_last_error(None);
                relay_status(true, port, None)
            }
            Err(error) => {
                let shared_status =
                    shared_runtime_status_async(&settings, registry_paths, None).await;
                if shared_status.running {
                    self.set_last_error(None);
                    shared_status
                } else {
                    self.set_last_error(Some(error.clone()));
                    relay_status(false, settings.port, Some(error))
                }
            }
        }
    }

    pub async fn apply_settings_for_tests(&self, settings: RelaySettings) -> RelayRuntimeStatus {
        let source = Arc::new(EmptyRelayCredentialSource);
        let registry_paths = RelayRegistryPaths::from_managed_root(
            &std::env::temp_dir().join(format!("aah-relay-tests-{}", Uuid::new_v4())),
        );
        self.apply_settings(settings, source, &registry_paths, RelayOwnerKind::Cli)
            .await
    }

    async fn start_runtime(
        &self,
        port: u16,
        credential_source: Arc<dyn RelayCredentialSource>,
        registry_paths: &RelayRegistryPaths,
        owner_kind: RelayOwnerKind,
    ) -> Result<u16, String> {
        let listener = tokio::net::TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|error| format!("failed to bind relay server: {error}"))?;
        let actual_port = listener
            .local_addr()
            .map_err(|error| format!("failed to read relay listener address: {error}"))?
            .port();
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        let shutdown = Arc::new(Mutex::new(Some(shutdown_tx)));
        let admin_token = Uuid::new_v4().to_string();
        let router = build_relay_router(
            RelayProxyState::new(credential_source).with_admin_control(RelayAdminControl::new(
                owner_kind,
                actual_port,
                admin_token.clone(),
                shutdown.clone(),
            )),
        );
        save_runtime_record(
            registry_paths,
            &RelayRuntimeRecord {
                owner_kind,
                pid: process::id(),
                port: actual_port,
                admin_token,
            },
        )?;
        let spawned_registry_paths = registry_paths.clone();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = shutdown_rx.await;
                })
                .await;
            let _ = remove_runtime_record(&spawned_registry_paths);
        });
        *self.runtime.lock().expect("relay runtime lock") = Some(RelayRuntime {
            port: actual_port,
            shutdown,
            handle,
            registry_paths: registry_paths.clone(),
        });
        Ok(actual_port)
    }

    async fn stop_current_runtime_async(&self) {
        let runtime = { self.runtime.lock().expect("relay runtime lock").take() };
        if let Some(runtime) = runtime {
            let shutdown = {
                runtime
                    .shutdown
                    .lock()
                    .ok()
                    .and_then(|mut shutdown| shutdown.take())
            };
            if let Some(shutdown) = shutdown {
                let _ = shutdown.send(());
            }
            // Give the server task a chance to complete graceful shutdown so the
            // listening socket is released before we report "stopped".
            let deadline = tokio::time::Instant::now() + Duration::from_millis(500);
            while !runtime.handle.is_finished() && tokio::time::Instant::now() < deadline {
                tokio::task::yield_now().await;
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
            if !runtime.handle.is_finished() {
                runtime.handle.abort();
                let abort_deadline = tokio::time::Instant::now() + Duration::from_millis(250);
                while !runtime.handle.is_finished() && tokio::time::Instant::now() < abort_deadline
                {
                    tokio::task::yield_now().await;
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
            }
            let _ = remove_runtime_record(&runtime.registry_paths);
        }
    }

    fn stop_current_runtime(&self, _registry_paths: &RelayRegistryPaths) {
        if let Some(runtime) = self.runtime.lock().expect("relay runtime lock").take() {
            if let Ok(mut shutdown) = runtime.shutdown.lock() {
                if let Some(shutdown) = shutdown.take() {
                    let _ = shutdown.send(());
                }
            }
            runtime.handle.abort();
            let _ = remove_runtime_record(&runtime.registry_paths);
        }
    }

    pub fn wait_for_runtime_shutdown(&self, timeout: Duration) {
        let handle = self
            .runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .map(|runtime| runtime.handle.abort_handle());
        if let Some(handle) = handle {
            let deadline = std::time::Instant::now() + timeout;
            while !handle.is_finished() && std::time::Instant::now() < deadline {
                std::thread::sleep(Duration::from_millis(25));
            }
        }
    }

    pub fn is_local_runtime_running(&self) -> bool {
        self.runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .is_some()
    }

    pub fn stop_shared_runtime(
        &self,
        registry_paths: &RelayRegistryPaths,
        settings: &RelaySettings,
    ) -> RelayRuntimeStatus {
        self.stop_current_runtime(registry_paths);
        match stop_shared_runtime(registry_paths) {
            Ok(_) => {
                self.set_last_error(None);
                relay_status(false, settings.port, None)
            }
            Err(error) => {
                self.set_last_error(Some(error.clone()));
                relay_status(false, settings.port, Some(error))
            }
        }
    }

    pub fn shared_status(
        &self,
        settings: &RelaySettings,
        registry_paths: &RelayRegistryPaths,
    ) -> RelayRuntimeStatus {
        self.status(settings, registry_paths)
    }

    pub fn local_runtime_port(&self) -> Option<u16> {
        self.runtime
            .lock()
            .expect("relay runtime lock")
            .as_ref()
            .map(|runtime| runtime.port)
    }

    pub fn trigger_local_shutdown(&self) {
        if let Some(runtime) = self.runtime.lock().expect("relay runtime lock").as_ref() {
            if let Ok(mut shutdown) = runtime.shutdown.lock() {
                if let Some(shutdown) = shutdown.take() {
                    let _ = shutdown.send(());
                }
            }
        }
    }

    pub fn local_last_error(&self) -> Option<String> {
        self.last_error()
    }

    pub fn set_local_last_error(&self, error: Option<String>) {
        self.set_last_error(error);
    }

    pub fn stop_and_report(
        &self,
        registry_paths: &RelayRegistryPaths,
        settings: &RelaySettings,
    ) -> RelayRuntimeStatus {
        self.stop_current_runtime(registry_paths);
        relay_status(false, settings.port, None)
    }

    pub fn stop_remote_and_report(
        &self,
        registry_paths: &RelayRegistryPaths,
        settings: &RelaySettings,
    ) -> RelayRuntimeStatus {
        match stop_shared_runtime(registry_paths) {
            Ok(_) => relay_status(false, settings.port, None),
            Err(error) => relay_status(false, settings.port, Some(error)),
        }
    }
}

impl Drop for RelayServerState {
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.get_mut().expect("relay runtime lock").take() {
            if let Ok(mut shutdown) = runtime.shutdown.lock() {
                if let Some(shutdown) = shutdown.take() {
                    let _ = shutdown.send(());
                }
            }
            let _ = remove_runtime_record(&runtime.registry_paths);
            runtime.handle.abort();
        }
    }
}

impl RelayServerState {
    fn last_error(&self) -> Option<String> {
        self.last_error.lock().expect("relay error lock").clone()
    }

    fn set_last_error(&self, error: Option<String>) {
        *self.last_error.lock().expect("relay error lock") = error;
    }
}

impl RelayRuntimeStatus {
    pub fn stopped(port: u16) -> Self {
        relay_status(false, port, None)
    }
}

pub fn relay_status(running: bool, port: u16, last_error: Option<String>) -> RelayRuntimeStatus {
    RelayRuntimeStatus {
        running,
        bind_host: "127.0.0.1".to_string(),
        port,
        last_error,
        codex_base_url: format!("http://127.0.0.1:{port}/codex"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    fn reserve_local_port() -> u16 {
        TcpListener::bind(("127.0.0.1", 0))
            .expect("reserve local port")
            .local_addr()
            .expect("local addr")
            .port()
    }

    #[tokio::test]
    async fn disabling_local_runtime_releases_bound_port_before_returning() {
        let state = RelayServerState::default();
        let port = reserve_local_port();
        let registry_paths = RelayRegistryPaths::from_managed_root(
            &std::env::temp_dir().join(Uuid::new_v4().to_string()),
        );
        let source = Arc::new(EmptyRelayCredentialSource);

        let started = state
            .apply_settings(
                RelaySettings {
                    enabled: true,
                    port,
                },
                source.clone(),
                &registry_paths,
                RelayOwnerKind::Cli,
            )
            .await;
        assert!(started.running, "{started:?}");

        let stopped = state
            .apply_settings(
                RelaySettings {
                    enabled: false,
                    port,
                },
                source,
                &registry_paths,
                RelayOwnerKind::Cli,
            )
            .await;
        assert!(!stopped.running, "{stopped:?}");

        TcpListener::bind(("127.0.0.1", port))
            .expect("port should be released after disabling relay");
    }
}
