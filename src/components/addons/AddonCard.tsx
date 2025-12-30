import { FC, useState, useMemo } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button } from '../common/Button';
import { DependencyDialog } from './DependencyDialog';
import { useAddonStore } from '../../stores/addonStore';
import { useIndexStore } from '../../stores/indexStore';
import type { IndexAddon } from '../../types/index';
import type { InstalledAddon, DependencyResult, ResolvedDependency } from '../../types/addon';

// Dependency status for highlighting
type DepStatus = 'installed' | 'available' | 'missing';

interface DependencyInfo {
  slug: string;
  status: DepStatus;
}

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

// Format date/time in local timezone (e.g., "Oct 19, 2024 3:16 AM")
function formatLocalDateTime(dateString: string): string {
  const date = new Date(dateString);
  return date.toLocaleString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

export const AddonCard: FC<AddonCardProps> = ({ addon }) => {
  const { installed, downloads, installAddon, uninstallAddon, resolveAddonDependencies } = useAddonStore();
  const { addons: indexAddons } = useIndexStore();

  // State for dependency dialog
  const [showDepDialog, setShowDepDialog] = useState(false);
  const [depResult, setDepResult] = useState<DependencyResult | null>(null);
  const [installingWithDeps, setInstallingWithDeps] = useState(false);
  const [resolvingDeps, setResolvingDeps] = useState(false);

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

  // Analyze dependency status
  const dependencyInfos = useMemo((): DependencyInfo[] => {
    if (!hasPotentialDependencies) return [];

    // Normalize slug for matching: lowercase, replace dots with hyphens
    // This handles "LibAddonMenu-2.0" matching "libaddonmenu-2-0" in the index
    const normalizeSlug = (s: string) => s.toLowerCase().replace(/\./g, '-');

    // Build sets for quick lookup (normalized for matching)
    const installedSlugs = new Set(installed.map(a => normalizeSlug(a.slug)));
    const indexSlugs = new Set(indexAddons.map(a => normalizeSlug(a.slug)));

    // Strip version suffix for base name matching (e.g., "libaddonmenu-2-0" -> "libaddonmenu")
    const getBaseName = (s: string) => s.replace(/-[\d-]+$/, '');

    return addon.compatibility.required_dependencies.map(depSlug => {
      const depNorm = normalizeSlug(depSlug);
      const depBase = getBaseName(depNorm);

      const isDepInstalled = installedSlugs.has(depNorm) ||
        installedSlugs.has(depBase) ||
        [...installedSlugs].some(s => s.includes(depBase) || depBase.includes(getBaseName(s)));

      const isInIndex = indexSlugs.has(depNorm) ||
        indexSlugs.has(depBase) ||
        [...indexSlugs].some(s => s.includes(depBase) || depBase.includes(getBaseName(s)));

      let status: DepStatus;
      if (isDepInstalled) {
        status = 'installed';
      } else if (isInIndex) {
        status = 'available';
      } else {
        status = 'missing';
      }

      return { slug: depSlug, status };
    });
  }, [addon.compatibility.required_dependencies, installed, indexAddons, hasPotentialDependencies]);

  const hasMissingDeps = dependencyInfos.some(d => d.status === 'missing');

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
    <div className="bg-gray-800 rounded-lg p-3 hover:bg-gray-750 transition-colors border border-gray-700">
      {/* Header: Name + Actions */}
      <div className="flex justify-between items-start gap-3">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-gray-100 truncate">{addon.name}</h3>
            {isInstalled && !hasUpdate && (
              <span className="flex-shrink-0 w-2 h-2 rounded-full bg-green-500" title="Installed" />
            )}
          </div>
          <p className="text-xs text-gray-500 truncate">
            {addon.authors.join(', ')}
            {addon.url && (
              <>
                <span className="mx-1.5">·</span>
                <button
                  onClick={handleOpenDocs}
                  className="text-amber-500/70 hover:text-amber-400 transition-colors"
                >
                  docs
                </button>
              </>
            )}
          </p>
        </div>

        {/* Action buttons */}
        <div className="flex gap-2 flex-shrink-0">
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
              <div className="w-16 h-1.5 bg-gray-700 rounded-full overflow-hidden">
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

      {/* Description */}
      <p className="mt-2 text-sm text-gray-400 line-clamp-1">
        {addon.description}
      </p>

      {/* Metadata grid */}
      <div className="mt-2 grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
        {/* Version */}
        <div className="flex items-center gap-1.5 text-gray-500">
          <svg className="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
          </svg>
          <span className="truncate">
            {addon.latest_release?.version ?? addon.source.branch}
            {isInstalled && hasUpdate && (
              <span className="text-amber-400 ml-1">← {installedAddon.installedVersion}</span>
            )}
          </span>
        </div>

        {/* Last updated */}
        <div className="flex items-center gap-1.5 text-gray-500">
          <svg className="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{addon.last_updated ? formatLocalDateTime(addon.last_updated) : 'unknown'}</span>
        </div>

        {/* Tags */}
        {addon.tags.length > 0 && (
          <div className="col-span-2 flex items-center gap-1.5 text-gray-500">
            <svg className="w-3 h-3 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 7h.01M7 3h5c.512 0 1.024.195 1.414.586l7 7a2 2 0 010 2.828l-7 7a2 2 0 01-2.828 0l-7-7A1.994 1.994 0 013 12V7a4 4 0 014-4z" />
            </svg>
            <span className="truncate">
              {addon.tags.slice(0, 4).join(', ')}
              {addon.tags.length > 4 && ` +${addon.tags.length - 4}`}
            </span>
          </div>
        )}

        {/* Dependencies */}
        {hasPotentialDependencies && (
          <div className="col-span-2 flex items-center gap-1.5">
            <svg className={`w-3 h-3 flex-shrink-0 ${hasMissingDeps ? 'text-red-400' : 'text-gray-500'}`} fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
            </svg>
            <span className="truncate">
              {dependencyInfos.map((dep, idx) => (
                <span key={dep.slug}>
                  <span
                    className={
                      dep.status === 'installed'
                        ? 'text-green-400'
                        : dep.status === 'available'
                          ? 'text-gray-500'
                          : 'text-red-400'
                    }
                    title={
                      dep.status === 'installed'
                        ? 'Installed'
                        : dep.status === 'available'
                          ? 'Available in index'
                          : 'Not available - install manually'
                    }
                  >
                    {dep.slug}
                  </span>
                  {idx < dependencyInfos.length - 1 && <span className="text-gray-600">, </span>}
                </span>
              ))}
            </span>
          </div>
        )}
      </div>

      {/* Update available banner */}
      {hasUpdate && (
        <div className="mt-2 px-2 py-1 bg-amber-900/30 border border-amber-700/50 rounded text-xs text-amber-200 flex items-center gap-1.5">
          <svg className="w-3.5 h-3.5 text-amber-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
          <span>
            Update: <span className="text-amber-400/70">{installedAddon.installedVersion}</span>
            <span className="mx-1">→</span>
            <span className="text-amber-300 font-medium">{addon.latest_release?.version}</span>
          </span>
        </div>
      )}

      {/* Missing dependencies warning */}
      {hasMissingDeps && (
        <div className="mt-2 px-2 py-1 bg-red-900/20 border border-red-800/50 rounded text-xs text-red-300">
          Missing dependencies - install via ESOUI or Minion
        </div>
      )}

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
