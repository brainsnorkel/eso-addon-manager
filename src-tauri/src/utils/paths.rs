use directories::{BaseDirs, UserDirs};
use std::path::PathBuf;

/// Get the ESO addon directory for the current platform
pub fn get_eso_addon_path() -> Option<PathBuf> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        let user_dirs = UserDirs::new()?;
        let docs = user_dirs.document_dir()?;
        Some(
            docs.join("Elder Scrolls Online")
                .join("live")
                .join("AddOns"),
        )
    }

    #[cfg(target_os = "linux")]
    {
        let base = BaseDirs::new()?;
        let home = base.home_dir();

        // Steam: ~/.steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns
        let steam_path = home
            .join(".steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns");

        if steam_path.exists() {
            return Some(steam_path);
        }

        // Fallback: check common Lutris paths
        let lutris_path = home
            .join("Games/elder-scrolls-online/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns");

        if lutris_path.exists() {
            return Some(lutris_path);
        }

        // Return None to prompt user for path
        None
    }
}

/// Get the application data directory
pub fn get_app_data_path() -> Option<PathBuf> {
    let base = BaseDirs::new()?;

    #[cfg(target_os = "windows")]
    {
        Some(
            base.data_local_dir()
                .to_path_buf()
                .join("eso-addon-manager"),
        )
    }

    #[cfg(target_os = "macos")]
    {
        Some(base.data_dir().to_path_buf().join("eso-addon-manager"))
    }

    #[cfg(target_os = "linux")]
    {
        Some(base.data_dir().to_path_buf().join("eso-addon-manager"))
    }
}

/// Get the SavedVariables directory
pub fn get_saved_variables_path() -> Option<PathBuf> {
    let addon_path = get_eso_addon_path()?;
    addon_path.parent().map(|p| p.join("SavedVariables"))
}

/// Get the database file path
pub fn get_database_path() -> Option<PathBuf> {
    get_app_data_path().map(|p| p.join("eso-addon-manager.db"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_data_path() {
        let path = get_app_data_path();
        assert!(path.is_some());
    }
}
