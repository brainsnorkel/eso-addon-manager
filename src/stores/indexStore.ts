import { create } from 'zustand';
import * as api from '../services/tauri';
import type { IndexAddon, IndexStats } from '../types/index';

export type SortOption = 'name' | 'updated';

interface IndexStore {
  addons: IndexAddon[];
  lastFetched: string | null;
  stats: IndexStats | null;
  loading: boolean;
  error: string | null;

  // Filters
  searchQuery: string;
  selectedTags: string[];

  // Sorting
  sortBy: SortOption;

  // Actions
  fetchIndex: (force?: boolean) => Promise<void>;
  fetchStats: () => Promise<void>;
  setSearchQuery: (query: string) => void;
  toggleTag: (tag: string) => void;
  clearFilters: () => void;
  setSortBy: (sort: SortOption) => void;

  // Computed
  filteredAddons: () => IndexAddon[];
  allTags: () => string[];
}

export const useIndexStore = create<IndexStore>((set, get) => ({
  addons: [],
  lastFetched: null,
  stats: null,
  loading: false,
  error: null,
  searchQuery: '',
  selectedTags: [],
  sortBy: 'name',

  fetchIndex: async (force = false) => {
    set({ loading: true, error: null });
    try {
      const index = await api.fetchIndex(force);
      set({
        addons: index.addons,
        lastFetched: index.fetched_at ?? index.generated_at,
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },

  fetchStats: async () => {
    try {
      const stats = await api.getIndexStats();
      set({ stats });
    } catch (e) {
      set({ error: String(e) });
    }
  },

  setSearchQuery: (query) => set({ searchQuery: query }),
  toggleTag: (tag) =>
    set((state) => ({
      selectedTags: state.selectedTags.includes(tag)
        ? state.selectedTags.filter((t) => t !== tag)
        : [...state.selectedTags, tag],
    })),
  clearFilters: () =>
    set({ searchQuery: '', selectedTags: [] }),
  setSortBy: (sort) => set({ sortBy: sort }),

  filteredAddons: () => {
    const { addons, searchQuery, selectedTags, sortBy } = get();

    const filtered = addons.filter((addon) => {
      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matches =
          addon.name.toLowerCase().includes(query) ||
          addon.description.toLowerCase().includes(query) ||
          addon.tags.some((t) => t.toLowerCase().includes(query)) ||
          addon.authors.some((a) => a.toLowerCase().includes(query));
        if (!matches) return false;
      }

      // Tags filter (addon must have ALL selected tags)
      if (selectedTags.length > 0) {
        if (!selectedTags.every((t) => addon.tags.includes(t))) {
          return false;
        }
      }

      return true;
    });

    // Sort the filtered results
    return filtered.sort((a, b) => {
      if (sortBy === 'updated') {
        // Sort by last_updated descending (most recent first)
        const dateA = a.last_updated ? new Date(a.last_updated).getTime() : 0;
        const dateB = b.last_updated ? new Date(b.last_updated).getTime() : 0;
        return dateB - dateA;
      }
      // Default: sort by name ascending
      return a.name.localeCompare(b.name);
    });
  },

  allTags: () => {
    const tags = new Set(get().addons.flatMap((a) => a.tags));
    return Array.from(tags).sort();
  },
}));
