import { invoke } from '@tauri-apps/api/core';
import type { InstalledAddon, UpdateInfo, ScannedAddon, VersionTracking, DependencyResult } from '../types/addon';
import type { AddonIndex, IndexStats, InstallInfo } from '../types/index';
import type { CustomRepo, GitHubRepoInfo, GitHubBranchInfo, GitHubReleaseInfo, RepoPreview } from '../types/github';
import type { AppSettings } from '../types/settings';

// ============================================================================
// Addon Commands
// ============================================================================

export async function getInstalledAddons(): Promise<InstalledAddon[]> {
  return invoke('get_installed_addons');
}

export async function installAddon(
  slug: string,
  name: string,
  version: string,
  downloadUrl: string,
  sourceType?: string,
  sourceRepo?: string,
  installInfo?: InstallInfo,
  versionTracking?: VersionTracking
): Promise<InstalledAddon> {
  return invoke('install_addon', {
    slug,
    name,
    version,
    downloadUrl,
    sourceType,
    sourceRepo,
    installInfo,
    versionTracking,
  });
}

export async function uninstallAddon(slug: string): Promise<void> {
  return invoke('uninstall_addon', { slug });
}

export async function scanLocalAddons(): Promise<ScannedAddon[]> {
  return invoke('scan_local_addons');
}

export async function checkUpdates(): Promise<UpdateInfo[]> {
  return invoke('check_updates');
}

export async function getAddonDirectory(): Promise<string | null> {
  return invoke('get_addon_directory');
}

export async function setAddonDirectory(path: string): Promise<void> {
  return invoke('set_addon_directory', { path });
}

export async function resolveAddonDependencies(slug: string): Promise<DependencyResult> {
  return invoke('resolve_addon_dependencies', { slug });
}

// ============================================================================
// GitHub Commands
// ============================================================================

export async function addCustomRepo(
  repo: string,
  branch?: string,
  releaseType?: string
): Promise<CustomRepo> {
  return invoke('add_custom_repo', { repo, branch, releaseType });
}

export async function getCustomRepos(): Promise<CustomRepo[]> {
  return invoke('get_custom_repos');
}

export async function removeCustomRepo(repo: string): Promise<void> {
  return invoke('remove_custom_repo', { repo });
}

export async function getGitHubRepoInfo(repo: string): Promise<GitHubRepoInfo> {
  return invoke('get_github_repo_info', { repo });
}

export async function installFromGitHub(
  repo: string,
  releaseType?: string,
  branch?: string
): Promise<InstalledAddon> {
  return invoke('install_from_github', { repo, releaseType, branch });
}

export async function getGitHubRelease(repo: string): Promise<GitHubReleaseInfo | null> {
  return invoke('get_github_release', { repo });
}

export async function getGitHubRepoPreview(repo: string): Promise<RepoPreview> {
  return invoke('get_github_repo_preview', { repo });
}

export async function listGitHubBranches(repo: string): Promise<GitHubBranchInfo[]> {
  return invoke('list_github_branches', { repo });
}

// Re-export types for convenience
export type { GitHubReleaseInfo, GitHubBranchInfo, RepoPreview };

// ============================================================================
// Index Commands
// ============================================================================

export async function fetchIndex(force?: boolean): Promise<AddonIndex> {
  return invoke('fetch_index', { force });
}

export async function getCachedIndex(): Promise<AddonIndex | null> {
  return invoke('get_cached_index');
}

export async function getIndexStats(): Promise<IndexStats> {
  return invoke('get_index_stats');
}

// ============================================================================
// Settings Commands
// ============================================================================

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function updateSettings(settings: AppSettings): Promise<void> {
  return invoke('update_settings', { settings });
}

export async function resetSettings(): Promise<AppSettings> {
  return invoke('reset_settings');
}
