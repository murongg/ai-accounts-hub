pub mod credentials;
pub mod proxy;
pub mod state;

use std::sync::Arc;

use aah_core::app_settings::store;
use aah_core::bootstrap;
use aah_core::claude_accounts::paths::ClaudeAccountPaths;
use aah_core::codex_accounts::paths::CodexAccountPaths;
use aah_core::gemini_accounts::paths::GeminiAccountPaths;
use credentials::LiveRelayCredentialSource;
pub use state::RelayRuntimeStatus;
use state::RelayServerState;
use tauri::Manager;

fn relay_context() -> Result<
    (
        std::path::PathBuf,
        CodexAccountPaths,
        ClaudeAccountPaths,
        GeminiAccountPaths,
    ),
    String,
> {
    let managed = bootstrap::bootstrap_managed_root(None, None)?;
    let managed_root = managed.root.clone();
    Ok((
        managed_root,
        CodexAccountPaths::from_roots(managed.root.clone(), managed.user_home.clone()),
        ClaudeAccountPaths::from_roots(managed.root.clone(), managed.user_home.clone()),
        GeminiAccountPaths::from_roots(managed.root, managed.user_home),
    ))
}

async fn apply_saved_settings(app: tauri::AppHandle) -> Result<RelayRuntimeStatus, String> {
    let (managed_root, codex_paths, claude_paths, gemini_paths) = relay_context()?;
    let settings = store::load_app_settings(&codex_paths)?;
    let source = Arc::new(LiveRelayCredentialSource::new(
        codex_paths,
        claude_paths,
        gemini_paths,
    ));
    let state = app.state::<RelayServerState>();
    Ok(state
        .apply_settings(settings.relay, source, &managed_root)
        .await)
}

pub fn initialize_relay_from_app(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(error) = apply_saved_settings(app).await {
            eprintln!("failed to initialize relay service: {error}");
        }
    });
}

pub async fn apply_relay_settings_from_app(
    app: tauri::AppHandle,
) -> Result<RelayRuntimeStatus, String> {
    apply_saved_settings(app).await
}

#[tauri::command]
pub async fn get_relay_status(app: tauri::AppHandle) -> Result<RelayRuntimeStatus, String> {
    let (managed_root, codex_paths, _, _) = relay_context()?;
    let settings = store::load_app_settings(&codex_paths)?;
    let state = app.state::<RelayServerState>();
    Ok(state.status(&settings.relay, &managed_root))
}
