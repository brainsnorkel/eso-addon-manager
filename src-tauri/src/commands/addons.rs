use crate::models::{DownloadProgress, DownloadStatus, InstalledAddon, SourceType, UpdateInfo};
use crate::services::{database, downloader, installer, scanner};
use crate::state::AppState;
use crate::utils::paths::get_eso_addon_path;
use std::path::PathBuf;
use tauri::{Emitter, State, Window};
use tempfile::NamedTempFile;

/// Get all installed addons
#[tauri::command]
pub async fn get_installed_addons(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledAddon>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_installed(&conn).map_err(|e| e.to_string())
}

/// Install an addon from a download URL
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_addon(
    slug: String,
    name: String,
    version: String,
    download_url: String,
    source_type: Option<String>,
    source_repo: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<InstalledAddon, String> {
    // Emit initial progress
    let _ = window.emit(
        "download-progress",
        DownloadProgress {
            slug: slug.clone(),
            status: DownloadStatus::Downloading,
            progress: 0.0,
            error: None,
        },
    );

    // Create temp file for download
    let temp_file =
        NamedTempFile::new().map_err(|e| format!("Failed to create temp file: {}", e))?;
    let temp_path = temp_file.path().to_path_buf();

    // Download the addon
    let window_clone = window.clone();
    let slug_clone = slug.clone();
    downloader::download_file(&download_url, &temp_path, move |progress| {
        let _ = window_clone.emit(
            "download-progress",
            DownloadProgress {
                slug: slug_clone.clone(),
                status: DownloadStatus::Downloading,
                progress,
                error: None,
            },
        );
    })
    .await
    .map_err(|e| format!("Download failed: {}", e))?;

    // Emit extracting status
    let _ = window.emit(
        "download-progress",
        DownloadProgress {
            slug: slug.clone(),
            status: DownloadStatus::Extracting,
            progress: 0.0,
            error: None,
        },
    );

    // Get ESO addon directory
    let addon_dir =
        get_eso_addon_path().ok_or_else(|| "Could not find ESO addon directory".to_string())?;

    // Install the addon
    let installed_path = installer::install_from_archive(&temp_path, &addon_dir)
        .map_err(|e| format!("Installation failed: {}", e))?;

    // Get manifest path
    let manifest_path = installer::get_manifest_path(&installed_path)
        .ok_or_else(|| "Could not find addon manifest".to_string())?;

    // Update database
    let source = source_type
        .and_then(|s| s.parse().ok())
        .unwrap_or(SourceType::Index);

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let addon = database::insert_installed(
        &conn,
        &slug,
        &name,
        &version,
        source,
        source_repo.as_deref(),
        manifest_path.to_string_lossy().as_ref(),
    )
    .map_err(|e| e.to_string())?;

    // Emit completion
    let _ = window.emit(
        "download-progress",
        DownloadProgress {
            slug: slug.clone(),
            status: DownloadStatus::Complete,
            progress: 1.0,
            error: None,
        },
    );

    Ok(addon)
}

/// Uninstall an addon
#[tauri::command]
pub async fn uninstall_addon(slug: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get addon info
    let addon = database::get_installed_by_slug(&conn, &slug)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Addon not found: {}", slug))?;

    // Get the addon directory from the manifest path
    let manifest_path = PathBuf::from(&addon.manifest_path);
    let addon_dir = manifest_path
        .parent()
        .ok_or_else(|| "Invalid manifest path".to_string())?;

    // Remove addon files
    installer::uninstall_addon(addon_dir).map_err(|e| e.to_string())?;

    // Remove from database
    database::delete_installed(&conn, &slug).map_err(|e| e.to_string())?;

    Ok(())
}

/// Scan local addon directory for untracked addons
#[tauri::command]
pub async fn scan_local_addons() -> Result<Vec<scanner::ScannedAddon>, String> {
    let addon_dir =
        get_eso_addon_path().ok_or_else(|| "Could not find ESO addon directory".to_string())?;

    scanner::scan_addon_directory(&addon_dir).map_err(|e| e.to_string())
}

/// Check for updates for all installed addons
#[tauri::command]
pub async fn check_updates(state: State<'_, AppState>) -> Result<Vec<UpdateInfo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    let installed = database::get_all_installed(&conn).map_err(|e| e.to_string())?;

    // Get cached index
    let index_data = database::get_cached_index(&conn)
        .map_err(|e| e.to_string())?
        .map(|(data, _, _)| data);

    let index: Option<crate::models::AddonIndex> =
        index_data.and_then(|data| serde_json::from_str(&data).ok());

    let mut updates = Vec::new();

    if let Some(index) = index {
        for addon in installed {
            if let Some(index_entry) = index.addons.iter().find(|a| a.slug == addon.slug) {
                if let Some(release) = &index_entry.latest_release {
                    if release.version != addon.installed_version {
                        updates.push(UpdateInfo {
                            slug: addon.slug,
                            name: addon.name,
                            current_version: addon.installed_version,
                            new_version: release.version.clone(),
                            download_url: release.download_url.clone(),
                        });
                    }
                }
            }
        }
    }

    Ok(updates)
}

/// Get the ESO addon directory path
#[tauri::command]
pub async fn get_addon_directory() -> Result<Option<String>, String> {
    Ok(get_eso_addon_path().map(|p| p.to_string_lossy().to_string()))
}

/// Set a custom ESO addon directory path
#[tauri::command]
pub async fn set_addon_directory(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::set_setting(&conn, "eso_addon_path", &path).map_err(|e| e.to_string())
}
