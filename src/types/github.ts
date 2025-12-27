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
