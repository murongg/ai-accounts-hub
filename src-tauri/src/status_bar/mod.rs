pub mod bridge_payload;
pub mod menu_model;
pub mod native_bridge;

use std::sync::Mutex;

#[cfg(target_os = "macos")]
use dirs::home_dir;
use tauri::{AppHandle, Manager};

#[cfg(target_os = "macos")]
use crate::claude_accounts::{paths::ClaudeAccountPaths, service::ClaudeAccountService};
#[cfg(target_os = "macos")]
use crate::codex_accounts::{paths::CodexAccountPaths, service::CodexAccountService};
#[cfg(target_os = "macos")]
use crate::codex_usage::scheduler::CodexUsageSchedulerState;
#[cfg(target_os = "macos")]
use crate::gemini_accounts::{paths::GeminiAccountPaths, service::GeminiAccountService};

use self::bridge_payload::StatusBarTab;
#[cfg(target_os = "macos")]
use self::menu_model::{build_provider_menu_state, MenuProvider};
#[cfg(target_os = "macos")]
use self::menu_model::{parse_menu_action, MenuAccountState, MenuAction};

const MAIN_WINDOW_LABEL: &str = "main";

#[cfg(target_os = "macos")]
const STATUS_TRAY_ID: &str = "status-bar";

#[cfg(target_os = "macos")]
const MENU_ID_PROVIDER_CODEX: &str = "provider:codex";
#[cfg(target_os = "macos")]
const MENU_ID_PROVIDER_CLAUDE: &str = "provider:claude";
#[cfg(target_os = "macos")]
const MENU_ID_PROVIDER_GEMINI: &str = "provider:gemini";
#[cfg(target_os = "macos")]
const MENU_ID_REFRESH: &str = "refresh";
#[cfg(target_os = "macos")]
const MENU_ID_OPEN_MAIN: &str = "open-main";
#[cfg(target_os = "macos")]
const MENU_ID_QUIT: &str = "quit";

pub struct StatusBarState {
    #[cfg(target_os = "macos")]
    selected_provider: Mutex<MenuProvider>,
    selected_tab: Mutex<StatusBarTab>,
}

impl Default for StatusBarState {
    fn default() -> Self {
        Self {
            #[cfg(target_os = "macos")]
            selected_provider: Mutex::new(MenuProvider::Codex),
            selected_tab: Mutex::new(StatusBarTab::Overview),
        }
    }
}

impl StatusBarState {
    #[cfg(target_os = "macos")]
    fn selected_provider(&self) -> Result<MenuProvider, String> {
        self.selected_provider
            .lock()
            .map(|value| *value)
            .map_err(|_| "status bar state lock poisoned".to_string())
    }

    #[cfg(target_os = "macos")]
    fn set_selected_provider(&self, provider: MenuProvider) -> Result<(), String> {
        let mut selected_provider = self
            .selected_provider
            .lock()
            .map_err(|_| "status bar state lock poisoned".to_string())?;
        *selected_provider = provider;
        Ok(())
    }

    pub fn selected_tab(&self) -> Result<StatusBarTab, String> {
        self.selected_tab
            .lock()
            .map(|value| *value)
            .map_err(|_| "status bar state lock poisoned".to_string())
    }

    pub fn set_selected_tab(&self, tab: StatusBarTab) -> Result<(), String> {
        let mut selected_tab = self
            .selected_tab
            .lock()
            .map_err(|_| "status bar state lock poisoned".to_string())?;
        *selected_tab = tab;
        Ok(())
    }
}

#[tauri::command]
pub fn show_main_window(app: AppHandle) -> Result<(), String> {
    show_main_window_internal(&app).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn quit_application(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn setup_status_bar<R: tauri::Runtime>(app: &mut tauri::App<R>) -> tauri::Result<()> {
    use tauri::tray::TrayIconBuilder;
    use tauri::ActivationPolicy;

    if let Ok(()) = native_bridge::initialize(app.handle()) {
        app.set_activation_policy(ActivationPolicy::Regular);
        app.set_dock_visibility(true);
        return Ok(());
    }

    let menu = build_status_menu(app.handle()).unwrap_or_else(|error| {
        build_fallback_menu(app.handle(), &error).expect("fallback status menu should build")
    });

    let mut builder = TrayIconBuilder::with_id(STATUS_TRAY_ID)
        .menu(&menu)
        .tooltip("AI Accounts Hub")
        .show_menu_on_left_click(true);

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder.build(app)?;

    app.set_activation_policy(ActivationPolicy::Regular);
    app.set_dock_visibility(true);

    Ok(())
}

#[cfg(not(target_os = "macos"))]
pub fn setup_status_bar<R: tauri::Runtime>(_app: &mut tauri::App<R>) -> tauri::Result<()> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn refresh_status_menu<R: tauri::Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    if native_bridge::is_ready() {
        return native_bridge::update_payload(app);
    }

    let Some(tray) = app.tray_by_id(STATUS_TRAY_ID) else {
        return Ok(());
    };

    let menu = build_status_menu(app)
        .or_else(|error| build_fallback_menu(app, &error))
        .map_err(|error| error.to_string())?;

    tray.set_menu(Some(menu)).map_err(|error| error.to_string())
}

#[cfg(not(target_os = "macos"))]
pub fn refresh_status_menu<R: tauri::Runtime>(_app: &AppHandle<R>) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
pub fn handle_menu_event<R: tauri::Runtime>(app: &AppHandle<R>, event: tauri::menu::MenuEvent) {
    if native_bridge::is_ready() {
        return;
    }

    let Some(action) = parse_menu_action(event.id().as_ref()) else {
        return;
    };

    match action {
        MenuAction::SelectProvider(provider) => {
            if let Err(error) = app
                .state::<StatusBarState>()
                .set_selected_provider(provider)
            {
                eprintln!("failed to update status bar provider: {error}");
                return;
            }

            if let Err(error) = refresh_status_menu(app) {
                eprintln!("failed to refresh status bar menu: {error}");
            }
        }
        MenuAction::SwitchAccount(provider, account_id) => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = switch_provider_account(app.clone(), provider, account_id).await
                {
                    eprintln!("failed to switch status bar account: {error}");
                }

                if let Err(error) = refresh_status_menu(&app) {
                    eprintln!("failed to refresh status bar menu: {error}");
                }
            });
        }
        MenuAction::RefreshSelectedProvider => {
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                let selected_provider = match app.state::<StatusBarState>().selected_provider() {
                    Ok(provider) => provider,
                    Err(error) => {
                        eprintln!("failed to read selected provider: {error}");
                        return;
                    }
                };

                if let Err(error) = refresh_selected_provider(app.clone(), selected_provider).await
                {
                    eprintln!("failed to refresh provider from status bar: {error}");
                }

                if let Err(error) = refresh_status_menu(&app) {
                    eprintln!("failed to refresh status bar menu: {error}");
                }
            });
        }
        MenuAction::OpenMainWindow => {
            if let Err(error) = show_main_window_internal(app) {
                eprintln!("failed to show main window from status bar: {error}");
            }
        }
        MenuAction::Quit => {
            app.exit(0);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn handle_menu_event<R: tauri::Runtime>(_app: &AppHandle<R>, _event: tauri::menu::MenuEvent) {}

#[cfg(target_os = "macos")]
pub fn handle_window_event<R: tauri::Runtime>(
    window: &tauri::Window<R>,
    event: &tauri::WindowEvent,
) {
    use tauri::{ActivationPolicy, WindowEvent};

    if let WindowEvent::CloseRequested { api, .. } = event {
        if window.label() == MAIN_WINDOW_LABEL {
            api.prevent_close();
            let _ = window.hide();
            let app = window.app_handle();
            let _ = app.set_dock_visibility(false);
            let _ = app.set_activation_policy(ActivationPolicy::Accessory);
        }
    }
}

#[cfg(not(target_os = "macos"))]
pub fn handle_window_event<R: tauri::Runtime>(
    _window: &tauri::Window<R>,
    _event: &tauri::WindowEvent,
) {
}

#[cfg(target_os = "macos")]
fn build_status_menu<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<tauri::menu::Menu<R>, String> {
    use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};

    let selected_provider = app.state::<StatusBarState>().selected_provider()?;
    let provider_state = load_provider_menu_state(app, selected_provider)?;
    let menu = Menu::new(app).map_err(|error| error.to_string())?;

    let codex = CheckMenuItem::with_id(
        app,
        MENU_ID_PROVIDER_CODEX,
        "Codex",
        true,
        selected_provider == MenuProvider::Codex,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let gemini = CheckMenuItem::with_id(
        app,
        MENU_ID_PROVIDER_GEMINI,
        "Gemini",
        true,
        selected_provider == MenuProvider::Gemini,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;
    let claude = CheckMenuItem::with_id(
        app,
        MENU_ID_PROVIDER_CLAUDE,
        "Claude",
        true,
        selected_provider == MenuProvider::Claude,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;

    menu.append(&codex).map_err(|error| error.to_string())?;
    menu.append(&claude).map_err(|error| error.to_string())?;
    menu.append(&gemini).map_err(|error| error.to_string())?;
    menu.append(&PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;

    if provider_state.accounts.is_empty() {
        let empty_label = match selected_provider {
            MenuProvider::Codex => "暂无 Codex 账号",
            MenuProvider::Claude => "暂无 Claude 账号",
            MenuProvider::Gemini => "暂无 Gemini 账号",
        };
        let empty_item = MenuItem::new(app, empty_label, false, None::<&str>)
            .map_err(|error| error.to_string())?;
        menu.append(&empty_item)
            .map_err(|error| error.to_string())?;
    } else {
        for account in &provider_state.accounts {
            let submenu = build_account_submenu(app, selected_provider, account)?;
            menu.append(&submenu).map_err(|error| error.to_string())?;
        }
    }

    menu.append(&PredefinedMenuItem::separator(app).map_err(|error| error.to_string())?)
        .map_err(|error| error.to_string())?;

    let refresh = MenuItem::with_id(app, MENU_ID_REFRESH, "刷新当前账号类型", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let open_main = MenuItem::with_id(app, MENU_ID_OPEN_MAIN, "打开主窗口", true, None::<&str>)
        .map_err(|error| error.to_string())?;
    let quit = MenuItem::with_id(
        app,
        MENU_ID_QUIT,
        "退出 AI Accounts Hub",
        true,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;

    menu.append(&refresh).map_err(|error| error.to_string())?;
    menu.append(&open_main).map_err(|error| error.to_string())?;
    menu.append(&quit).map_err(|error| error.to_string())?;

    Ok(menu)
}

#[cfg(target_os = "macos")]
fn build_account_submenu<R: tauri::Runtime>(
    app: &AppHandle<R>,
    provider: MenuProvider,
    account: &MenuAccountState,
) -> Result<tauri::menu::Submenu<R>, String> {
    use tauri::menu::{MenuItem, Submenu};

    let title = build_account_menu_title(account);
    let submenu_id = format!("account:{}:{}", provider_slug(provider), account.id);
    let submenu =
        Submenu::with_id(app, submenu_id, title, true).map_err(|error| error.to_string())?;

    let plan = MenuItem::new(app, format!("套餐: {}", account.plan), false, None::<&str>)
        .map_err(|error| error.to_string())?;
    let status = MenuItem::new(
        app,
        format!("状态: {}", status_display_label(&account.status_label)),
        false,
        None::<&str>,
    )
    .map_err(|error| error.to_string())?;

    submenu.append(&plan).map_err(|error| error.to_string())?;
    submenu.append(&status).map_err(|error| error.to_string())?;

    if let Some(quota_summary) = &account.quota_summary {
        let quota = MenuItem::new(
            app,
            format!("剩余配额: {quota_summary}"),
            false,
            None::<&str>,
        )
        .map_err(|error| error.to_string())?;
        submenu.append(&quota).map_err(|error| error.to_string())?;
    }

    let action = if account.is_active {
        MenuItem::new(app, "当前使用中", false, None::<&str>).map_err(|error| error.to_string())?
    } else {
        MenuItem::with_id(
            app,
            format!("switch:{}:{}", provider_slug(provider), account.id),
            "切换到此账号",
            true,
            None::<&str>,
        )
        .map_err(|error| error.to_string())?
    };

    submenu.append(&action).map_err(|error| error.to_string())?;

    Ok(submenu)
}

#[cfg(target_os = "macos")]
fn build_account_menu_title(account: &MenuAccountState) -> String {
    let prefix = if account.is_active { "● " } else { "" };

    if let Some(quota_summary) = &account.quota_summary {
        format!("{prefix}{} · {quota_summary}", account.email)
    } else if account.status_label == "Re-login required" {
        format!("{prefix}{} · 需要重登", account.email)
    } else {
        format!("{prefix}{}", account.email)
    }
}

#[cfg(target_os = "macos")]
fn build_fallback_menu<R: tauri::Runtime>(
    app: &AppHandle<R>,
    error: &str,
) -> tauri::Result<tauri::menu::Menu<R>> {
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    let menu = Menu::new(app)?;
    let unavailable = MenuItem::new(app, "状态栏数据暂不可用", false, None::<&str>)?;
    let error_item = MenuItem::new(app, truncate_menu_error(error), false, None::<&str>)?;
    let open_main = MenuItem::with_id(app, MENU_ID_OPEN_MAIN, "打开主窗口", true, None::<&str>)?;
    let quit = MenuItem::with_id(
        app,
        MENU_ID_QUIT,
        "退出 AI Accounts Hub",
        true,
        None::<&str>,
    )?;

    menu.append(&unavailable)?;
    menu.append(&error_item)?;
    menu.append(&PredefinedMenuItem::separator(app)?)?;
    menu.append(&open_main)?;
    menu.append(&quit)?;

    Ok(menu)
}

#[cfg(target_os = "macos")]
fn truncate_menu_error(error: &str) -> String {
    let mut chars = error.chars();
    let preview: String = chars.by_ref().take(96).collect();
    if chars.next().is_some() {
        format!("错误: {preview}…")
    } else {
        format!("错误: {preview}")
    }
}

#[cfg(target_os = "macos")]
fn status_display_label(status_label: &str) -> &str {
    match status_label {
        "Re-login required" => "需要重登",
        _ => "凭证正常",
    }
}

#[cfg(target_os = "macos")]
fn provider_slug(provider: MenuProvider) -> &'static str {
    match provider {
        MenuProvider::Codex => "codex",
        MenuProvider::Claude => "claude",
        MenuProvider::Gemini => "gemini",
    }
}

#[cfg(target_os = "macos")]
fn load_provider_menu_state<R: tauri::Runtime>(
    app: &AppHandle<R>,
    selected_provider: MenuProvider,
) -> Result<menu_model::ProviderMenuState, String> {
    let (codex_accounts, claude_accounts, gemini_accounts) = load_account_lists(app)?;

    Ok(build_provider_menu_state(
        selected_provider,
        codex_accounts,
        claude_accounts,
        gemini_accounts,
    ))
}

#[cfg(target_os = "macos")]
fn load_account_lists<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<
    (
        Vec<crate::codex_accounts::models::CodexAccountListItem>,
        Vec<crate::claude_accounts::models::ClaudeAccountListItem>,
        Vec<crate::gemini_accounts::models::GeminiAccountListItem>,
    ),
    String,
> {
    let app_data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
    let user_home = home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;

    let codex_accounts = CodexAccountService::with_process_runner(CodexAccountPaths::from_roots(
        app_data_dir.clone(),
        user_home.clone(),
    ))
    .list_accounts()?;
    let gemini_accounts = GeminiAccountService::with_process_runner(
        GeminiAccountPaths::from_roots(app_data_dir.clone(), user_home.clone()),
    )
    .list_accounts()?;
    let claude_accounts = ClaudeAccountService::with_process_runner(
        ClaudeAccountPaths::from_roots(app_data_dir, user_home),
    )
    .list_accounts()?;

    Ok((codex_accounts, claude_accounts, gemini_accounts))
}

#[cfg(target_os = "macos")]
async fn switch_provider_account<R: tauri::Runtime>(
    app: AppHandle<R>,
    provider: MenuProvider,
    account_id: String,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || match provider {
        MenuProvider::Codex => {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
            let user_home =
                home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;
            CodexAccountService::with_process_runner(CodexAccountPaths::from_roots(
                app_data_dir,
                user_home,
            ))
            .switch_account(&account_id)
        }
        MenuProvider::Gemini => {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
            let user_home =
                home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;
            GeminiAccountService::with_process_runner(GeminiAccountPaths::from_roots(
                app_data_dir,
                user_home,
            ))
            .switch_account(&account_id)
        }
        MenuProvider::Claude => {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| format!("failed to resolve app data dir: {error}"))?;
            let user_home =
                home_dir().ok_or_else(|| "failed to resolve user home dir".to_string())?;
            ClaudeAccountService::with_process_runner(ClaudeAccountPaths::from_roots(
                app_data_dir,
                user_home,
            ))
            .switch_account(&account_id)
        }
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(target_os = "macos")]
async fn refresh_selected_provider<R: tauri::Runtime>(
    app: AppHandle<R>,
    provider: MenuProvider,
) -> Result<(), String> {
    let scheduler = app.state::<CodexUsageSchedulerState>();
    match provider {
        MenuProvider::Codex => scheduler.refresh_codex_now().await,
        MenuProvider::Claude => scheduler.refresh_claude_now().await,
        MenuProvider::Gemini => scheduler.refresh_gemini_now().await,
    }
}

#[cfg(target_os = "macos")]
async fn refresh_provider_for_tab<R: tauri::Runtime>(
    app: AppHandle<R>,
    tab: StatusBarTab,
) -> Result<(), String> {
    let scheduler = app.state::<CodexUsageSchedulerState>();

    match tab {
        StatusBarTab::Overview => scheduler.refresh_all_now().await,
        StatusBarTab::Codex => scheduler.refresh_codex_now().await,
        StatusBarTab::Claude => scheduler.refresh_claude_now().await,
        StatusBarTab::Gemini => scheduler.refresh_gemini_now().await,
    }
}

#[cfg(target_os = "macos")]
fn show_main_window_internal<R: tauri::Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    use tauri::{ActivationPolicy, Manager};

    app.set_dock_visibility(true)?;
    app.set_activation_policy(ActivationPolicy::Regular)?;

    if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        main_window.show()?;
        main_window.unminimize()?;
        main_window.set_focus()?;
    }

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn show_main_window_internal<R: tauri::Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    use tauri::Manager;

    if let Some(main_window) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        main_window.show()?;
        main_window.unminimize()?;
        main_window.set_focus()?;
    }

    Ok(())
}
