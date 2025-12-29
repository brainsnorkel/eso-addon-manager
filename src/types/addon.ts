/** Represents an addon installed on the local system */
export interface InstalledAddon {
  id: number;
  slug: string;
  name: string;
  installedVersion: string;
  sourceType: 'index' | 'github' | 'local';
  sourceRepo?: string;
  installedAt: string;
  updatedAt: string;
  autoUpdate: boolean;
  manifestPath: string;
  /** Pre-computed sort key from index for version comparison (index addons only) */
  versionSortKey?: number;
  /** Commit SHA for branch-based version tracking */
  commitSha?: string;
}

/** Version tracking info for simplified update detection */
export interface VersionTracking {
  /** Pre-computed sort key from index for direct integer comparison */
  versionSortKey?: number;
  /** Commit SHA for branch-based version tracking */
  commitSha?: string;
}

/** Parsed addon manifest from .txt file */
export interface AddonManifest {
  title: string;
  apiVersion?: string;
  author?: string;
  version?: string;
  description?: string;
  dependencies: string[];
  optionalDependencies: string[];
  savedVariables: string[];
  files: string[];
}

/** Information about an available update */
export interface UpdateInfo {
  slug: string;
  name: string;
  currentVersion: string;
  newVersion: string;
  downloadUrl: string;
  sourceType: 'index' | 'github' | 'local';
  sourceRepo?: string;
  installInfo?: import('./index').InstallInfo;
}

/** Download progress event */
export interface DownloadProgress {
  slug: string;
  status: 'pending' | 'downloading' | 'extracting' | 'complete' | 'failed';
  progress: number;
  error?: string;
}

/** Locally scanned addon info */
export interface ScannedAddon {
  name: string;
  path: string;
  manifest: AddonManifest;
  hasSavedVariables: boolean;
}
