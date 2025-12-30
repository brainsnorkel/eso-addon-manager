use serde::{Deserialize, Serialize};

/// The addon index containing all available addons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonIndex {
    /// Index format version
    pub version: String,
    /// When the index was generated
    pub generated_at: String,
    /// Total number of addons
    pub addon_count: usize,
    /// List of addons
    pub addons: Vec<IndexAddon>,
    /// When we fetched this index (added client-side)
    #[serde(default)]
    pub fetched_at: Option<String>,
}

/// An addon entry from the index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexAddon {
    pub slug: String,
    pub name: String,
    pub description: String,
    pub authors: Vec<String>,
    pub license: Option<String>,
    pub tags: Vec<String>,
    /// Link to addon docs/homepage
    pub url: Option<String>,
    pub source: AddonSource,
    pub compatibility: AddonCompatibility,
    /// Installation instructions
    pub install: InstallInfo,
    pub latest_release: Option<AddonRelease>,
    /// Version metadata for comparison (from index)
    pub version_info: Option<VersionInfo>,
    /// Multiple download sources with jsDelivr CDN as primary and GitHub as fallback
    #[serde(default)]
    pub download_sources: Vec<DownloadSource>,
    /// ISO 8601 timestamp of when the addon was last updated
    #[serde(default)]
    pub last_updated: Option<String>,
}

/// Source repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub repo: String,
    pub branch: String,
    /// Optional path within the repo for monorepo structures
    #[serde(default)]
    pub path: Option<String>,
}

/// Installation instructions for an addon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallInfo {
    /// Installation method: "branch", "github_release", or "github_archive"
    pub method: String,
    /// Path within the archive to extract from (null for root-level addons)
    #[serde(default)]
    pub extract_path: Option<String>,
    /// Target folder name in the ESO AddOns directory
    pub target_folder: String,
    /// Glob patterns for files/directories to exclude
    #[serde(default)]
    pub excludes: Vec<String>,
}

/// Compatibility information for an addon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonCompatibility {
    pub api_version: Option<String>,
    pub game_versions: Vec<String>,
    pub required_dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
}

/// Release information for an addon
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonRelease {
    pub version: String,
    pub download_url: String,
    pub published_at: Option<String>,
    pub file_size: Option<u64>,
    pub checksum: Option<String>,
    /// Commit SHA for the release
    pub commit_sha: Option<String>,
    /// Commit date (for branch-based releases)
    pub commit_date: Option<String>,
    /// Commit message (for branch-based releases)
    pub commit_message: Option<String>,
}

/// A download source for an addon (jsDelivr CDN or GitHub archive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadSource {
    /// Source type: "jsdelivr" or "github_archive"
    #[serde(rename = "type")]
    pub source_type: String,
    /// Download URL
    pub url: String,
    /// Optional note about the source
    #[serde(default)]
    pub note: Option<String>,
}

/// Normalized semantic version components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionNormalized {
    pub major: Option<i32>,
    pub minor: Option<i32>,
    pub patch: Option<i32>,
    pub prerelease: Option<String>,
}

/// Version metadata for comparison (from index)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Parsed semantic version components (null for branch-based releases)
    pub version_normalized: Option<VersionNormalized>,
    /// Pre-computed sort key for direct integer comparison
    pub version_sort_key: Option<i64>,
    /// Whether this is a pre-release version
    pub is_prerelease: Option<bool>,
    /// Release channel: "stable", "prerelease", or "branch"
    pub release_channel: Option<String>,
    /// Commit message (for branch-based releases)
    pub commit_message: Option<String>,
}

/// Cached index data stored in database
#[derive(Debug, Clone)]
pub struct IndexCache {
    pub id: i64,
    pub data: String,
    pub fetched_at: String,
    pub etag: Option<String>,
}
