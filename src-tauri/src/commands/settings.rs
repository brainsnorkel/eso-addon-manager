use crate::models::{AppSettings, Theme};
use crate::services::database;
use crate::state::AppState;
use tauri::State;

/// Get application settings
#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let eso_addon_path = database::get_setting(&conn, "eso_addon_path")
        .ok()
        .flatten()
        .map(std::path::PathBuf::from);

    let check_updates_on_startup = database::get_setting(&conn, "check_updates_on_startup")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(true);

    let auto_update = database::get_setting(&conn, "auto_update")
        .ok()
        .flatten()
        .map(|v| v == "true")
        .unwrap_or(false);

    let theme = database::get_setting(&conn, "theme")
        .ok()
        .flatten()
        .and_then(|v| match v.as_str() {
            "light" => Some(Theme::Light),
            "dark" => Some(Theme::Dark),
            _ => Some(Theme::System),
        })
        .unwrap_or(Theme::System);

    let index_url = database::get_setting(&conn, "index_url").ok().flatten();

    Ok(AppSettings {
        eso_addon_path,
        check_updates_on_startup,
        auto_update,
        theme,
        index_url,
    })
}

/// Update application settings
#[tauri::command]
pub async fn update_settings(settings: AppSettings, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    if let Some(path) = &settings.eso_addon_path {
        database::set_setting(&conn, "eso_addon_path", &path.to_string_lossy())
            .map_err(|e| e.to_string())?;
    }

    database::set_setting(
        &conn,
        "check_updates_on_startup",
        if settings.check_updates_on_startup {
            "true"
        } else {
            "false"
        },
    )
    .map_err(|e| e.to_string())?;

    database::set_setting(
        &conn,
        "auto_update",
        if settings.auto_update { "true" } else { "false" },
    )
    .map_err(|e| e.to_string())?;

    let theme_str = match settings.theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        Theme::System => "system",
    };
    database::set_setting(&conn, "theme", theme_str).map_err(|e| e.to_string())?;

    if let Some(url) = &settings.index_url {
        database::set_setting(&conn, "index_url", url).map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Reset all settings to defaults
#[tauri::command]
pub async fn reset_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Clear all settings
    conn.execute("DELETE FROM settings", [])
        .map_err(|e| e.to_string())?;

    Ok(AppSettings::default())
}
