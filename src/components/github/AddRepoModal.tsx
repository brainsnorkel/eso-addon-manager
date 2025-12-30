import { FC, useState, useEffect } from 'react';
import { Button } from '../common/Button';
import { useGitHubStore } from '../../stores/githubStore';

interface AddRepoModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSuccess: () => void;
}

type ReleaseType = 'release' | 'branch';

export const AddRepoModal: FC<AddRepoModalProps> = ({ isOpen, onClose, onSuccess }) => {
  const [repoInput, setRepoInput] = useState('');
  const [releaseType, setReleaseType] = useState<ReleaseType>('release');
  const [selectedBranch, setSelectedBranch] = useState('');
  const [addError, setAddError] = useState<string | null>(null);
  const [isAdding, setIsAdding] = useState(false);

  const {
    repoPreview,
    previewLoading,
    previewError,
    fetchRepoPreview,
    clearPreview,
    addRepo,
  } = useGitHubStore();

  // Reset state when modal opens/closes
  useEffect(() => {
    if (!isOpen) {
      setRepoInput('');
      setReleaseType('release');
      setSelectedBranch('');
      setAddError(null);
      clearPreview();
    }
  }, [isOpen, clearPreview]);

  // Auto-select release type based on repo capabilities
  useEffect(() => {
    if (repoPreview) {
      if (repoPreview.hasReleases) {
        setReleaseType('release');
      } else {
        setReleaseType('branch');
      }
      setSelectedBranch(repoPreview.defaultBranch);
    }
  }, [repoPreview]);

  const handleValidate = async () => {
    if (!repoInput.trim()) return;

    // Validate format: owner/repo
    if (!repoInput.includes('/')) {
      setAddError('Please enter a valid GitHub repository (e.g., owner/repo)');
      return;
    }

    setAddError(null);
    try {
      await fetchRepoPreview(repoInput.trim());
    } catch {
      // Error is set in store
    }
  };

  const handleAdd = async () => {
    if (!repoPreview) return;

    setIsAdding(true);
    setAddError(null);

    try {
      const branch = releaseType === 'branch' ? selectedBranch : undefined;
      await addRepo(repoInput.trim(), branch, releaseType);
      onSuccess();
      onClose();
    } catch (e) {
      setAddError(String(e));
    } finally {
      setIsAdding(false);
    }
  };

  const handleClose = () => {
    setRepoInput('');
    setAddError(null);
    clearPreview();
    onClose();
  };

  const formatDate = (dateString?: string) => {
    if (!dateString) return 'Unknown';
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Yesterday';
    if (diffDays < 7) return `${diffDays} days ago`;
    if (diffDays < 30) return `${Math.floor(diffDays / 7)} weeks ago`;
    if (diffDays < 365) return `${Math.floor(diffDays / 30)} months ago`;
    return `${Math.floor(diffDays / 365)} years ago`;
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg p-6 w-full max-w-lg border border-gray-700">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg font-semibold text-gray-100">Add GitHub Repository</h2>
          <button
            onClick={handleClose}
            className="text-gray-400 hover:text-gray-300 text-xl leading-none"
          >
            &times;
          </button>
        </div>

        <div className="space-y-4">
          {/* Repository Input */}
          <div>
            <label className="block text-sm text-gray-400 mb-1">Repository</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={repoInput}
                onChange={(e) => {
                  setRepoInput(e.target.value);
                  setAddError(null);
                  if (repoPreview) clearPreview();
                }}
                placeholder="owner/repository"
                className="flex-1 bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-100 focus:ring-amber-500 focus:border-amber-500"
                onKeyDown={(e) => {
                  if (e.key === 'Enter' && !repoPreview) {
                    handleValidate();
                  }
                }}
              />
              <Button
                onClick={handleValidate}
                loading={previewLoading}
                disabled={!repoInput.trim() || !!repoPreview}
                variant={repoPreview ? 'secondary' : 'primary'}
              >
                {repoPreview ? 'Validated' : 'Validate'}
              </Button>
            </div>
            <p className="text-xs text-gray-500 mt-1">
              Example: brainsnorkel/libaddonmenu
            </p>
          </div>

          {/* Error display */}
          {(addError || previewError) && (
            <div className="p-3 bg-red-900/50 border border-red-700 rounded-lg">
              <p className="text-sm text-red-300">{addError || previewError}</p>
            </div>
          )}

          {/* Repository Preview */}
          {repoPreview && (
            <>
              <div className="border-t border-gray-700 pt-4">
                <div className="bg-gray-700/50 rounded-lg p-4 border border-gray-600">
                  <div className="flex justify-between items-start">
                    <div className="flex-1 min-w-0">
                      <h3 className="font-semibold text-gray-100 truncate">{repoPreview.name}</h3>
                      {repoPreview.description && (
                        <p className="text-sm text-gray-400 mt-1 line-clamp-2">
                          {repoPreview.description}
                        </p>
                      )}
                      <p className="text-xs text-gray-500 mt-2">
                        Last updated: {formatDate(repoPreview.updatedAt)}
                      </p>
                    </div>
                    <div className="flex items-center gap-1 text-gray-400 ml-4">
                      <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                        <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
                      </svg>
                      <span className="text-sm">{repoPreview.stars.toLocaleString()}</span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Download Source Selection */}
              <div>
                <label className="block text-sm text-gray-400 mb-2">Download Source</label>
                <div className="space-y-2">
                  {/* GitHub Releases Option */}
                  <label
                    className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                      releaseType === 'release'
                        ? 'bg-amber-500/10 border-amber-500/50'
                        : 'bg-gray-700/50 border-gray-600 hover:border-gray-500'
                    } ${!repoPreview.hasReleases ? 'opacity-50 cursor-not-allowed' : ''}`}
                  >
                    <input
                      type="radio"
                      name="releaseType"
                      value="release"
                      checked={releaseType === 'release'}
                      onChange={() => setReleaseType('release')}
                      disabled={!repoPreview.hasReleases}
                      className="mt-1 text-amber-500 focus:ring-amber-500 bg-gray-600 border-gray-500"
                    />
                    <div className="flex-1">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-gray-100">GitHub Releases</span>
                        {repoPreview.hasReleases && (
                          <span className="text-xs px-2 py-0.5 bg-green-900/50 text-green-300 rounded border border-green-700">
                            Recommended
                          </span>
                        )}
                      </div>
                      {repoPreview.hasReleases && repoPreview.latestRelease ? (
                        <p className="text-sm text-gray-400 mt-0.5">
                          Latest: {repoPreview.latestRelease.tagName}
                          {repoPreview.latestRelease.publishedAt && (
                            <span className="text-gray-500">
                              {' '}({formatDate(repoPreview.latestRelease.publishedAt)})
                            </span>
                          )}
                        </p>
                      ) : (
                        <p className="text-sm text-gray-500 mt-0.5">
                          No releases available
                        </p>
                      )}
                    </div>
                  </label>

                  {/* Branch Option */}
                  <label
                    className={`flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors ${
                      releaseType === 'branch'
                        ? 'bg-amber-500/10 border-amber-500/50'
                        : 'bg-gray-700/50 border-gray-600 hover:border-gray-500'
                    }`}
                  >
                    <input
                      type="radio"
                      name="releaseType"
                      value="branch"
                      checked={releaseType === 'branch'}
                      onChange={() => setReleaseType('branch')}
                      className="mt-1 text-amber-500 focus:ring-amber-500 bg-gray-600 border-gray-500"
                    />
                    <div className="flex-1">
                      <span className="font-medium text-gray-100">Branch</span>
                      {releaseType === 'branch' && (
                        <div className="mt-2">
                          <select
                            value={selectedBranch}
                            onChange={(e) => setSelectedBranch(e.target.value)}
                            className="w-full bg-gray-700 border border-gray-600 rounded-lg px-3 py-2 text-gray-100 focus:ring-amber-500 focus:border-amber-500"
                          >
                            {repoPreview.branches.map((branch) => (
                              <option key={branch.name} value={branch.name}>
                                {branch.name}
                                {branch.isDefault ? ' (default)' : ''}
                              </option>
                            ))}
                          </select>
                        </div>
                      )}
                      {releaseType !== 'branch' && (
                        <p className="text-sm text-gray-400 mt-0.5">
                          Download directly from a branch
                        </p>
                      )}
                    </div>
                  </label>
                </div>
              </div>
            </>
          )}

          {/* Action Buttons */}
          <div className="flex justify-end gap-2 pt-2">
            <Button variant="secondary" onClick={handleClose}>
              Cancel
            </Button>
            <Button
              onClick={handleAdd}
              disabled={!repoPreview}
              loading={isAdding}
            >
              Add Repository
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
};
