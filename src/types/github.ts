/** Custom GitHub repository tracked by the manager */
export interface CustomRepo {
  id: number;
  repo: string;
  branch: string;
  releaseType: 'release' | 'branch';
  addedAt: string;
  lastChecked?: string;
}

/** GitHub repository information */
export interface GitHubRepoInfo {
  name: string;
  description?: string;
  defaultBranch: string;
  stars: number;
  updatedAt?: string;
  hasReleases: boolean;
}

/** GitHub branch information */
export interface GitHubBranchInfo {
  name: string;
  isDefault: boolean;
}

/** GitHub release information */
export interface GitHubReleaseInfo {
  tagName: string;
  name?: string;
  downloadUrl: string;
  publishedAt?: string;
}

/** Repository preview with all info needed for the add modal */
export interface RepoPreview {
  name: string;
  description?: string;
  stars: number;
  defaultBranch: string;
  branches: GitHubBranchInfo[];
  hasReleases: boolean;
  latestRelease?: GitHubReleaseInfo;
  updatedAt?: string;
}
