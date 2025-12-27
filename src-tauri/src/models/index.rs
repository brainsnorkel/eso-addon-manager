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
    pub category: String,
    pub tags: Vec<String>,
    pub source: AddonSource,
    pub compatibility: AddonCompatibility,
    pub latest_release: Option<AddonRelease>,
}

/// Source repository information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddonSource {
    #[serde(rename = "type")]
    pub source_type: String,
    pub repo: String,
    pub branch: String,
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
}

/// Cached index data stored in database
#[derive(Debug, Clone)]
pub struct IndexCache {
    pub id: i64,
    pub data: String,
    pub fetched_at: String,
    pub etag: Option<String>,
}
