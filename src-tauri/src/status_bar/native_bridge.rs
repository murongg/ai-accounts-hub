#[cfg(target_os = "macos")]
use std::ffi::{CStr, CString};
#[cfg(target_os = "macos")]
use std::os::raw::c_char;
#[cfg(target_os = "macos")]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(target_os = "macos")]
use std::sync::OnceLock;

#[cfg(target_os = "macos")]
use serde::Deserialize;
#[cfg(target_os = "macos")]
use tauri::{AppHandle, Manager, Runtime};
#[cfg(target_os = "macos")]
use tokio::sync::mpsc;

#[cfg(target_os = "macos")]
use super::bridge_payload::build_bridge_payload;
use super::bridge_payload::StatusBarTab;
#[cfg(target_os = "macos")]
use super::{load_account_lists, refresh_provider_for_tab, show_main_window_internal};
#[cfg(target_os = "macos")]
use crate::status_bar::menu_model::MenuProvider;

#[cfg(target_os = "macos")]
static BRIDGE_READY: AtomicBool = AtomicBool::new(false);
#[cfg(target_os = "macos")]
static ACTION_SENDER: OnceLock<mpsc::UnboundedSender<String>> = OnceLock::new();

#[cfg(target_os = "macos")]
#[link(name = "aah_status_bar_bridge", kind = "static")]
unsafe extern "C" {
    fn aah_status_bar_bridge_initialize(callback: extern "C" fn(*const c_char)) -> bool;
    fn aah_status_bar_bridge_set_payload(payload_json: *const c_char);
}

#[cfg(target_os = "macos")]
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NativeBridgeAction {
    SelectTab {
        tab: StatusBarTab,
    },
    SwitchAccount {
        provider: String,
        account_id: String,
    },
    Refresh,
    OpenMainWindow,
    Quit,
}

#[cfg(target_os = "macos")]
extern "C" fn native_bridge_callback(message: *const c_char) {
    if message.is_null() {
        return;
    }

    let Ok(message) = unsafe { CStr::from_ptr(message) }.to_str() else {
        return;
    };

    if let Some(sender) = ACTION_SENDER.get() {
        let _ = sender.send(message.to_string());
    }
}

#[cfg(target_os = "macos")]
pub fn is_ready() -> bool {
    BRIDGE_READY.load(Ordering::SeqCst)
}

#[cfg(target_os = "macos")]
pub fn initialize<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if BRIDGE_READY.load(Ordering::SeqCst) {
        return update_payload(app);
    }

    let (sender, mut receiver) = mpsc::unbounded_channel();
    let _ = ACTION_SENDER.set(sender);

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(message) = receiver.recv().await {
            if let Err(error) = handle_native_action(app_handle.clone(), &message).await {
                eprintln!("native status bar bridge action failed: {error}");
            }
        }
    });

    let initialized = unsafe { aah_status_bar_bridge_initialize(native_bridge_callback) };
    if !initialized {
        return Err("failed to initialize macOS native status bar bridge".to_string());
    }

    BRIDGE_READY.store(true, Ordering::SeqCst);
    update_payload(app)
}

#[cfg(target_os = "macos")]
pub fn update_payload<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if !BRIDGE_READY.load(Ordering::SeqCst) {
        return Ok(());
    }

    let (codex_accounts, gemini_accounts) = load_account_lists(app)?;
    let selected_tab = app.state::<super::StatusBarState>().selected_tab()?;
    let visible_tab = visible_native_tab(
        selected_tab,
        !codex_accounts.is_empty(),
        !gemini_accounts.is_empty(),
    );

    if visible_tab != selected_tab {
        app.state::<super::StatusBarState>()
            .set_selected_tab(visible_tab)?;
    }

    let payload = build_bridge_payload(
        visible_tab,
        codex_accounts,
        gemini_accounts,
        current_time_ms(),
    );
    let payload_json = serde_json::to_string(&payload)
        .map_err(|error| format!("failed to serialize native status payload: {error}"))?;
    let payload_cstring = CString::new(payload_json)
        .map_err(|error| format!("failed to encode native status payload: {error}"))?;

    unsafe { aah_status_bar_bridge_set_payload(payload_cstring.as_ptr()) };

    Ok(())
}

#[cfg(target_os = "macos")]
async fn handle_native_action<R: Runtime>(app: AppHandle<R>, message: &str) -> Result<(), String> {
    let action: NativeBridgeAction = serde_json::from_str(message)
        .map_err(|error| format!("failed to parse native bridge action: {error}"))?;

    match action {
        NativeBridgeAction::SelectTab { tab } => {
            app.state::<super::StatusBarState>().set_selected_tab(tab)?;
            update_payload(&app)?;
        }
        NativeBridgeAction::SwitchAccount {
            provider,
            account_id,
        } => {
            let provider = match provider.as_str() {
                "codex" => MenuProvider::Codex,
                "gemini" => MenuProvider::Gemini,
                _ => return Err(format!("unknown provider: {provider}")),
            };

            super::switch_provider_account(app.clone(), provider, account_id).await?;
            refresh_provider_for_tab(
                app.clone(),
                app.state::<super::StatusBarState>().selected_tab()?,
            )
            .await?;
            update_payload(&app)?;
        }
        NativeBridgeAction::Refresh => {
            refresh_provider_for_tab(
                app.clone(),
                app.state::<super::StatusBarState>().selected_tab()?,
            )
            .await?;
            update_payload(&app)?;
        }
        NativeBridgeAction::OpenMainWindow => {
            show_main_window_internal(&app).map_err(|error| error.to_string())?;
        }
        NativeBridgeAction::Quit => {
            app.exit(0);
        }
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn current_time_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as i64)
        .unwrap_or_default()
}

fn visible_native_tab(
    selected_tab: StatusBarTab,
    has_codex: bool,
    has_gemini: bool,
) -> StatusBarTab {
    match selected_tab {
        StatusBarTab::Overview => {
            if has_codex {
                StatusBarTab::Codex
            } else if has_gemini {
                StatusBarTab::Gemini
            } else {
                StatusBarTab::Codex
            }
        }
        explicit_tab => explicit_tab,
    }
}

#[cfg(test)]
mod tests {
    use super::visible_native_tab;
    use super::StatusBarTab;

    #[test]
    fn visible_native_tab_prefers_codex_when_overview_is_hidden() {
        assert_eq!(
            visible_native_tab(StatusBarTab::Overview, true, true),
            StatusBarTab::Codex
        );
    }

    #[test]
    fn visible_native_tab_falls_back_to_gemini_when_codex_is_unavailable() {
        assert_eq!(
            visible_native_tab(StatusBarTab::Overview, false, true),
            StatusBarTab::Gemini
        );
    }

    #[test]
    fn visible_native_tab_keeps_explicit_provider_selection() {
        assert_eq!(
            visible_native_tab(StatusBarTab::Gemini, true, true),
            StatusBarTab::Gemini
        );
    }
}
