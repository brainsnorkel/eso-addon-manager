import { create } from 'zustand';
import * as api from '../services/tauri';
import type { CustomRepo, GitHubRepoInfo, GitHubReleaseInfo, RepoPreview } from '../types/github';
import type { InstalledAddon } from '../types/addon';

interface GitHubStore {
  repos: CustomRepo[];
  repoInfo: Map<string, GitHubRepoInfo>;
  releaseInfo: Map<string, GitHubReleaseInfo>;
  loading: boolean;
  installing: string | null; // repo being installed
  error: string | null;

  // Preview state for add modal
  repoPreview: RepoPreview | null;
  previewLoading: boolean;
  previewError: string | null;

  // Actions
  fetchRepos: () => Promise<void>;
  addRepo: (repo: string, branch?: string, releaseType?: string) => Promise<void>;
  removeRepo: (repo: string) => Promise<void>;
  fetchRepoInfo: (repo: string) => Promise<GitHubRepoInfo>;
  fetchRelease: (repo: string) => Promise<GitHubReleaseInfo | null>;
  installFromRepo: (repo: string, releaseType?: string, branch?: string) => Promise<InstalledAddon>;
  clearError: () => void;

  // Preview actions
  fetchRepoPreview: (repo: string) => Promise<RepoPreview>;
  clearPreview: () => void;
}

export const useGitHubStore = create<GitHubStore>((set) => ({
  repos: [],
  repoInfo: new Map(),
  releaseInfo: new Map(),
  loading: false,
  installing: null,
  error: null,

  // Preview state
  repoPreview: null,
  previewLoading: false,
  previewError: null,

  fetchRepos: async () => {
    set({ loading: true, error: null });
    try {
      const repos = await api.getCustomRepos();
      set({ repos, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  addRepo: async (repo, branch, releaseType) => {
    set({ loading: true, error: null });
    try {
      const newRepo = await api.addCustomRepo(repo, branch, releaseType);
      set((state) => ({
        repos: [...state.repos, newRepo],
        loading: false,
      }));
    } catch (e) {
      set({ error: String(e), loading: false });
      throw e;
    }
  },

  removeRepo: async (repo) => {
    try {
      await api.removeCustomRepo(repo);
      set((state) => ({
        repos: state.repos.filter((r) => r.repo !== repo),
        repoInfo: new Map(
          Array.from(state.repoInfo.entries()).filter(([key]) => key !== repo)
        ),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  fetchRepoInfo: async (repo) => {
    try {
      const info = await api.getGitHubRepoInfo(repo);
      set((state) => {
        const repoInfo = new Map(state.repoInfo);
        repoInfo.set(repo, info);
        return { repoInfo };
      });
      return info;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  fetchRelease: async (repo) => {
    try {
      const release = await api.getGitHubRelease(repo);
      if (release) {
        set((state) => {
          const releaseInfo = new Map(state.releaseInfo);
          releaseInfo.set(repo, release);
          return { releaseInfo };
        });
      }
      return release;
    } catch (e) {
      set({ error: String(e) });
      throw e;
    }
  },

  installFromRepo: async (repo, releaseType, branch) => {
    set({ installing: repo, error: null });
    try {
      const addon = await api.installFromGitHub(repo, releaseType, branch);
      set({ installing: null });
      return addon;
    } catch (e) {
      set({ installing: null, error: String(e) });
      throw e;
    }
  },

  clearError: () => set({ error: null }),

  fetchRepoPreview: async (repo) => {
    set({ previewLoading: true, previewError: null, repoPreview: null });
    try {
      const preview = await api.getGitHubRepoPreview(repo);
      set({ repoPreview: preview, previewLoading: false });
      return preview;
    } catch (e) {
      set({ previewError: String(e), previewLoading: false });
      throw e;
    }
  },

  clearPreview: () => set({ repoPreview: null, previewError: null, previewLoading: false }),
}));
