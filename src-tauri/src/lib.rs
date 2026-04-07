use tauri::Manager;

pub mod app_settings;
pub mod codex_accounts;
pub mod codex_usage;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(codex_usage::scheduler::CodexUsageSchedulerState::default())
        .setup(|app| {
            let scheduler = app.state::<codex_usage::scheduler::CodexUsageSchedulerState>();
            codex_usage::initialize_scheduler(&app.handle(), &scheduler)?;
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(tauri::generate_handler![
            greet,
            codex_accounts::list_codex_accounts,
            codex_accounts::start_codex_account_login,
            codex_accounts::switch_codex_account,
            codex_accounts::delete_codex_account,
            app_settings::get_app_settings,
            app_settings::update_app_settings,
            app_settings::get_app_data_directory_info,
            app_settings::reset_app_data_directory,
            app_settings::clear_all_app_data,
            codex_usage::get_codex_refresh_settings,
            codex_usage::update_codex_refresh_settings,
            codex_usage::refresh_codex_usage_now
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
