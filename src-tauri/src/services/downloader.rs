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
