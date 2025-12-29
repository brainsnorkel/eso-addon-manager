/** The addon index containing all available addons */
export interface AddonIndex {
  version: string;
  generated_at: string;
  addon_count: number;
  addons: IndexAddon[];
  fetched_at?: string;
}

/** An addon entry from the index */
export interface IndexAddon {
  slug: string;
  name: string;
  description: string;
  authors: string[];
  license?: string;
  category: string;
  tags: string[];
  url?: string; // Link to addon docs/homepage
  source: AddonSource;
  compatibility: AddonCompatibility;
  install: InstallInfo;
  latest_release?: AddonRelease;
}

/** Source repository information */
export interface AddonSource {
  type: 'github' | 'gitlab' | 'custom';
  repo: string;
  branch: string;
  path?: string; // Optional path within the repo for monorepo structures
}

/** Installation instructions for an addon */
export interface InstallInfo {
  method: 'branch' | 'github_release' | 'github_archive';
  extract_path?: string; // Path within the archive to extract from (null for root-level addons)
  target_folder: string; // Target folder name in the ESO AddOns directory
  excludes: string[]; // Glob patterns for files/directories to exclude
}

/** Compatibility information for an addon */
export interface AddonCompatibility {
  api_version?: string;
  game_versions: string[];
  required_dependencies: string[];
  optional_dependencies: string[];
}

/** Release information for an addon */
export interface AddonRelease {
  version: string;
  download_url: string;
  published_at?: string;
  file_size?: number;
  checksum?: string;
}

/** Index statistics */
export interface IndexStats {
  total_addons: number;
  categories: [string, number][];
  fetched_at: string;
}
