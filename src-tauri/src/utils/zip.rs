use crate::error::Result;
use std::fs::{self, File};
use std::io;
use std::path::Path;

/// Extract a ZIP archive to the target directory
pub fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<Vec<String>> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut extracted_paths = Vec::new();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }

        // Set permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
            }
        }

        if let Some(path_str) = outpath.to_str() {
            extracted_paths.push(path_str.to_string());
        }
    }

    Ok(extracted_paths)
}

/// Find the root addon directory inside an extracted archive
/// Some archives have the addon in a subdirectory (up to 2 levels deep)
pub fn find_addon_root(extracted_dir: &Path) -> Option<std::path::PathBuf> {
    // First, check if there's a manifest directly in the extracted dir
    if has_manifest(extracted_dir) {
        return Some(extracted_dir.to_path_buf());
    }

    // Check first-level subdirectories
    if let Ok(entries) = fs::read_dir(extracted_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if has_manifest(&path) {
                    return Some(path);
                }
                // Check second-level subdirectories (for repos like LibAddonMenu)
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir() && has_manifest(&sub_path) {
                            return Some(sub_path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Check if a directory contains an addon manifest
/// ESO addons can use either .txt or .addon extension for manifests
fn has_manifest(dir: &Path) -> bool {
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
