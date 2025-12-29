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
  latest_release?: AddonRelease;
}

/** Source repository information */
export interface AddonSource {
  type: 'github' | 'gitlab' | 'custom';
  repo: string;
  branch: string;
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
