use crate::models::{AddonIndex, IndexAddon, InstallInfo, InstalledAddon};
use std::collections::{HashMap, HashSet};

/// A resolved dependency ready for installation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedDependency {
    /// The addon slug
    pub slug: String,
    /// Display name
    pub name: String,
    /// Version to install
    pub version: String,
    /// Download URL
    pub download_url: String,
    /// Installation info from the index
    pub install_info: InstallInfo,
    /// Depth in the dependency tree (0 = direct dependency)
    pub depth: usize,
}

/// Result of dependency resolution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependencyResult {
    /// Dependencies that can be installed from the index
    pub resolved: Vec<ResolvedDependency>,
    /// Dependencies that are already installed (by slug)
    pub already_installed: Vec<String>,
    /// Dependencies not found in the index (external/unknown)
    pub unresolved: Vec<String>,
}

impl DependencyResult {
    /// Returns true if there are dependencies to install
    pub fn has_dependencies(&self) -> bool {
        !self.resolved.is_empty()
    }

    /// Returns true if there are unresolved dependencies
    pub fn has_unresolved(&self) -> bool {
        !self.unresolved.is_empty()
    }
}

/// Resolve dependencies for an addon
///
/// This function:
/// 1. Looks up the addon in the index
/// 2. Extracts required_dependencies
/// 3. Recursively resolves nested dependencies
/// 4. Filters out already-installed addons
/// 5. Returns a structured result with resolved, already_installed, and unresolved deps
pub fn resolve_dependencies(
    slug: &str,
    index: &AddonIndex,
    installed: &[InstalledAddon],
) -> DependencyResult {
    let mut result = DependencyResult {
        resolved: Vec::new(),
        already_installed: Vec::new(),
        unresolved: Vec::new(),
    };

    // Find the addon in the index
    let Some(addon) = index.addons.iter().find(|a| a.slug == slug) else {
        return result;
    };

    // Build a lookup map for the index
    let index_map: HashMap<&str, &IndexAddon> =
        index.addons.iter().map(|a| (a.slug.as_str(), a)).collect();

    // Build a set of installed addon slugs (lowercase for case-insensitive matching)
    let installed_slugs: HashSet<String> =
        installed.iter().map(|a| a.slug.to_lowercase()).collect();

    // Also track target folder names for matching
    let installed_folders: HashSet<String> = installed
        .iter()
        .filter_map(|a| {
            std::path::PathBuf::from(&a.manifest_path)
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|s| s.to_lowercase())
        })
        .collect();

    // Track visited slugs to detect circular dependencies
    let mut visited: HashSet<String> = HashSet::new();
    visited.insert(slug.to_string());

    // Recursively resolve dependencies
    resolve_recursive(
        &addon.compatibility.required_dependencies,
        0,
        &index_map,
        &installed_slugs,
        &installed_folders,
        &mut visited,
        &mut result,
    );

    // Sort resolved dependencies by depth (deepest first = install first)
    // This ensures dependencies are installed before their dependents
    result.resolved.sort_by(|a, b| b.depth.cmp(&a.depth));

    result
}

/// Recursively resolve dependencies
fn resolve_recursive(
    deps: &[String],
    depth: usize,
    index_map: &HashMap<&str, &IndexAddon>,
    installed_slugs: &HashSet<String>,
    installed_folders: &HashSet<String>,
    visited: &mut HashSet<String>,
    result: &mut DependencyResult,
) {
    for dep_slug in deps {
        let slug_lower = dep_slug.to_lowercase();

        // Skip if already visited (circular dependency protection)
        if visited.contains(&slug_lower) {
            continue;
        }
        visited.insert(slug_lower.clone());

        // Check if already installed
        if is_installed(&slug_lower, installed_slugs, installed_folders) {
            if !result.already_installed.contains(dep_slug) {
                result.already_installed.push(dep_slug.clone());
            }
            continue;
        }

        // Try to find in index
        if let Some(index_addon) = find_in_index(dep_slug, index_map) {
            // Get download URL
            let download_url = index_addon
                .latest_release
                .as_ref()
                .map(|r| r.download_url.clone())
                .or_else(|| {
                    // Fall back to branch download for branch-based addons
                    if index_addon.source.source_type == "github" {
                        Some(format!(
                            "https://api.github.com/repos/{}/zipball/{}",
                            index_addon.source.repo, index_addon.source.branch
                        ))
                    } else {
                        None
                    }
                });

            if let Some(url) = download_url {
                let version = index_addon
                    .latest_release
                    .as_ref()
                    .map(|r| r.version.clone())
                    .unwrap_or_else(|| format!("{}-latest", index_addon.source.branch));

                // Check if we already resolved this dependency
                if !result.resolved.iter().any(|r| r.slug == index_addon.slug) {
                    result.resolved.push(ResolvedDependency {
                        slug: index_addon.slug.clone(),
                        name: index_addon.name.clone(),
                        version,
                        download_url: url,
                        install_info: index_addon.install.clone(),
                        depth,
                    });
                }

                // Recursively resolve this addon's dependencies
                resolve_recursive(
                    &index_addon.compatibility.required_dependencies,
                    depth + 1,
                    index_map,
                    installed_slugs,
                    installed_folders,
                    visited,
                    result,
                );
            } else {
                // Has index entry but no download URL
                if !result.unresolved.contains(dep_slug) {
                    result.unresolved.push(dep_slug.clone());
                }
            }
        } else {
            // Not found in index
            if !result.unresolved.contains(dep_slug) {
                result.unresolved.push(dep_slug.clone());
            }
        }
    }
}

/// Check if an addon is installed by slug or folder name
fn is_installed(
    slug: &str,
    installed_slugs: &HashSet<String>,
    installed_folders: &HashSet<String>,
) -> bool {
    let slug_lower = slug.to_lowercase();

    // Direct slug match
    if installed_slugs.contains(&slug_lower) {
        return true;
    }

    // Folder name match
    if installed_folders.contains(&slug_lower) {
        return true;
    }

    // Strip version suffix and try again (e.g., "libaddonmenu-2.0" -> "libaddonmenu")
    let base_slug = slug_lower
        .split('-')
        .next()
        .unwrap_or(&slug_lower)
        .to_string();

    installed_slugs
        .iter()
        .any(|s| s.starts_with(&base_slug) || base_slug.starts_with(s.as_str()))
        || installed_folders
            .iter()
            .any(|f| f.starts_with(&base_slug) || base_slug.starts_with(f.as_str()))
}

/// Find an addon in the index by slug (case-insensitive with fallbacks)
fn find_in_index<'a>(
    slug: &str,
    index_map: &HashMap<&str, &'a IndexAddon>,
) -> Option<&'a IndexAddon> {
    // Exact match first
    if let Some(addon) = index_map.get(slug) {
        return Some(addon);
    }

    // Case-insensitive match
    let slug_lower = slug.to_lowercase();
    for (key, addon) in index_map {
        if key.to_lowercase() == slug_lower {
            return Some(addon);
        }
    }

    // Try with/without version suffix
    let base_slug = slug_lower.split('-').next().unwrap_or(&slug_lower);

    for (key, addon) in index_map {
        let key_lower = key.to_lowercase();
        let key_base = key_lower.split('-').next().unwrap_or(&key_lower);
        if key_base == base_slug {
            return Some(addon);
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AddonCompatibility, AddonRelease, AddonSource};

    fn create_test_addon(slug: &str, name: &str, deps: Vec<&str>) -> IndexAddon {
        IndexAddon {
            slug: slug.to_string(),
            name: name.to_string(),
            description: "Test addon".to_string(),
            authors: vec!["Author".to_string()],
            license: None,
            tags: vec![],
            url: None,
            source: AddonSource {
                source_type: "github".to_string(),
                repo: "test/repo".to_string(),
                branch: "main".to_string(),
                path: None,
            },
            compatibility: AddonCompatibility {
                api_version: None,
                game_versions: vec![],
                required_dependencies: deps.into_iter().map(String::from).collect(),
                optional_dependencies: vec![],
            },
            install: InstallInfo {
                method: "branch".to_string(),
                extract_path: None,
                target_folder: slug.to_string(),
                excludes: vec![],
            },
            latest_release: Some(AddonRelease {
                version: "1.0.0".to_string(),
                download_url: format!("https://example.com/{}.zip", slug),
                published_at: None,
                file_size: None,
                checksum: None,
                commit_sha: None,
                commit_date: None,
                commit_message: None,
            }),
            version_info: None,
            download_sources: vec![],
        }
    }

    #[test]
    fn test_no_dependencies() {
        let index = AddonIndex {
            version: "1.0".to_string(),
            generated_at: "2024-01-01".to_string(),
            addon_count: 1,
            addons: vec![create_test_addon("test-addon", "Test Addon", vec![])],
            fetched_at: None,
        };

        let result = resolve_dependencies("test-addon", &index, &[]);

        assert!(result.resolved.is_empty());
        assert!(result.already_installed.is_empty());
        assert!(result.unresolved.is_empty());
    }

    #[test]
    fn test_single_dependency() {
        let index = AddonIndex {
            version: "1.0".to_string(),
            generated_at: "2024-01-01".to_string(),
            addon_count: 2,
            addons: vec![
                create_test_addon("test-addon", "Test Addon", vec!["lib-addon"]),
                create_test_addon("lib-addon", "Lib Addon", vec![]),
            ],
            fetched_at: None,
        };

        let result = resolve_dependencies("test-addon", &index, &[]);

        assert_eq!(result.resolved.len(), 1);
        assert_eq!(result.resolved[0].slug, "lib-addon");
        assert!(result.already_installed.is_empty());
        assert!(result.unresolved.is_empty());
    }

    #[test]
    fn test_unresolved_dependency() {
        let index = AddonIndex {
            version: "1.0".to_string(),
            generated_at: "2024-01-01".to_string(),
            addon_count: 1,
            addons: vec![create_test_addon(
                "test-addon",
                "Test Addon",
                vec!["unknown-lib"],
            )],
            fetched_at: None,
        };

        let result = resolve_dependencies("test-addon", &index, &[]);

        assert!(result.resolved.is_empty());
        assert!(result.already_installed.is_empty());
        assert_eq!(result.unresolved.len(), 1);
        assert_eq!(result.unresolved[0], "unknown-lib");
    }
}
