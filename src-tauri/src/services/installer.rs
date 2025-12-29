use crate::error::{AppError, Result};
use crate::models::InstallInfo;
use crate::utils::manifest::find_manifests;
use crate::utils::zip::{extract_archive, extract_archive_with_options, find_addon_root};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Install an addon from a downloaded archive using explicit install info from the index
pub fn install_from_archive_with_info(
    archive_path: &Path,
    addon_dir: &Path,
    install_info: &InstallInfo,
) -> Result<PathBuf> {
    // Target path using the explicit target_folder from install info
    let target_path = addon_dir.join(&install_info.target_folder);

    // Remove existing addon if present
    if target_path.exists() {
        fs::remove_dir_all(&target_path)?;
    }

    // Create target directory
    fs::create_dir_all(&target_path)?;

    // Extract directly to target path with install options (handles extract_path and excludes)
    extract_archive_with_options(archive_path, &target_path, Some(install_info))?;

    // Verify the addon was extracted correctly by checking for manifest
    if !has_addon_content(&target_path) {
        return Err(AppError::InvalidManifest(
            "No addon manifest found after extraction".into(),
        ));
    }

    Ok(target_path)
}

/// Install an addon from a downloaded archive (legacy fallback for custom repos without install info)
pub fn install_from_archive(archive_path: &Path, addon_dir: &Path) -> Result<PathBuf> {
    // Create a temporary directory for extraction
    let temp_dir = TempDir::new()?;

    // Extract the archive
    extract_archive(archive_path, temp_dir.path())?;

    // Find the addon root (may be in a subdirectory)
    let addon_root = find_addon_root(temp_dir.path())
        .ok_or_else(|| AppError::InvalidManifest("No addon manifest found in archive".into()))?;

    // Get the addon name from the manifest filename, not the folder name
    // This handles cases like "WarMask-1.3.0/" containing "WarMask.txt"
    let addon_name = get_addon_name_from_manifest(&addon_root)?;

    // Target path in the ESO addons directory
    let target_path = addon_dir.join(&addon_name);

    // Remove existing addon if present
    if target_path.exists() {
        fs::remove_dir_all(&target_path)?;
    }

    // Copy addon to target directory
    copy_dir_recursive(&addon_root, &target_path)?;

    Ok(target_path)
}

/// Check if a directory contains addon content (manifest file)
fn has_addon_content(dir: &Path) -> bool {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_manifest_ext = path
                .extension()
                .map(|e| e == "txt" || e == "addon")
                .unwrap_or(false);
            if is_manifest_ext {
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("## Title:") {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Uninstall an addon by removing its directory
pub fn uninstall_addon(addon_path: &Path) -> Result<()> {
    if addon_path.exists() {
        fs::remove_dir_all(addon_path)?;
    }
    Ok(())
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Get the manifest file path for an addon
/// ESO addons can use either .txt or .addon extension for manifests
pub fn get_manifest_path(addon_path: &Path) -> Option<PathBuf> {
    let addon_name = addon_path.file_name()?.to_str()?;

    // First try exact match with .txt extension
    let txt_manifest = addon_path.join(format!("{}.txt", addon_name));
    if txt_manifest.exists() {
        return Some(txt_manifest);
    }

    // Try .addon extension
    let addon_manifest = addon_path.join(format!("{}.addon", addon_name));
    if addon_manifest.exists() {
        return Some(addon_manifest);
    }

    // Search for any manifest file (.txt or .addon)
    fs::read_dir(addon_path).ok()?.find_map(|entry| {
        let path = entry.ok()?.path();
        let is_manifest_ext = path
            .extension()
            .map(|e| e == "txt" || e == "addon")
            .unwrap_or(false);
        if is_manifest_ext {
            if let Ok(content) = fs::read_to_string(&path) {
                if content.contains("## Title:") {
                    return Some(path);
                }
            }
        }
        None
    })
}

/// Get the correct addon name from the manifest file in a directory
/// The manifest filename determines the required addon folder name
/// e.g., "WarMask.txt" means the addon must be in a "WarMask" folder
fn get_addon_name_from_manifest(addon_root: &Path) -> Result<String> {
    let mut manifests = find_manifests(addon_root);

    if manifests.is_empty() {
        return Err(AppError::InvalidManifest(
            "No manifest file found in addon".into(),
        ));
    }

    // Sort manifests to prefer "real" addons over examples:
    // 1. Exclude manifests starting with underscore (e.g., _example_addon)
    // 2. Exclude manifests containing "example" in the name
    // 3. Prefer manifests with capitalized names (typical addon naming)
    manifests.sort_by(|a, b| {
        let a_name = a.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let b_name = b.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        let a_is_example = a_name.starts_with('_') || a_name.to_lowercase().contains("example");
        let b_is_example = b_name.starts_with('_') || b_name.to_lowercase().contains("example");

        // Non-examples come first
        match (a_is_example, b_is_example) {
            (true, false) => std::cmp::Ordering::Greater,
            (false, true) => std::cmp::Ordering::Less,
            _ => a_name.cmp(b_name), // Alphabetical as tiebreaker
        }
    });

    let manifest_path = &manifests[0];
    manifest_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from)
        .ok_or_else(|| AppError::InvalidManifest("Invalid manifest filename".into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_manifest_path() {
        let temp = tempdir().unwrap();
        let addon_path = temp.path().join("TestAddon");
        fs::create_dir_all(&addon_path).unwrap();

        let manifest_path = addon_path.join("TestAddon.txt");
        let mut file = File::create(&manifest_path).unwrap();
        writeln!(file, "## Title: Test Addon").unwrap();

        let result = get_manifest_path(&addon_path);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), manifest_path);
    }
}
