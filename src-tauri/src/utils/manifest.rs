use crate::error::{AppError, Result};
use crate::models::AddonManifest;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Parse an ESO addon manifest (.txt file)
pub fn parse_manifest(path: &Path) -> Result<AddonManifest> {
    let content = fs::read_to_string(path)?;
    let mut meta: HashMap<String, String> = HashMap::new();
    let mut files: Vec<String> = Vec::new();

    for line in content.lines() {
        let line = line.trim();

        if line.starts_with("## ") {
            // Metadata line: ## Key: Value
            if let Some(colon_pos) = line.find(':') {
                let key = line[3..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                meta.insert(key, value);
            }
        } else if !line.is_empty() && !line.starts_with(';') && !line.starts_with('#') {
            // File reference (not a comment)
            files.push(line.to_string());
        }
    }

    let title = meta
        .get("title")
        .cloned()
        .ok_or_else(|| AppError::InvalidManifest("Missing ## Title:".into()))?;

    Ok(AddonManifest {
        title,
        api_version: meta.get("apiversion").cloned(),
        author: meta.get("author").cloned(),
        version: meta
            .get("version")
            .cloned()
            .or_else(|| meta.get("addonversion").cloned()),
        description: meta.get("description").cloned(),
        dependencies: parse_dependency_list(meta.get("dependson")),
        optional_dependencies: parse_dependency_list(meta.get("optionaldependson")),
        saved_variables: parse_dependency_list(meta.get("savedvariables")),
        files,
    })
}

/// Parse a space-separated dependency list
fn parse_dependency_list(value: Option<&String>) -> Vec<String> {
    value
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default()
}

/// Find all manifest files in an addon directory
pub fn find_manifests(addon_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut manifests = Vec::new();

    if let Ok(entries) = fs::read_dir(addon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                // Check if it's actually a manifest (has ## Title:)
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("## Title:") {
                        manifests.push(path);
                    }
                }
            }
        }
    }

    manifests
}

/// Get the addon name from a manifest file path
pub fn get_addon_name_from_path(manifest_path: &Path) -> Option<String> {
    manifest_path
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_dependency_list() {
        let deps = Some("LibAddonMenu-2.0 LibStub".to_string());
        let result = parse_dependency_list(deps.as_ref());
        assert_eq!(result, vec!["LibAddonMenu-2.0", "LibStub"]);
    }

    #[test]
    fn test_parse_dependency_list_empty() {
        let result = parse_dependency_list(None);
        assert!(result.is_empty());
    }
}
