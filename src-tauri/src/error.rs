use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),

    #[error("Archive error: {0}")]
    Archive(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Addon not found: {0}")]
    AddonNotFound(String),

    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),

    #[error("ESO directory not found")]
    EsoDirectoryNotFound,

    #[error("Repository not found: {0}")]
    RepoNotFound(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("{0}")]
    Custom(String),
}

impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
