import { FC, useState } from 'react';
import { Button } from '../common/Button';
import type { DependencyResult, ResolvedDependency } from '../../types/addon';

interface DependencyDialogProps {
  /** The addon being installed */
  addonName: string;
  /** Resolved dependency information */
  dependencies: DependencyResult;
  /** Called when user confirms installation */
  onConfirm: (selectedDeps: ResolvedDependency[]) => void;
  /** Called when user cancels */
  onCancel: () => void;
  /** Whether installation is in progress */
  installing?: boolean;
}

export const DependencyDialog: FC<DependencyDialogProps> = ({
  addonName,
  dependencies,
  onConfirm,
  onCancel,
  installing = false,
}) => {
  // Track which dependencies are selected (all selected by default)
  const [selectedSlugs, setSelectedSlugs] = useState<Set<string>>(
    new Set(dependencies.resolved.map((d) => d.slug))
  );

  const handleToggle = (slug: string) => {
    setSelectedSlugs((prev) => {
      const next = new Set(prev);
      if (next.has(slug)) {
        next.delete(slug);
      } else {
        next.add(slug);
      }
      return next;
    });
  };

  const handleConfirm = () => {
    const selectedDeps = dependencies.resolved.filter((d) =>
      selectedSlugs.has(d.slug)
    );
    onConfirm(selectedDeps);
  };

  const hasResolved = dependencies.resolved.length > 0;
  const hasAlreadyInstalled = dependencies.alreadyInstalled.length > 0;
  const hasUnresolved = dependencies.unresolved.length > 0;

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-gray-800 rounded-lg w-full max-w-lg border border-gray-700 shadow-xl">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-700">
          <h2 className="text-lg font-semibold text-gray-100">
            Install Dependencies
          </h2>
          <p className="text-sm text-gray-400 mt-1">
            <span className="font-medium text-gray-300">{addonName}</span> requires the following dependencies
          </p>
        </div>

        {/* Content */}
        <div className="px-6 py-4 max-h-[60vh] overflow-y-auto space-y-4">
          {/* Resolved dependencies - can be installed */}
          {hasResolved && (
            <div>
              <h3 className="text-sm font-medium text-gray-300 mb-2 flex items-center gap-2">
                <svg className="w-4 h-4 text-green-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                </svg>
                Available to Install ({dependencies.resolved.length})
              </h3>
              <div className="space-y-2">
                {dependencies.resolved.map((dep) => (
                  <label
                    key={dep.slug}
                    className="flex items-start gap-3 p-3 bg-gray-700/50 rounded-lg cursor-pointer hover:bg-gray-700 transition-colors"
                  >
                    <input
                      type="checkbox"
                      checked={selectedSlugs.has(dep.slug)}
                      onChange={() => handleToggle(dep.slug)}
                      disabled={installing}
                      className="mt-0.5 rounded bg-gray-600 border-gray-500 text-amber-500 focus:ring-amber-500 focus:ring-offset-gray-800"
                    />
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="font-medium text-gray-200">{dep.name}</span>
                        <span className="text-xs text-gray-500">v{dep.version}</span>
                      </div>
                      <span className="text-xs text-gray-500">{dep.slug}</span>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          )}

          {/* Already installed */}
          {hasAlreadyInstalled && (
            <div>
              <h3 className="text-sm font-medium text-gray-300 mb-2 flex items-center gap-2">
                <svg className="w-4 h-4 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                Already Installed ({dependencies.alreadyInstalled.length})
              </h3>
              <div className="flex flex-wrap gap-2">
                {dependencies.alreadyInstalled.map((slug) => (
                  <span
                    key={slug}
                    className="px-2 py-1 text-xs rounded bg-blue-900/30 text-blue-300 border border-blue-800/50"
                  >
                    {slug}
                  </span>
                ))}
              </div>
            </div>
          )}

          {/* Unresolved dependencies - not in index */}
          {hasUnresolved && (
            <div>
              <h3 className="text-sm font-medium text-gray-300 mb-2 flex items-center gap-2">
                <svg className="w-4 h-4 text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
                Not Found in Index ({dependencies.unresolved.length})
              </h3>
              <div className="p-3 bg-yellow-900/20 rounded-lg border border-yellow-800/50">
                <p className="text-xs text-yellow-200 mb-2">
                  These dependencies are not available in the addon index. You may need to install them manually from ESOUI or Minion.
                </p>
                <div className="flex flex-wrap gap-2">
                  {dependencies.unresolved.map((slug) => (
                    <span
                      key={slug}
                      className="px-2 py-1 text-xs rounded bg-yellow-900/30 text-yellow-300 border border-yellow-700/50"
                    >
                      {slug}
                    </span>
                  ))}
                </div>
              </div>
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-700 flex justify-end gap-3">
          <Button variant="secondary" onClick={onCancel} disabled={installing}>
            Cancel
          </Button>
          <Button onClick={handleConfirm} loading={installing}>
            {hasResolved && selectedSlugs.size > 0
              ? `Install ${selectedSlugs.size + 1} Addon${selectedSlugs.size > 0 ? 's' : ''}`
              : 'Install Addon'}
          </Button>
        </div>
      </div>
    </div>
  );
};
