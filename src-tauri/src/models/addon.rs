use serde::{Deserialize, Serialize};

/// Represents an addon installed on the local system
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledAddon {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub installed_version: String,
    pub source_type: SourceType,
    pub source_repo: Option<String>,
    pub installed_at: String,
    pub updated_at: String,
    pub auto_update: bool,
    pub manifest_path: String,
}

/// Source type for an installed addon
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Index,
    Github,
    Local,
}

impl std::fmt::Display for SourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SourceType::Index => write!(f, "index"),
            SourceType::Github => write!(f, "github"),
            SourceType::Local => write!(f, "local"),
        }
    }
}

impl std::str::FromStr for SourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "index" => Ok(SourceType::Index),
            "github" => Ok(SourceType::Github),
            "local" => Ok(SourceType::Local),
            _ => Err(format!("Unknown source type: {}", s)),
        }
    }
}

/// Custom GitHub repository tracked by the manager
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomRepo {
    pub id: i64,
    pub repo: String,
    pub branch: String,
    pub release_type: ReleaseType,
    pub added_at: String,
    pub last_checked: Option<String>,
}

/// Type of release to track from GitHub
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseType {
    Release,
    Branch,
}

impl std::fmt::Display for ReleaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReleaseType::Release => write!(f, "release"),
            ReleaseType::Branch => write!(f, "branch"),
        }
    }
}

impl std::str::FromStr for ReleaseType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "release" => Ok(ReleaseType::Release),
            "branch" => Ok(ReleaseType::Branch),
            _ => Err(format!("Unknown release type: {}", s)),
        }
    }
}

/// Parsed addon manifest from .txt file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddonManifest {
    pub title: String,
    pub api_version: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub saved_variables: Vec<String>,
    pub files: Vec<String>,
}

/// Information about an available update
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub slug: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub download_url: String,
    /// Source type for the addon (index, github, local)
    pub source_type: SourceType,
    /// Repository for GitHub addons
    pub source_repo: Option<String>,
    /// Installation info for proper extraction (from index)
    pub install_info: Option<super::InstallInfo>,
}

/// Download progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub slug: String,
    pub status: DownloadStatus,
    pub progress: f64,
    pub error: Option<String>,
}

/// Status of a download operation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Extracting,
    Complete,
    Failed,
}
