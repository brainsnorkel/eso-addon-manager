use crate::error::Result;
use crate::models::InstallInfo;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

/// Extract a ZIP archive to the target directory
pub fn extract_archive(archive_path: &Path, target_dir: &Path) -> Result<Vec<String>> {
    extract_archive_with_options(archive_path, target_dir, None)
}

/// Extract a ZIP archive with install options (extract_path and excludes)
pub fn extract_archive_with_options(
    archive_path: &Path,
    target_dir: &Path,
    install_info: Option<&InstallInfo>,
) -> Result<Vec<String>> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let mut extracted_paths = Vec::new();

    // Get exclude patterns and extract path from install info
    let empty_excludes = Vec::new();
    let excludes = install_info.map(|i| &i.excludes).unwrap_or(&empty_excludes);
    let extract_path = install_info.and_then(|i| i.extract_path.as_deref());

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let enclosed = match file.enclosed_name() {
            Some(path) => path.to_path_buf(),
            None => continue,
        };

        // Handle extract_path: adjust the path relative to extract_path if specified
        let adjusted_path = if let Some(extract_prefix) = extract_path {
            // For GitHub archives, the first component is typically "repo-branch/"
            // We need to find the extract_path within that structure
            let components: Vec<_> = enclosed.components().collect();
            if components.len() < 2 {
                continue; // Skip root-level items in the archive
            }

            // Skip the first component (archive root like "repo-main/")
            let inner_path: PathBuf = components[1..].iter().collect();

            // Check if this path starts with our extract_path
            if let Ok(stripped) = inner_path.strip_prefix(extract_prefix) {
                stripped.to_path_buf()
            } else if inner_path.to_string_lossy() == extract_prefix {
                // This is the extract_path directory itself
                PathBuf::new()
            } else {
                continue; // Skip files not under extract_path
            }
        } else {
            // No extract_path: strip only the archive root folder
            let components: Vec<_> = enclosed.components().collect();
            if components.is_empty() {
                continue;
            }
            if components.len() == 1 {
                PathBuf::new() // Archive root directory
            } else {
                components[1..].iter().collect()
            }
        };

        // Skip empty paths (the root directories themselves)
        if adjusted_path.as_os_str().is_empty() && !file.is_dir() {
            continue;
        }

        // Check if any path component matches an exclude pattern
        if should_exclude(&adjusted_path, excludes) {
            continue;
        }

        let outpath = target_dir.join(&adjusted_path);

        if file.is_dir() {
            if !adjusted_path.as_os_str().is_empty() {
                fs::create_dir_all(&outpath)?;
            }
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

/// Check if a path should be excluded based on glob patterns
fn should_exclude(path: &Path, excludes: &[String]) -> bool {
    for component in path.components() {
        let component_str = component.as_os_str().to_string_lossy();
        for pattern in excludes {
            if matches_glob_pattern(&component_str, pattern) {
                return true;
            }
        }
    }
    false
}

/// Simple glob pattern matching for exclude patterns
/// Supports: * (any chars), .* (hidden files), *.ext (extension match)
fn matches_glob_pattern(name: &str, pattern: &str) -> bool {
    // Exact match
    if name == pattern {
        return true;
    }

    // Pattern ".*" matches hidden files/directories (starting with .)
    if pattern == ".*" && name.starts_with('.') {
        return true;
    }

    // Pattern "*.ext" matches files with that extension
    if let Some(ext) = pattern.strip_prefix("*.") {
        if name.ends_with(&format!(".{}", ext)) {
            return true;
        }
    }

    // Pattern "*suffix" matches files ending with suffix
    if let Some(suffix) = pattern.strip_prefix('*') {
        if !suffix.is_empty() && name.ends_with(suffix) {
            return true;
        }
    }

    false
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
            if path.is_dir() && !is_example_dir(&path) {
                if has_manifest(&path) {
                    return Some(path);
                }
                // Check second-level subdirectories (for repos like LibAddonMenu)
                // Skip example directories
                if let Ok(sub_entries) = fs::read_dir(&path) {
                    for sub_entry in sub_entries.flatten() {
                        let sub_path = sub_entry.path();
                        if sub_path.is_dir()
                            && !is_example_dir(&sub_path)
                            && has_manifest(&sub_path)
                        {
                            return Some(sub_path);
                        }
                    }
                }
            }
        }
    }

    None
}

/// Check if a directory looks like an example/test addon that should be skipped
fn is_example_dir(dir: &Path) -> bool {
    if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
        let lower = name.to_lowercase();
        // Skip directories that are clearly examples or tests
        lower.contains("example") || lower.contains("_test") || name.starts_with('_')
    } else {
        false
    }
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
