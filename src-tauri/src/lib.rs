pub mod commands;
pub mod error;
pub mod models;
pub mod services;
pub mod state;
pub mod utils;

use services::database;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize database
    let db = database::init_database().expect("Failed to initialize database");
    let app_state = AppState::new(db);

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            // Addon commands
            commands::get_installed_addons,
            commands::install_addon,
            commands::uninstall_addon,
            commands::scan_local_addons,
            commands::check_updates,
            commands::get_addon_directory,
            commands::set_addon_directory,
            commands::resolve_addon_dependencies,
            // GitHub commands
            commands::add_custom_repo,
            commands::get_custom_repos,
            commands::remove_custom_repo,
            commands::get_github_repo_info,
            commands::install_from_github,
            commands::get_github_release,
            // Index commands
            commands::fetch_index,
            commands::get_cached_index,
            commands::get_index_stats,
            // Settings commands
            commands::get_settings,
            commands::update_settings,
            commands::reset_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
