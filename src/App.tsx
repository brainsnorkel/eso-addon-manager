import { useState, useEffect } from 'react';
import { Sidebar } from './components/layout/Sidebar';
import { Header } from './components/layout/Header';
import { Button } from './components/common/Button';
import { UpdateBanner } from './components/common/UpdateBanner';
import { SearchBar } from './components/search/SearchBar';
import { AddonCard } from './components/addons/AddonCard';
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
  const { installed, loading, fetchInstalled, scanLocalAddons } = useAddonStore();

  return (
    <>
      <Header
        title="Installed Addons"
        subtitle={`${installed.length} addons installed`}
        actions={
          <div className="flex gap-2">
            <Button onClick={scanLocalAddons} variant="secondary">
              Scan Local
            </Button>
            <Button onClick={fetchInstalled} loading={loading} variant="secondary">
              Refresh
            </Button>
          </div>
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
              {installed.map((addon) => (
                <div
                  key={addon.slug}
                  className="bg-gray-800 rounded-lg p-4 border border-gray-700"
                >
                  <div className="flex justify-between items-center">
                    <div>
                      <h3 className="font-semibold text-gray-100">{addon.name}</h3>
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
  const [repoInput, setRepoInput] = useState('');
  const [addError, setAddError] = useState<string | null>(null);

  const {
    repos,
    loading,
    installing,
    error,
    fetchRepos,
    addRepo,
    removeRepo,
    installFromRepo,
    clearError
  } = useGitHubStore();
  const { fetchInstalled } = useAddonStore();

  useEffect(() => {
    fetchRepos();
  }, []);

  const handleAddRepo = async () => {
    if (!repoInput.trim()) return;

    // Validate format: owner/repo
    if (!repoInput.includes('/')) {
      setAddError('Please enter a valid GitHub repository (e.g., owner/repo)');
      return;
    }

    try {
      setAddError(null);
      await addRepo(repoInput.trim());
      setRepoInput('');
      setShowAddModal(false);
    } catch (e) {
      setAddError(String(e));
    }
  };

  const handleInstall = async (repo: string, releaseType: string) => {
    try {
      await installFromRepo(repo, releaseType);
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
              <button onClick={clearError} className="text-red-400 hover:text-red-300">×</button>
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
                        onClick={() => handleInstall(repo.repo, repo.releaseType)}
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
      {showAddModal && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="bg-gray-800 rounded-lg p-6 w-full max-w-md border border-gray-700">
            <h2 className="text-lg font-semibold text-gray-100 mb-4">Add GitHub Repository</h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm text-gray-400 mb-1">Repository</label>
                <input
                  type="text"
                  value={repoInput}
                  onChange={(e) => setRepoInput(e.target.value)}
                  placeholder="owner/repository"
                  className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-100 focus:ring-amber-500 focus:border-amber-500"
                />
                <p className="text-xs text-gray-500 mt-1">
                  Example: brainsnorkel/eso-addon-index
                </p>
              </div>

              {addError && (
                <p className="text-sm text-red-400">{addError}</p>
              )}

              <div className="flex justify-end gap-2">
                <Button variant="secondary" onClick={() => {
                  setShowAddModal(false);
                  setRepoInput('');
                  setAddError(null);
                }}>
                  Cancel
                </Button>
                <Button onClick={handleAddRepo} loading={loading}>
                  Add Repository
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
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
