use crate::error::{AppError, Result};
use crate::models::index::DownloadSource;
use std::path::PathBuf;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Download a file from a URL with progress callback
pub async fn download_file<F>(url: &str, target_path: &PathBuf, on_progress: F) -> Result<()>
where
    F: Fn(f64) + Send + 'static,
{
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "eso-addon-manager")
        .send()
        .await?;

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded: u64 = 0;

    let mut file = File::create(target_path).await?;
    let mut stream = response.bytes_stream();

    use futures_util::StreamExt;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = downloaded as f64 / total_size as f64;
            on_progress(progress);
        }
    }

    file.flush().await?;
    on_progress(1.0);

    Ok(())
}

/// Download from multiple sources with fallback
/// Tries each source in order until one succeeds
/// Prefers github_archive sources since they provide ZIP files directly
/// (jsDelivr serves individual files which requires different handling)
pub async fn download_with_fallback<F>(
    sources: &[DownloadSource],
    fallback_url: Option<&str>,
    target_path: &PathBuf,
    on_progress: F,
) -> Result<()>
where
    F: Fn(f64) + Send + Clone + 'static,
{
    let mut last_error: Option<String> = None;

    // Filter to prefer github_archive sources (they provide ZIP files)
    // jsDelivr serves individual files which we can't easily handle as a ZIP
    let archive_sources: Vec<_> = sources
        .iter()
        .filter(|s| s.source_type == "github_archive")
        .collect();

    // Try archive sources first
    for source in &archive_sources {
        match download_file(&source.url, target_path, on_progress.clone()).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                last_error = Some(format!("{} download failed: {}", source.source_type, e));
            }
        }
    }

    // Try remaining sources (jsdelivr etc) if they provide direct file downloads
    // Note: jsDelivr typically serves individual files, not ZIP archives
    // But some repos may have pre-packaged ZIPs available
    for source in sources.iter().filter(|s| s.source_type != "github_archive") {
        // Only try if URL ends with .zip (pre-packaged archive)
        if source.url.ends_with(".zip") {
            match download_file(&source.url, target_path, on_progress.clone()).await {
                Ok(_) => {
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(format!("{} download failed: {}", source.source_type, e));
                }
            }
        }
    }

    // Fallback to the legacy download_url if provided
    if let Some(url) = fallback_url {
        match download_file(url, target_path, on_progress).await {
            Ok(_) => {
                return Ok(());
            }
            Err(e) => {
                last_error = Some(format!("Fallback download failed: {}", e));
            }
        }
    }

    Err(AppError::Download(last_error.unwrap_or_else(|| {
        "All download sources failed".to_string()
    })))
}

/// Get the best download URL from sources, preferring github_archive
pub fn get_best_download_url(
    sources: &[DownloadSource],
    fallback_url: Option<&str>,
) -> Option<String> {
    // Prefer github_archive sources
    if let Some(source) = sources.iter().find(|s| s.source_type == "github_archive") {
        return Some(source.url.clone());
    }

    // Try any source with a .zip URL
    if let Some(source) = sources.iter().find(|s| s.url.ends_with(".zip")) {
        return Some(source.url.clone());
    }

    // Fall back to legacy URL
    fallback_url.map(|s| s.to_string())
}

/// Get the download URL for a GitHub release asset
pub async fn get_github_release_url(repo: &str) -> Result<Option<String>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let response: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "eso-addon-manager")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?
        .json()
        .await?;

    // Look for a .zip asset
    if let Some(assets) = response.get("assets").and_then(|a| a.as_array()) {
        for asset in assets {
            if let Some(name) = asset.get("name").and_then(|n| n.as_str()) {
                if name.ends_with(".zip") {
                    return Ok(asset
                        .get("browser_download_url")
                        .and_then(|u| u.as_str())
                        .map(String::from));
                }
            }
        }
    }

    // Fallback to zipball URL
    Ok(response
        .get("zipball_url")
        .and_then(|u| u.as_str())
        .map(String::from))
}

/// Get the download URL for a GitHub branch
pub async fn get_github_branch_url(repo: &str, branch: &str) -> String {
    format!(
        "https://github.com/{}/archive/refs/heads/{}.zip",
        repo, branch
    )
}

/// Validate that a GitHub repository exists
pub async fn validate_github_repo(repo: &str) -> Result<bool> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo);

    let response = client
        .get(&url)
        .header("User-Agent", "eso-addon-manager")
        .send()
        .await?;

    Ok(response.status().is_success())
}

/// GitHub release information
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubReleaseInfo {
    pub tag_name: String,
    pub name: Option<String>,
    pub download_url: String,
    pub published_at: Option<String>,
}

/// GitHub branch information
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitHubBranch {
    pub name: String,
    pub is_default: bool,
}

/// List branches for a GitHub repository
pub async fn list_github_branches(repo: &str, default_branch: &str) -> Result<Vec<GitHubBranch>> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.github.com/repos/{}/branches?per_page=100",
        repo
    );

    let response = client
        .get(&url)
        .header("User-Agent", "eso-addon-manager")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(vec![GitHubBranch {
            name: default_branch.to_string(),
            is_default: true,
        }]);
    }

    let data: serde_json::Value = response.json().await?;

    let branches: Vec<GitHubBranch> = data
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|b| {
                    b.get("name")
                        .and_then(|n| n.as_str())
                        .map(|name| GitHubBranch {
                            name: name.to_string(),
                            is_default: name == default_branch,
                        })
                })
                .collect()
        })
        .unwrap_or_else(|| {
            vec![GitHubBranch {
                name: default_branch.to_string(),
                is_default: true,
            }]
        });

    Ok(branches)
}

/// Get the latest release information from a GitHub repository
pub async fn get_github_release_info(repo: &str) -> Result<Option<GitHubReleaseInfo>> {
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}/releases/latest", repo);

    let response = client
        .get(&url)
        .header("User-Agent", "eso-addon-manager")
        .header("Accept", "application/vnd.github.v3+json")
        .send()
        .await?;

    if !response.status().is_success() {
        return Ok(None);
    }

    let data: serde_json::Value = response.json().await?;

    let tag_name = data
        .get("tag_name")
        .and_then(|t| t.as_str())
        .map(String::from);

    let tag_name = match tag_name {
        Some(t) => t,
        None => return Ok(None),
    };

    let name = data.get("name").and_then(|n| n.as_str()).map(String::from);

    let published_at = data
        .get("published_at")
        .and_then(|p| p.as_str())
        .map(String::from);

    // Look for a .zip asset first
    let download_url = if let Some(assets) = data.get("assets").and_then(|a| a.as_array()) {
        assets
            .iter()
            .find(|asset| {
                asset
                    .get("name")
                    .and_then(|n| n.as_str())
                    .map(|n| n.ends_with(".zip"))
                    .unwrap_or(false)
            })
            .and_then(|asset| {
                asset
                    .get("browser_download_url")
                    .and_then(|u| u.as_str())
                    .map(String::from)
            })
    } else {
        None
    };

    // Fallback to zipball URL
    let download_url = download_url.unwrap_or_else(|| {
        data.get("zipball_url")
            .and_then(|u| u.as_str())
            .map(String::from)
            .unwrap_or_else(|| {
                format!(
                    "https://github.com/{}/archive/refs/tags/{}.zip",
                    repo, tag_name
                )
            })
    });

    Ok(Some(GitHubReleaseInfo {
        tag_name,
        name,
        download_url,
        published_at,
    }))
}
