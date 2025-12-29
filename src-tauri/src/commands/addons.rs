use crate::models::{
    DownloadProgress, DownloadStatus, InstallInfo, InstalledAddon, SourceType, UpdateInfo,
};
use crate::services::{database, downloader, installer, scanner};
use crate::state::AppState;
use crate::utils::paths::get_eso_addon_path_with_custom;
use crate::utils::version::is_update_available;
use std::path::PathBuf;
use tauri::{Emitter, State, Window};
use tempfile::NamedTempFile;

/// Version tracking info passed from frontend for simplified update detection
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionTracking {
    /// Pre-computed sort key from index for direct integer comparison
    pub version_sort_key: Option<i64>,
    /// Commit SHA for branch-based version tracking
    pub commit_sha: Option<String>,
}

/// Helper to get the ESO addon path, checking database for custom path first
fn get_addon_path_from_state(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let custom_path = database::get_setting(&conn, "eso_addon_path")
        .ok()
        .flatten();
    drop(conn); // Release lock before potentially long operations

    get_eso_addon_path_with_custom(custom_path.as_deref()).ok_or_else(|| {
        "Could not find ESO addon directory. Please set it manually in Settings.".to_string()
    })
}

/// Get all installed addons (from database + auto-discovered from filesystem)
#[tauri::command]
pub async fn get_installed_addons(
    state: State<'_, AppState>,
) -> Result<Vec<InstalledAddon>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;

    // Get addons already in database
    let mut db_addons = database::get_all_installed(&conn).map_err(|e| e.to_string())?;

    // Get custom path setting
    let custom_path = database::get_setting(&conn, "eso_addon_path")
        .ok()
        .flatten();

    // Try to scan the addon directory for untracked addons
    if let Some(addon_dir) = get_eso_addon_path_with_custom(custom_path.as_deref()) {
        if let Ok(scanned) = scanner::scan_addon_directory(&addon_dir) {
            // Create a set of manifest paths already in database for quick lookup
            let db_manifest_paths: std::collections::HashSet<_> =
                db_addons.iter().map(|a| a.manifest_path.clone()).collect();

            // Also track folder names from database addons
            let db_folders: std::collections::HashSet<_> = db_addons
                .iter()
                .filter_map(|a| {
                    PathBuf::from(&a.manifest_path)
                        .parent()
                        .and_then(|p| p.file_name())
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_lowercase())
                })
                .collect();

            for scanned_addon in scanned {
                let scanned_folder = PathBuf::from(&scanned_addon.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|s| s.to_lowercase())
                    .unwrap_or_default();

                // Check if this addon is already tracked
                let manifest_path = addon_dir
                    .join(&scanned_folder)
                    .join(format!("{}.txt", scanned_folder));
                let manifest_str = manifest_path.to_string_lossy().to_string();

                if !db_manifest_paths.contains(&scanned_addon.path)
                    && !db_manifest_paths.contains(&manifest_str)
                    && !db_folders.contains(&scanned_folder)
                {
                    // Auto-import this addon as a local addon
                    let slug = scanned_folder.clone();
                    let version = scanned_addon
                        .manifest
                        .version
                        .clone()
                        .unwrap_or_else(|| "unknown".to_string());

                    if let Ok(addon) = database::insert_installed(
                        &conn,
                        &slug,
                        &scanned_addon.name,
                        &version,
                        SourceType::Local,
                        None,
                        &scanned_addon.path,
                        None, // No version_sort_key for local addons
                        None, // No commit_sha for local addons
                    ) {
                        db_addons.push(addon);
                    }
                }
            }

            // Re-sort by name after adding new addons
            db_addons.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        }
    }

    Ok(db_addons)
}

/// Install an addon from a download URL with optional install info from the index
#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn install_addon(
    slug: String,
    name: String,
    version: String,
    download_url: String,
    source_type: Option<String>,
    source_repo: Option<String>,
    install_info: Option<InstallInfo>,
    version_tracking: Option<VersionTracking>,
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

    // Get ESO addon directory (checks custom path from database first)
    let addon_dir = get_addon_path_from_state(&state)?;

    // Install the addon using install_info if provided (index addons), otherwise fallback to auto-detection
    let installed_path = if let Some(ref info) = install_info {
        installer::install_from_archive_with_info(&temp_path, &addon_dir, info)
            .map_err(|e| format!("Installation failed: {}", e))?
    } else {
        installer::install_from_archive(&temp_path, &addon_dir)
            .map_err(|e| format!("Installation failed: {}", e))?
    };

    // Get manifest path
    let manifest_path = installer::get_manifest_path(&installed_path)
        .ok_or_else(|| "Could not find addon manifest".to_string())?;

    // Update database
    let source = source_type
        .and_then(|s| s.parse().ok())
        .unwrap_or(SourceType::Index);

    // Extract version tracking info
    let (version_sort_key, commit_sha) = version_tracking
        .map(|vt| (vt.version_sort_key, vt.commit_sha))
        .unwrap_or((None, None));

    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let addon = database::insert_installed(
        &conn,
        &slug,
        &name,
        &version,
        source,
        source_repo.as_deref(),
        manifest_path.to_string_lossy().as_ref(),
        version_sort_key,
        commit_sha.as_deref(),
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
pub async fn scan_local_addons(
    state: State<'_, AppState>,
) -> Result<Vec<scanner::ScannedAddon>, String> {
    let addon_dir = get_addon_path_from_state(&state)?;
    scanner::scan_addon_directory(&addon_dir).map_err(|e| e.to_string())
}

/// Check for updates for all installed addons
#[tauri::command]
pub async fn check_updates(state: State<'_, AppState>) -> Result<Vec<UpdateInfo>, String> {
    // Collect all data from database in a separate scope to ensure lock is released
    let (installed, index, custom_repos) = {
        let conn = state.db.lock().map_err(|e| e.to_string())?;

        let installed = database::get_all_installed(&conn).map_err(|e| e.to_string())?;

        // Get cached index for Index source addons
        let index_data = database::get_cached_index(&conn)
            .map_err(|e| e.to_string())?
            .map(|(data, _, _)| data);

        let index: Option<crate::models::AddonIndex> =
            index_data.and_then(|data| serde_json::from_str(&data).ok());

        // Get custom repos for GitHub source addons
        let custom_repos = database::get_all_custom_repos(&conn).unwrap_or_default();

        (installed, index, custom_repos)
    }; // conn is dropped here

    let mut updates = Vec::new();

    for addon in installed {
        match addon.source_type {
            SourceType::Index => {
                // Check against the index
                if let Some(ref index) = index {
                    if let Some(index_entry) = index.addons.iter().find(|a| a.slug == addon.slug) {
                        // First check if there's a release with a newer version
                        if let Some(release) = &index_entry.latest_release {
                            if is_update_available(&addon.installed_version, &release.version) {
                                updates.push(UpdateInfo {
                                    slug: addon.slug.clone(),
                                    name: addon.name.clone(),
                                    current_version: addon.installed_version.clone(),
                                    new_version: release.version.clone(),
                                    download_url: release.download_url.clone(),
                                    source_type: SourceType::Index,
                                    source_repo: Some(index_entry.source.repo.clone()),
                                    install_info: Some(index_entry.install.clone()),
                                });
                            }
                        } else {
                            // No release - check if installed from branch and branch has updates
                            // For branch installs, we can't easily detect updates without commit tracking
                            // Skip for now - could be enhanced with commit SHA tracking later
                        }
                    }
                }
            }
            SourceType::Github => {
                // Check GitHub releases for custom repos
                if let Some(repo) = &addon.source_repo {
                    // Find the custom repo config
                    let custom_repo = custom_repos.iter().find(|r| &r.repo == repo);

                    // Only check release-based repos (branch-based would need commit tracking)
                    if custom_repo
                        .map(|r| r.release_type == crate::models::ReleaseType::Release)
                        .unwrap_or(true)
                    {
                        // Fetch latest release from GitHub
                        if let Ok(Some(release_info)) =
                            downloader::get_github_release_info(repo).await
                        {
                            // Clean up tag name (remove 'v' prefix if present) for comparison
                            let new_version = release_info
                                .tag_name
                                .strip_prefix('v')
                                .unwrap_or(&release_info.tag_name)
                                .to_string();

                            if is_update_available(&addon.installed_version, &new_version) {
                                updates.push(UpdateInfo {
                                    slug: addon.slug.clone(),
                                    name: addon.name.clone(),
                                    current_version: addon.installed_version.clone(),
                                    new_version: new_version.clone(),
                                    download_url: release_info.download_url,
                                    source_type: SourceType::Github,
                                    source_repo: Some(repo.clone()),
                                    install_info: None, // GitHub repos don't have index install info
                                });
                            }
                        }
                    }
                }
            }
            SourceType::Local => {
                // Local addons have no update source - skip
                // Could potentially be enhanced to check if slug matches an index entry
            }
        }
    }

    Ok(updates)
}

/// Get the ESO addon directory path
#[tauri::command]
pub async fn get_addon_directory(state: State<'_, AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let custom_path = database::get_setting(&conn, "eso_addon_path")
        .ok()
        .flatten();
    drop(conn);

    Ok(get_eso_addon_path_with_custom(custom_path.as_deref())
        .map(|p| p.to_string_lossy().to_string()))
}

/// Set a custom ESO addon directory path
#[tauri::command]
pub async fn set_addon_directory(path: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::set_setting(&conn, "eso_addon_path", &path).map_err(|e| e.to_string())
}
