use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Custom ESO addon directory (if auto-detection fails)
    pub eso_addon_path: Option<PathBuf>,

    /// Whether to check for updates on startup
    pub check_updates_on_startup: bool,

    /// Whether to auto-update addons
    pub auto_update: bool,

    /// Theme preference
    pub theme: Theme,

    /// Index source URL
    pub index_url: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            eso_addon_path: None,
            check_updates_on_startup: true,
            auto_update: false,
            theme: Theme::System,
            index_url: None,
        }
    }
}

/// UI theme preference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    #[default]
    System,
    Light,
    Dark,
}
