import { FC, useState } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button } from '../common/Button';
import { DependencyDialog } from './DependencyDialog';
import { useAddonStore } from '../../stores/addonStore';
import type { IndexAddon } from '../../types/index';
import type { InstalledAddon, DependencyResult, ResolvedDependency } from '../../types/addon';

interface AddonCardProps {
  addon: IndexAddon;
}

// Construct download URL from branch when no release exists
function getBranchDownloadUrl(addon: IndexAddon): string | null {
  if (addon.source.type !== 'github') return null;
  const { repo, branch } = addon.source;
  return `https://api.github.com/repos/${repo}/zipball/${branch}`;
}

// Find installed addon by matching slug, target folder, or name
function findInstalledAddon(addon: IndexAddon, installed: InstalledAddon[]): InstalledAddon | undefined {
  const slugLower = addon.slug.toLowerCase();
  const targetFolder = addon.install.target_folder.toLowerCase();
  const nameLower = addon.name.toLowerCase();
  // Strip version suffixes for matching (e.g., "libaddonmenu-2.0" -> "libaddonmenu")
  const slugBase = slugLower.replace(/-[\d.]+$/, '');
  const targetBase = targetFolder.replace(/-[\d.]+$/, '');

  return installed.find((i) => {
    const installedSlug = i.slug.toLowerCase();
    const installedName = i.name.toLowerCase();
    const installedBase = installedSlug.replace(/-[\d.]+$/, '');

    // Exact slug match
    if (installedSlug === slugLower) return true;

    // Target folder match (for locally scanned addons)
    if (installedSlug === targetFolder) return true;

    // Base name match (without version suffix)
    if (installedBase === slugBase || installedBase === targetBase) return true;

    // Name match (fallback)
    if (installedName === nameLower) return true;

    // Partial match - installed slug contains target or vice versa
    if (installedSlug.includes(slugLower) || slugLower.includes(installedSlug)) return true;
    if (installedSlug.includes(targetFolder) || targetFolder.includes(installedSlug)) return true;

    return false;
  });
}

export const AddonCard: FC<AddonCardProps> = ({ addon }) => {
  const { installed, downloads, installAddon, uninstallAddon, resolveAddonDependencies } = useAddonStore();

  // State for dependency dialog
  const [showDepDialog, setShowDepDialog] = useState(false);
  const [depResult, setDepResult] = useState<DependencyResult | null>(null);
  const [installingWithDeps, setInstallingWithDeps] = useState(false);
  const [resolvingDeps, setResolvingDeps] = useState(false);

  // Debug: log first addon's matching attempt
  if (addon.slug === 'libaddonmenu' || installed.length > 0 && addon.slug === 'libaddonmenu') {
    console.log('AddonCard debug:', {
      indexSlug: addon.slug,
      targetFolder: addon.install.target_folder,
      installedCount: installed.length,
      installedSlugs: installed.map(i => i.slug).slice(0, 5),
    });
  }

  const installedAddon = findInstalledAddon(addon, installed);
  const isInstalled = !!installedAddon;
  const downloadState = downloads.get(addon.slug);

  // Use release URL if available, otherwise fall back to branch
  const downloadUrl = addon.latest_release?.download_url ?? getBranchDownloadUrl(addon);
  const version = addon.latest_release?.version ?? `${addon.source.branch}-latest`;
  const canInstall = !!downloadUrl;

  const hasUpdate =
    isInstalled &&
    addon.latest_release &&
    installedAddon.installedVersion !== addon.latest_release.version;

  // Check for dependencies from the addon's compatibility info
  const hasPotentialDependencies = addon.compatibility.required_dependencies.length > 0;

  const handleInstall = async () => {
    if (!downloadUrl) return;

    // If addon has dependencies, resolve them first
    if (hasPotentialDependencies) {
      setResolvingDeps(true);
      try {
        const result = await resolveAddonDependencies(addon.slug);
        if (result && (result.resolved.length > 0 || result.unresolved.length > 0)) {
          // Show the dependency dialog
          setDepResult(result);
          setShowDepDialog(true);
          return;
        }
      } catch (e) {
        console.error('Failed to resolve dependencies:', e);
      } finally {
        setResolvingDeps(false);
      }
    }

    // No dependencies or all already installed - proceed with install
    await doInstall();
  };

  const doInstall = async () => {
    if (!downloadUrl) return;
    // Pass version tracking info for simplified update detection
    const versionTracking = {
      versionSortKey: addon.version_info?.version_sort_key,
      commitSha: addon.latest_release?.commit_sha,
    };
    // Pass download sources for multi-source fallback (jsDelivr CDN -> GitHub archive)
    await installAddon(addon.slug, addon.name, version, downloadUrl, addon.install, versionTracking, addon.download_sources);
  };

  const handleDepConfirm = async (selectedDeps: ResolvedDependency[]) => {
    setInstallingWithDeps(true);
    try {
      // Install dependencies first (in order - deepest first)
      for (const dep of selectedDeps) {
        await installAddon(
          dep.slug,
          dep.name,
          dep.version,
          dep.downloadUrl,
          dep.installInfo,
          undefined
        );
      }

      // Then install the main addon
      await doInstall();
    } finally {
      setInstallingWithDeps(false);
      setShowDepDialog(false);
      setDepResult(null);
    }
  };

  const handleDepCancel = () => {
    setShowDepDialog(false);
    setDepResult(null);
  };

  const handleUninstall = async () => {
    // Use the installed addon's actual slug (may differ from index slug for local addons)
    if (installedAddon) {
      await uninstallAddon(installedAddon.slug);
    }
  };

  const handleOpenDocs = async () => {
    if (addon.url) {
      await openUrl(addon.url);
    }
  };

  return (
    <div className="bg-gray-800 rounded-lg p-4 hover:bg-gray-750 transition-colors border border-gray-700">
      <div className="flex justify-between items-start gap-4">
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-gray-100 truncate">{addon.name}</h3>
          <p className="text-sm text-gray-400 mt-0.5">
            {addon.authors.join(', ')}
          </p>
          {addon.url && (
            <button
              onClick={handleOpenDocs}
              className="text-xs text-amber-500/70 hover:text-amber-400 transition-colors mt-0.5 truncate max-w-full text-left"
              title={addon.url}
            >
              {addon.url.replace(/^https?:\/\/(www\.)?/, '').replace(/\/$/, '')}
            </button>
          )}
        </div>
      </div>

      <p className="mt-3 text-sm text-gray-400 line-clamp-2">
        {addon.description}
      </p>

      {addon.tags.length > 0 && (
        <div className="mt-3 flex flex-wrap gap-1">
          {addon.tags.slice(0, 3).map((tag) => (
            <span
              key={tag}
              className="px-2 py-0.5 text-xs rounded bg-gray-700/50 text-gray-400"
            >
              {tag}
            </span>
          ))}
          {addon.tags.length > 3 && (
            <span className="px-2 py-0.5 text-xs text-gray-500">
              +{addon.tags.length - 3} more
            </span>
          )}
        </div>
      )}

      {/* Dependencies list */}
      {hasPotentialDependencies && (
        <div className="mt-3 flex items-center gap-1.5 text-xs text-gray-500">
          <svg className="w-3.5 h-3.5 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
          </svg>
          <span className="truncate">
            Requires: {addon.compatibility.required_dependencies.join(', ')}
          </span>
        </div>
      )}

      {/* Update available banner */}
      {hasUpdate && (
        <div className="mt-3 px-3 py-2 bg-amber-900/30 border border-amber-700/50 rounded-lg flex items-center gap-2">
          <svg className="w-4 h-4 text-amber-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <span className="text-xs text-amber-200">
            Update available: <span className="text-amber-400/70">{installedAddon.installedVersion}</span>
            <span className="mx-1">→</span>
            <span className="text-amber-300 font-medium">{addon.latest_release?.version}</span>
          </span>
        </div>
      )}

      <div className="mt-4 flex items-center justify-between">
        <span className="text-sm text-gray-500">
          {addon.latest_release?.version ?? (canInstall ? 'Branch: ' + addon.source.branch : 'No release')}
          {isInstalled && !hasUpdate && (
            <span className="ml-2 text-green-400">
              (installed: {installedAddon.installedVersion})
            </span>
          )}
        </span>

        <div className="flex gap-2">
          {downloadState && downloadState.status === 'failed' ? (
            <div className="flex items-center gap-2">
              <span className="text-xs text-red-400" title={downloadState.error || 'Installation failed'}>
                Failed
              </span>
              <Button size="sm" onClick={handleInstall}>
                Retry
              </Button>
            </div>
          ) : downloadState && downloadState.status !== 'complete' ? (
            <div className="flex items-center gap-2">
              <div className="w-20 h-2 bg-gray-700 rounded-full overflow-hidden">
                <div
                  className="h-full bg-amber-500 transition-all"
                  style={{ width: `${downloadState.progress * 100}%` }}
                />
              </div>
              <span className="text-xs text-gray-400 capitalize">
                {downloadState.status}
              </span>
            </div>
          ) : isInstalled ? (
            <>
              {hasUpdate && (
                <Button size="sm" onClick={handleInstall} loading={resolvingDeps}>
                  Update
                </Button>
              )}
              <Button size="sm" variant="danger" onClick={handleUninstall}>
                Remove
              </Button>
            </>
          ) : (
            <Button
              size="sm"
              onClick={handleInstall}
              disabled={!canInstall}
              loading={resolvingDeps}
            >
              Install
            </Button>
          )}
        </div>
      </div>

      {/* Dependency Dialog */}
      {showDepDialog && depResult && (
        <DependencyDialog
          addonName={addon.name}
          dependencies={depResult}
          onConfirm={handleDepConfirm}
          onCancel={handleDepCancel}
          installing={installingWithDeps}
        />
      )}
    </div>
  );
};
