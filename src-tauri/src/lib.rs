use tauri::Manager;

pub mod app_settings;
pub mod claude_accounts;
pub mod claude_usage;
pub mod codex_accounts;
pub mod codex_usage;
pub mod gemini_accounts;
pub mod gemini_usage;
pub mod proxy_env;
pub mod status_bar;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(codex_usage::scheduler::CodexUsageSchedulerState::default())
        .manage(status_bar::StatusBarState::default())
        .on_menu_event(|app, event| status_bar::handle_menu_event(app, event))
        .on_window_event(|window, event| status_bar::handle_window_event(window, event))
        .setup(|app| {
            proxy_env::import_shell_proxy_env_if_missing();
            let scheduler = app.state::<codex_usage::scheduler::CodexUsageSchedulerState>();
            codex_usage::initialize_scheduler(&app.handle(), &scheduler)?;
            status_bar::setup_status_bar(app)?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            claude_accounts::list_claude_accounts,
            claude_accounts::start_claude_account_login,
            claude_accounts::switch_claude_account,
            claude_accounts::delete_claude_account,
            claude_usage::refresh_claude_usage_now,
            codex_accounts::list_codex_accounts,
            codex_accounts::start_codex_account_login,
            codex_accounts::switch_codex_account,
            codex_accounts::delete_codex_account,
            gemini_accounts::list_gemini_accounts,
            gemini_accounts::start_gemini_account_login,
            gemini_accounts::switch_gemini_account,
            gemini_accounts::delete_gemini_account,
            gemini_usage::refresh_gemini_usage_now,
            app_settings::get_app_settings,
            app_settings::update_app_settings,
            app_settings::get_app_data_directory_info,
            app_settings::reset_app_data_directory,
            app_settings::clear_all_app_data,
            codex_usage::get_codex_refresh_settings,
            codex_usage::update_codex_refresh_settings,
            codex_usage::refresh_codex_usage_now,
            status_bar::show_main_window,
            status_bar::quit_application
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
