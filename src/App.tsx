import { useState, useEffect } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import { Header } from './components/layout/Header';
import { Button } from './components/common/Button';
import { UpdateBanner } from './components/common/UpdateBanner';
import { SearchBar } from './components/search/SearchBar';
import { AddonCard } from './components/addons/AddonCard';
import { AddRepoModal } from './components/github/AddRepoModal';
import { useIndexStore } from './stores/indexStore';
import { useAddonStore } from './stores/addonStore';
import { useSettingsStore } from './stores/settingsStore';
import { useGitHubStore } from './stores/githubStore';

type View = 'browse' | 'installed' | 'github' | 'updates' | 'settings';

function App() {
  const [activeView, setActiveView] = useState<View>('browse');

  // Initialize stores
  const { fetchIndex } = useIndexStore();
  const { fetchInstalled } = useAddonStore();
  const { fetchSettings, fetchAddonDirectory } = useSettingsStore();

  useEffect(() => {
    fetchSettings();
    fetchAddonDirectory();
    fetchIndex(true); // Always refresh index on startup
    fetchInstalled();
  }, []);

  const renderContent = () => {
    switch (activeView) {
      case 'browse':
        return <BrowseView />;
      case 'installed':
        return <InstalledView />;
      case 'github':
        return <GitHubView />;
      case 'updates':
        return <UpdatesView />;
      case 'settings':
        return <SettingsView />;
      default:
        return <BrowseView />;
    }
  };

  return (
    <div className="flex flex-col h-screen bg-gray-900 text-gray-100">
      <UpdateBanner />
      <div className="flex flex-1 overflow-hidden">
        <Sidebar activeView={activeView} onViewChange={(v) => setActiveView(v as View)} />
        <main className="flex-1 flex flex-col overflow-hidden">
          {renderContent()}
        </main>
      </div>
    </div>
  );
}

function BrowseView() {
  const { loading, fetchIndex, filteredAddons, addons } = useIndexStore();
  const { error, clearError } = useAddonStore();
  const filtered = filteredAddons();

  return (
    <>
      <Header
        title="Browse Addons"
        subtitle={`${filtered.length} of ${addons.length} addons`}
        actions={
          <Button onClick={() => fetchIndex(true)} loading={loading} variant="secondary">
            Refresh Index
          </Button>
        }
      />
      <div className="p-6 flex-1 overflow-auto">
        <div className="max-w-4xl mx-auto">
          {error && (
            <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg">
              <div className="flex justify-between items-start gap-2">
                <div className="flex-1 min-w-0">
                  <span className="text-red-200 text-sm font-medium">Installation Error</span>
                  <p className="text-red-300 text-sm mt-1 break-words">{error}</p>
                </div>
                <button
                  onClick={clearError}
                  className="text-red-400 hover:text-red-300 text-xl leading-none flex-shrink-0"
                  title="Dismiss"
                >
                  ×
                </button>
              </div>
            </div>
          )}
          <SearchBar />
          <div className="mt-6 grid gap-4">
            {loading && filtered.length === 0 ? (
              <div className="text-center py-12 text-gray-400">
                Loading addons...
              </div>
            ) : filtered.length === 0 ? (
              <div className="text-center py-12 text-gray-400">
                No addons found. Try adjusting your search.
              </div>
            ) : (
              filtered.map((addon) => (
                <AddonCard key={addon.slug} addon={addon} />
              ))
            )}
          </div>
        </div>
      </div>
    </>
  );
}

function InstalledView() {
  const { installed, loading, fetchInstalled } = useAddonStore();
  const { addons: indexAddons } = useIndexStore();

  // Create a set of index slugs for fast lookup
  const indexSlugs = new Set(indexAddons.map(a => a.slug));

  // Classify addons as managed (in index) or unmanaged (not in index)
  const classifiedAddons = installed.map(addon => ({
    ...addon,
    isManaged: indexSlugs.has(addon.slug)
  }));

  const managedCount = classifiedAddons.filter(a => a.isManaged).length;
  const unmanagedCount = classifiedAddons.filter(a => !a.isManaged).length;

  // Build subtitle
  const subtitleParts: string[] = [];
  if (managedCount > 0) subtitleParts.push(`${managedCount} managed`);
  if (unmanagedCount > 0) subtitleParts.push(`${unmanagedCount} unmanaged`);
  const subtitle = subtitleParts.length > 0 ? subtitleParts.join(', ') : 'No addons installed';

  return (
    <>
      <Header
        title="Installed Addons"
        subtitle={subtitle}
        actions={
          // Refresh scans the addon directory and auto-imports any untracked addons
          <Button onClick={fetchInstalled} loading={loading} variant="secondary">
            Refresh
          </Button>
        }
      />
      <div className="p-6 flex-1 overflow-auto">
        <div className="max-w-4xl mx-auto">
          {installed.length === 0 ? (
            <div className="text-center py-12 text-gray-400">
              <p>No addons installed yet.</p>
              <p className="mt-2 text-sm">Browse addons to get started!</p>
            </div>
          ) : (
            <div className="grid gap-4">
              {classifiedAddons.map((addon) => (
                <div
                  key={addon.slug}
                  className="bg-gray-800 rounded-lg p-4 border border-gray-700"
                >
                  <div className="flex justify-between items-center">
                    <div>
                      <div className="flex items-center gap-2">
                        <h3 className="font-semibold text-gray-100">{addon.name}</h3>
                        {!addon.isManaged && (
                          <span className="px-2 py-0.5 text-xs font-medium rounded bg-yellow-900/50 text-yellow-300 border border-yellow-700">
                            unmanaged
                          </span>
                        )}
                      </div>
                      <p className="text-sm text-gray-400">
                        v{addon.installedVersion} - {addon.sourceType}
                      </p>
                    </div>
                    <span className="text-xs text-gray-500">
                      Installed {new Date(addon.installedAt).toLocaleDateString()}
                    </span>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

function GitHubView() {
  const [showAddModal, setShowAddModal] = useState(false);

  const {
    repos,
    loading,
    installing,
    error,
    fetchRepos,
    removeRepo,
    installFromRepo,
    clearError
  } = useGitHubStore();
  const { fetchInstalled } = useAddonStore();

  useEffect(() => {
    fetchRepos();
  }, []);

  const handleInstall = async (repo: string, releaseType: string, branch?: string) => {
    try {
      await installFromRepo(repo, releaseType, releaseType === 'branch' ? branch : undefined);
      await fetchInstalled();
    } catch (e) {
      console.error('Install failed:', e);
    }
  };

  return (
    <>
      <Header
        title="GitHub Repositories"
        subtitle={`${repos.length} repositories tracked`}
        actions={<Button onClick={() => setShowAddModal(true)}>Add Repository</Button>}
      />
      <div className="p-6 flex-1 overflow-auto">
        <div className="max-w-4xl mx-auto">
          {error && (
            <div className="mb-4 p-3 bg-red-900/50 border border-red-700 rounded-lg flex justify-between items-center">
              <span className="text-red-200 text-sm">{error}</span>
              <button onClick={clearError} className="text-red-400 hover:text-red-300">&times;</button>
            </div>
          )}

          {loading && repos.length === 0 ? (
            <div className="text-center py-12 text-gray-400">Loading repositories...</div>
          ) : repos.length === 0 ? (
            <div className="text-center py-12 text-gray-400">
              <p>No custom repositories added yet.</p>
              <p className="mt-2 text-sm">
                Add a GitHub repository to track addons that aren't in the main index.
              </p>
            </div>
          ) : (
            <div className="grid gap-4">
              {repos.map((repo) => (
                <div
                  key={repo.repo}
                  className="bg-gray-800 rounded-lg p-4 border border-gray-700"
                >
                  <div className="flex justify-between items-center">
                    <div>
                      <h3 className="font-semibold text-gray-100">{repo.repo}</h3>
                      <p className="text-sm text-gray-400">
                        {repo.releaseType === 'release' ? 'Latest Release' : `Branch: ${repo.branch}`}
                      </p>
                    </div>
                    <div className="flex gap-2">
                      <Button
                        size="sm"
                        onClick={() => handleInstall(repo.repo, repo.releaseType, repo.branch)}
                        loading={installing === repo.repo}
                      >
                        Install
                      </Button>
                      <Button
                        size="sm"
                        variant="secondary"
                        onClick={() => removeRepo(repo.repo)}
                      >
                        Remove
                      </Button>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>

      {/* Add Repository Modal */}
      <AddRepoModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onSuccess={() => fetchRepos()}
      />
    </>
  );
}

function UpdatesView() {
  const { updates, checkUpdates } = useAddonStore();

  useEffect(() => {
    checkUpdates();
  }, []);

  return (
    <>
      <Header
        title="Available Updates"
        subtitle={`${updates.length} updates available`}
        actions={<Button onClick={checkUpdates}>Check for Updates</Button>}
      />
      <div className="p-6 flex-1 overflow-auto">
        <div className="max-w-4xl mx-auto">
          {updates.length === 0 ? (
            <div className="text-center py-12 text-gray-400">
              <p>All addons are up to date!</p>
            </div>
          ) : (
            <div className="grid gap-4">
              {updates.map((update) => (
                <div
                  key={update.slug}
                  className="bg-gray-800 rounded-lg p-4 border border-gray-700"
                >
                  <div className="flex justify-between items-center">
                    <div>
                      <h3 className="font-semibold text-gray-100">{update.name}</h3>
                      <p className="text-sm text-gray-400">
                        {update.currentVersion} → {update.newVersion}
                      </p>
                    </div>
                    <Button size="sm">Update</Button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </>
  );
}

function SettingsView() {
  const { settings, addonDirectory, updateSettings } = useSettingsStore();

  return (
    <>
      <Header title="Settings" subtitle="Configure the addon manager" />
      <div className="p-6 flex-1 overflow-auto">
        <div className="max-w-2xl mx-auto space-y-6">
          {/* Addon Directory */}
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 className="font-semibold text-gray-100 mb-2">ESO Addon Directory</h3>
            <p className="text-sm text-gray-400 mb-3">
              {addonDirectory ?? 'Not detected - please set manually'}
            </p>
            <Button variant="secondary" size="sm">
              Browse...
            </Button>
          </div>

          {/* Update Settings */}
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 className="font-semibold text-gray-100 mb-4">Updates</h3>
            <div className="space-y-3">
              <label className="flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={settings?.checkUpdatesOnStartup ?? true}
                  onChange={(e) =>
                    updateSettings({ checkUpdatesOnStartup: e.target.checked })
                  }
                  className="rounded bg-gray-700 border-gray-600 text-amber-500 focus:ring-amber-500"
                />
                <span className="text-sm text-gray-300">
                  Check for updates on startup
                </span>
              </label>
              <label className="flex items-center gap-3">
                <input
                  type="checkbox"
                  checked={settings?.autoUpdate ?? false}
                  onChange={(e) =>
                    updateSettings({ autoUpdate: e.target.checked })
                  }
                  className="rounded bg-gray-700 border-gray-600 text-amber-500 focus:ring-amber-500"
                />
                <span className="text-sm text-gray-300">
                  Automatically install updates
                </span>
              </label>
            </div>
          </div>

          {/* Theme */}
          <div className="bg-gray-800 rounded-lg p-4 border border-gray-700">
            <h3 className="font-semibold text-gray-100 mb-4">Appearance</h3>
            <select
              value={settings?.theme ?? 'system'}
              onChange={(e) =>
                updateSettings({ theme: e.target.value as 'system' | 'light' | 'dark' })
              }
              className="bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-100 focus:ring-amber-500 focus:border-amber-500"
            >
              <option value="system">System</option>
              <option value="light">Light</option>
              <option value="dark">Dark</option>
            </select>
          </div>
        </div>
      </div>
    </>
  );
}

export default App;
