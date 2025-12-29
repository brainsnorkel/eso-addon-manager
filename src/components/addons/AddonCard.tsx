import { FC } from 'react';
import { openUrl } from '@tauri-apps/plugin-opener';
import { Button } from '../common/Button';
import { useAddonStore } from '../../stores/addonStore';
import type { IndexAddon } from '../../types/index';

interface AddonCardProps {
  addon: IndexAddon;
}

// Construct download URL from branch when no release exists
function getBranchDownloadUrl(addon: IndexAddon): string | null {
  if (addon.source.type !== 'github') return null;
  const { repo, branch } = addon.source;
  return `https://api.github.com/repos/${repo}/zipball/${branch}`;
}

export const AddonCard: FC<AddonCardProps> = ({ addon }) => {
  const { installed, downloads, installAddon, uninstallAddon } = useAddonStore();

  const installedAddon = installed.find((i) => i.slug === addon.slug);
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

  const handleInstall = async () => {
    if (!downloadUrl) return;
    await installAddon(addon.slug, addon.name, version, downloadUrl, addon.install);
  };

  const handleUninstall = async () => {
    await uninstallAddon(addon.slug);
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
        <span className="px-2 py-1 text-xs rounded-full bg-gray-700 text-gray-300 whitespace-nowrap">
          {addon.category}
        </span>
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

      <div className="mt-4 flex items-center justify-between">
        <span className="text-sm text-gray-500">
          {addon.latest_release?.version ?? (canInstall ? 'Branch: ' + addon.source.branch : 'No release')}
          {isInstalled && (
            <span className="ml-2 text-green-400">
              (installed: {installedAddon.installedVersion})
            </span>
          )}
        </span>

        <div className="flex gap-2">
          {downloadState && downloadState.status !== 'complete' ? (
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
                <Button size="sm" onClick={handleInstall}>
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
            >
              Install
            </Button>
          )}
        </div>
      </div>
    </div>
  );
};
