import { create } from 'zustand';
import * as api from '../services/tauri';
import type { AppSettings } from '../types/settings';

interface SettingsStore {
  settings: AppSettings | null;
  addonDirectory: string | null;
  loading: boolean;
  error: string | null;

  // Actions
  fetchSettings: () => Promise<void>;
  updateSettings: (settings: Partial<AppSettings>) => Promise<void>;
  resetSettings: () => Promise<void>;
  fetchAddonDirectory: () => Promise<void>;
  setAddonDirectory: (path: string) => Promise<void>;
  clearError: () => void;
}

const defaultSettings: AppSettings = {
  checkUpdatesOnStartup: true,
  autoUpdate: false,
  theme: 'system',
};

export const useSettingsStore = create<SettingsStore>((set, get) => ({
  settings: null,
  addonDirectory: null,
  loading: false,
  error: null,

  fetchSettings: async () => {
    set({ loading: true, error: null });
    try {
      const settings = await api.getSettings();
      set({ settings, loading: false });
    } catch (e) {
      set({ settings: defaultSettings, error: String(e), loading: false });
    }
  },

  updateSettings: async (updates) => {
    const current = get().settings ?? defaultSettings;
    const newSettings = { ...current, ...updates };
    set({ settings: newSettings });
    try {
      await api.updateSettings(newSettings);
    } catch (e) {
      set({ error: String(e) });
    }
  },

  resetSettings: async () => {
    set({ loading: true, error: null });
    try {
      const settings = await api.resetSettings();
      set({ settings, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchAddonDirectory: async () => {
    try {
      const addonDirectory = await api.getAddonDirectory();
      set({ addonDirectory });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setAddonDirectory: async (path) => {
    try {
      await api.setAddonDirectory(path);
      set({ addonDirectory: path });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  clearError: () => set({ error: null }),
}));
