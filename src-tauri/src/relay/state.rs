use std::path::Path;
use std::sync::Arc;

use aah_core::app_settings::models::RelaySettings;
use aah_core::relay::credentials::RelayCredentialSource;
use aah_core::relay::registry::RelayRegistryPaths;
pub use aah_core::relay::RelayRuntimeStatus;
use aah_core::relay::{RelayOwnerKind, RelayServerState as CoreRelayServerState};

#[derive(Default)]
pub struct RelayServerState {
    inner: CoreRelayServerState,
}

impl RelayServerState {
    pub fn status(&self, settings: &RelaySettings, managed_root: &Path) -> RelayRuntimeStatus {
        self.inner.status(
            settings,
            &RelayRegistryPaths::from_managed_root(managed_root),
        )
    }

    pub async fn apply_settings(
        &self,
        settings: RelaySettings,
        credential_source: Arc<dyn RelayCredentialSource>,
        managed_root: &Path,
    ) -> RelayRuntimeStatus {
        self.inner
            .apply_settings(
                settings,
                credential_source,
                &RelayRegistryPaths::from_managed_root(managed_root),
                RelayOwnerKind::Tauri,
            )
            .await
    }

    pub async fn apply_settings_for_tests(&self, settings: RelaySettings) -> RelayRuntimeStatus {
        self.inner.apply_settings_for_tests(settings).await
    }
}
