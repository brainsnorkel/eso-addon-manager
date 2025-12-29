/// Version parsing and comparison utilities for addon version strings
///
/// Handles various version formats:
/// - Semantic versions: "1.2.3", "1.2.3-beta"
/// - Prefixed versions: "v1.2.3", "r32"
/// - Simple versions: "1.2", "1"
/// - Date versions: "2024.01.15"
/// - Branch versions: "main-latest" (treated as always outdated)

use std::cmp::Ordering;

/// Parsed version for comparison
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    /// Numeric components (e.g., [1, 2, 3] for "1.2.3")
    pub components: Vec<u32>,
    /// Pre-release suffix (e.g., "beta", "alpha", "rc1")
    pub prerelease: Option<String>,
    /// Original string for display
    pub original: String,
    /// Whether this is a branch-based "version" (not a real version)
    pub is_branch: bool,
}

impl Version {
    /// Parse a version string into a Version struct
    pub fn parse(version_str: &str) -> Self {
        let original = version_str.to_string();
        let trimmed = version_str.trim();

        // Check for branch-based versions (e.g., "main-latest", "master-latest")
        if trimmed.ends_with("-latest") || trimmed.contains("-branch") {
            return Version {
                components: vec![],
                prerelease: None,
                original,
                is_branch: true,
            };
        }

        // Strip common prefixes
        let cleaned = trimmed
            .strip_prefix('v')
            .or_else(|| trimmed.strip_prefix('V'))
            .or_else(|| trimmed.strip_prefix('r'))
            .or_else(|| trimmed.strip_prefix('R'))
            .unwrap_or(trimmed);

        // Split into version and prerelease parts
        let (version_part, prerelease) = if let Some(idx) = cleaned.find('-') {
            let (v, p) = cleaned.split_at(idx);
            (v, Some(p[1..].to_string()))
        } else if let Some(idx) = cleaned.find('+') {
            // Handle build metadata like "1.2.3+build123"
            let (v, _) = cleaned.split_at(idx);
            (v, None)
        } else {
            (cleaned, None)
        };

        // Parse numeric components
        let components: Vec<u32> = version_part
            .split('.')
            .filter_map(|s| s.parse::<u32>().ok())
            .collect();

        Version {
            components,
            prerelease,
            original,
            is_branch: false,
        }
    }

    /// Check if this version is newer than another
    pub fn is_newer_than(&self, other: &Version) -> bool {
        matches!(self.cmp(other), Ordering::Greater)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        // Branch versions are always considered "older" than real versions
        // This means any real version will trigger an update for branch-installed addons
        match (self.is_branch, other.is_branch) {
            (true, true) => Ordering::Equal,
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            (false, false) => {}
        }

        // Compare numeric components
        let max_len = self.components.len().max(other.components.len());
        for i in 0..max_len {
            let a = self.components.get(i).copied().unwrap_or(0);
            let b = other.components.get(i).copied().unwrap_or(0);
            match a.cmp(&b) {
                Ordering::Equal => continue,
                other => return other,
            }
        }

        // If numeric parts are equal, compare prerelease
        // No prerelease > has prerelease (e.g., "1.0.0" > "1.0.0-beta")
        match (&self.prerelease, &other.prerelease) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
            (None, None) => Ordering::Equal,
        }
    }
}

/// Compare two version strings and determine if the new version is an update
pub fn is_update_available(installed: &str, available: &str) -> bool {
    let installed_ver = Version::parse(installed);
    let available_ver = Version::parse(available);

    available_ver.is_newer_than(&installed_ver)
}

/// Normalize a version string for display
pub fn normalize_version(version: &str) -> String {
    let v = Version::parse(version);
    if v.is_branch {
        return version.to_string();
    }

    if v.components.is_empty() {
        return version.to_string();
    }

    let base = v
        .components
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(".");

    match v.prerelease {
        Some(pre) => format!("{}-{}", base, pre),
        None => base,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_version() {
        let v = Version::parse("1.2.3");
        assert_eq!(v.components, vec![1, 2, 3]);
        assert_eq!(v.prerelease, None);
        assert!(!v.is_branch);
    }

    #[test]
    fn test_parse_prefixed_version() {
        let v = Version::parse("v2.0.1");
        assert_eq!(v.components, vec![2, 0, 1]);

        let v = Version::parse("r32");
        assert_eq!(v.components, vec![32]);
    }

    #[test]
    fn test_parse_prerelease() {
        let v = Version::parse("1.0.0-beta");
        assert_eq!(v.components, vec![1, 0, 0]);
        assert_eq!(v.prerelease, Some("beta".to_string()));
    }

    #[test]
    fn test_parse_branch_version() {
        let v = Version::parse("main-latest");
        assert!(v.is_branch);
        assert!(v.components.is_empty());
    }

    #[test]
    fn test_version_comparison() {
        assert!(is_update_available("1.0.0", "1.0.1"));
        assert!(is_update_available("1.0.0", "1.1.0"));
        assert!(is_update_available("1.0.0", "2.0.0"));
        assert!(!is_update_available("1.0.1", "1.0.0"));
        assert!(!is_update_available("1.0.0", "1.0.0"));
    }

    #[test]
    fn test_version_comparison_different_lengths() {
        assert!(is_update_available("1.0", "1.0.1"));
        assert!(is_update_available("1", "1.0.1"));
        assert!(!is_update_available("1.0.1", "1.0"));
    }

    #[test]
    fn test_version_comparison_prefixes() {
        assert!(is_update_available("v1.0.0", "v1.0.1"));
        assert!(is_update_available("r31", "r32"));
        assert!(is_update_available("v1.0.0", "1.0.1")); // Mixed prefix
    }

    #[test]
    fn test_prerelease_comparison() {
        assert!(is_update_available("1.0.0-alpha", "1.0.0-beta"));
        assert!(is_update_available("1.0.0-beta", "1.0.0"));
        assert!(!is_update_available("1.0.0", "1.0.0-beta"));
    }

    #[test]
    fn test_branch_version_always_outdated() {
        assert!(is_update_available("main-latest", "1.0.0"));
        assert!(is_update_available("master-latest", "0.0.1"));
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version("v1.2.3"), "1.2.3");
        assert_eq!(normalize_version("1.2.3-beta"), "1.2.3-beta");
        assert_eq!(normalize_version("main-latest"), "main-latest");
    }
}
