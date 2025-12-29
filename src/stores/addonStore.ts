import { create } from 'zustand';
import { listen } from '@tauri-apps/api/event';
import * as api from '../services/tauri';
import type { InstalledAddon, UpdateInfo, DownloadProgress, ScannedAddon } from '../types/addon';
import type { InstallInfo } from '../types/index';

interface AddonStore {
  installed: InstalledAddon[];
  updates: UpdateInfo[];
  downloads: Map<string, DownloadProgress>;
  scannedAddons: ScannedAddon[];
  loading: boolean;
  error: string | null;

  // Actions
  fetchInstalled: () => Promise<void>;
  installAddon: (slug: string, name: string, version: string, downloadUrl: string, installInfo?: InstallInfo) => Promise<void>;
  uninstallAddon: (slug: string) => Promise<void>;
  checkUpdates: () => Promise<void>;
  scanLocalAddons: () => Promise<void>;
  clearError: () => void;
}

export const useAddonStore = create<AddonStore>((set, get) => ({
  installed: [],
  updates: [],
  downloads: new Map(),
  scannedAddons: [],
  loading: false,
  error: null,

  fetchInstalled: async () => {
    set({ loading: true, error: null });
    try {
      const installed = await api.getInstalledAddons();
      set({ installed, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  installAddon: async (slug, name, version, downloadUrl, installInfo) => {
    // Listen for progress updates
    const unlisten = await listen<DownloadProgress>('download-progress', (event) => {
      set((state) => {
        const downloads = new Map(state.downloads);
        downloads.set(event.payload.slug, event.payload);
        return { downloads };
      });
    });

    try {
      await api.installAddon(slug, name, version, downloadUrl, undefined, undefined, installInfo);
      await get().fetchInstalled();
    } catch (e) {
      set({ error: String(e) });
    } finally {
      unlisten();
      // Clean up download state after a delay
      setTimeout(() => {
        set((state) => {
          const downloads = new Map(state.downloads);
          downloads.delete(slug);
          return { downloads };
        });
      }, 2000);
    }
  },

  uninstallAddon: async (slug) => {
    try {
      await api.uninstallAddon(slug);
      set((state) => ({
        installed: state.installed.filter((a) => a.slug !== slug),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },

  checkUpdates: async () => {
    try {
      const updates = await api.checkUpdates();
      set({ updates });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  scanLocalAddons: async () => {
    set({ loading: true, error: null });
    try {
      const scannedAddons = await api.scanLocalAddons();
      set({ scannedAddons, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  clearError: () => set({ error: null }),
}));
