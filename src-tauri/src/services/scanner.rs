use crate::error::Result;
use crate::models::AddonManifest;
use crate::utils::manifest::{find_manifests, parse_manifest};
use std::fs;
use std::path::Path;

/// Information about a locally scanned addon
#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScannedAddon {
    pub name: String,
    pub path: String,
    pub manifest: AddonManifest,
    pub has_saved_variables: bool,
}

/// Scan the ESO addon directory for installed addons
pub fn scan_addon_directory(addon_dir: &Path) -> Result<Vec<ScannedAddon>> {
    let mut addons = Vec::new();

    if !addon_dir.exists() {
        return Ok(addons);
    }

    for entry in fs::read_dir(addon_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        // Find manifests in this addon folder
        let manifests = find_manifests(&path);

        for manifest_path in manifests {
            if let Ok(manifest) = parse_manifest(&manifest_path) {
                let saved_vars_path = addon_dir
                    .parent()
                    .map(|p| p.join("SavedVariables"))
                    .and_then(|sv_path| {
                        manifest
                            .saved_variables
                            .first()
                            .map(|sv| sv_path.join(format!("{}.lua", sv)))
                    });

                let has_saved_variables =
                    saved_vars_path.map(|p| p.exists()).unwrap_or(false);

                addons.push(ScannedAddon {
                    name: manifest.title.clone(),
                    path: path.to_string_lossy().to_string(),
                    manifest,
                    has_saved_variables,
                });
            }
        }
    }

    // Sort by name
    addons.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    Ok(addons)
}

/// Check if an addon exists in the addon directory
pub fn addon_exists(addon_dir: &Path, addon_name: &str) -> bool {
    addon_dir.join(addon_name).exists()
}

/// Get the list of addon folder names
pub fn get_addon_folders(addon_dir: &Path) -> Result<Vec<String>> {
    let mut folders = Vec::new();

    if !addon_dir.exists() {
        return Ok(folders);
    }

    for entry in fs::read_dir(addon_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip hidden folders
                if !name.starts_with('.') {
                    folders.push(name.to_string());
                }
            }
        }
    }

    folders.sort();
    Ok(folders)
}
