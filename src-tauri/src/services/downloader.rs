use crate::error::Result;
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
