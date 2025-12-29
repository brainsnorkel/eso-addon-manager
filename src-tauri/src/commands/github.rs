use crate::models::{
    CustomRepo, DownloadProgress, DownloadStatus, InstalledAddon, ReleaseType, SourceType,
};
use crate::services::{database, downloader, installer};
use crate::state::AppState;
use crate::utils::paths::get_eso_addon_path_with_custom;
use std::path::PathBuf;
use tauri::{Emitter, State, Window};
use tempfile::NamedTempFile;

/// Helper to get the ESO addon path, checking database for custom path first
fn get_addon_path_from_state(state: &State<'_, AppState>) -> Result<PathBuf, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let custom_path = database::get_setting(&conn, "eso_addon_path")
        .ok()
        .flatten();
    drop(conn);

    get_eso_addon_path_with_custom(custom_path.as_deref()).ok_or_else(|| {
        "Could not find ESO addon directory. Please set it manually in Settings.".to_string()
    })
}

/// Add a custom GitHub repository to track
#[tauri::command]
pub async fn add_custom_repo(
    repo: String,
    branch: Option<String>,
    release_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<CustomRepo, String> {
    // Validate repo exists
    let exists = downloader::validate_github_repo(&repo)
        .await
        .map_err(|e| e.to_string())?;

    if !exists {
        return Err(format!("Repository not found: {}", repo));
    }

    // Check for releases if release_type is "release"
    let release_type = release_type
        .and_then(|s| s.parse().ok())
        .unwrap_or(ReleaseType::Release);

    if release_type == ReleaseType::Release {
        let release_url = downloader::get_github_release_url(&repo)
            .await
            .map_err(|e| e.to_string())?;

        if release_url.is_none() {
            return Err(format!(
                "No releases found for {}. Try using branch mode instead.",
                repo
            ));
        }
    }

    // Save to database
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::insert_custom_repo(
        &conn,
        &repo,
        branch.as_deref().unwrap_or("main"),
        release_type,
    )
    .map_err(|e| e.to_string())
}

/// Get all custom tracked repositories
#[tauri::command]
pub async fn get_custom_repos(state: State<'_, AppState>) -> Result<Vec<CustomRepo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_custom_repos(&conn).map_err(|e| e.to_string())
}

/// Remove a custom repository
#[tauri::command]
pub async fn remove_custom_repo(repo: String, state: State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::delete_custom_repo(&conn, &repo).map_err(|e| e.to_string())
}

/// Get GitHub repository information
#[tauri::command]
pub async fn get_github_repo_info(repo: String) -> Result<GitHubRepoInfo, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo);

    let response: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "eso-addon-manager")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;

    // Check for error response
    if let Some(message) = response.get("message").and_then(|m| m.as_str()) {
        if message == "Not Found" {
            return Err(format!("Repository not found: {}", repo));
        }
    }

    let name = response
        .get("name")
        .and_then(|n| n.as_str())
        .unwrap_or_default()
        .to_string();

    let description = response
        .get("description")
        .and_then(|d| d.as_str())
        .map(String::from);

    let default_branch = response
        .get("default_branch")
        .and_then(|b| b.as_str())
        .unwrap_or("main")
        .to_string();

    let stars = response
        .get("stargazers_count")
        .and_then(|s| s.as_u64())
        .unwrap_or(0);

    let updated_at = response
        .get("updated_at")
        .and_then(|u| u.as_str())
        .map(String::from);

    // Check for latest release
    let release_url = format!("https://api.github.com/repos/{}/releases/latest", repo);
    let latest_release = client
        .get(&release_url)
        .header("User-Agent", "eso-addon-manager")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await
        .ok()
        .and_then(|r| {
            if r.status().is_success() {
                Some(r)
            } else {
                None
            }
        });

    let has_releases = latest_release.is_some();

    Ok(GitHubRepoInfo {
        name,
        description,
        default_branch,
        stars,
        updated_at,
        has_releases,
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubRepoInfo {
    pub name: String,
    pub description: Option<String>,
    pub default_branch: String,
    pub stars: u64,
    pub updated_at: Option<String>,
    pub has_releases: bool,
}

/// Install an addon from a GitHub repository
#[tauri::command]
pub async fn install_from_github(
    repo: String,
    release_type: Option<String>,
    branch: Option<String>,
    state: State<'_, AppState>,
    window: Window,
) -> Result<InstalledAddon, String> {
    // Generate a slug from the repo name
    let slug = repo
        .split('/')
        .next_back()
        .unwrap_or(&repo)
        .to_lowercase()
        .replace(' ', "-");

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

    let release_type = release_type
        .and_then(|s| s.parse().ok())
        .unwrap_or(ReleaseType::Release);

    // Get download URL and version based on release type
    let (download_url, version) = if release_type == ReleaseType::Release {
        let release_info = downloader::get_github_release_info(&repo)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| format!("No releases found for {}", repo))?;

        (release_info.download_url, release_info.tag_name)
    } else {
        let branch_name = branch.as_deref().unwrap_or("main");
        let url = downloader::get_github_branch_url(&repo, branch_name).await;
        (url, format!("branch:{}", branch_name))
    };

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

    // Install the addon
    let installed_path = installer::install_from_archive(&temp_path, &addon_dir)
        .map_err(|e| format!("Installation failed: {}", e))?;

    // Get manifest path and addon name
    let manifest_path = installer::get_manifest_path(&installed_path)
        .ok_or_else(|| "Could not find addon manifest".to_string())?;

    // Use the installed folder name as the addon name
    let addon_name = installed_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(&slug)
        .to_string();

    // Update database
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let addon = database::insert_installed(
        &conn,
        &slug,
        &addon_name,
        &version,
        SourceType::Github,
        Some(&repo),
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

/// Get release information for a GitHub repository
#[tauri::command]
pub async fn get_github_release(repo: String) -> Result<Option<GitHubReleaseInfo>, String> {
    downloader::get_github_release_info(&repo)
        .await
        .map(|info| {
            info.map(|i| GitHubReleaseInfo {
                tag_name: i.tag_name,
                name: i.name,
                download_url: i.download_url,
                published_at: i.published_at,
            })
        })
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub download_url: String,
    pub published_at: Option<String>,
}
