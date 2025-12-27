/** The addon index containing all available addons */
export interface AddonIndex {
  addons: IndexAddon[];
  fetchedAt: string;
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
  source: AddonSource;
  compatibility: AddonCompatibility;
  latestRelease?: AddonRelease;
}

/** Source repository information */
export interface AddonSource {
  type: 'github' | 'gitlab' | 'custom';
  repo: string;
  branch: string;
}

/** Compatibility information for an addon */
export interface AddonCompatibility {
  apiVersion?: string;
  gameVersions: string[];
  requiredDependencies: string[];
  optionalDependencies: string[];
}

/** Release information for an addon */
export interface AddonRelease {
  version: string;
  downloadUrl: string;
  publishedAt: string;
  fileSize?: number;
  checksum?: string;
}

/** Index statistics */
export interface IndexStats {
  totalAddons: number;
  categories: [string, number][];
  fetchedAt: string;
}
