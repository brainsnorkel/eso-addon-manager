use crate::models::{CustomRepo, ReleaseType};
use crate::services::{database, downloader};
use crate::state::AppState;
use tauri::State;

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
