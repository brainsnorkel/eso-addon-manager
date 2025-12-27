# eso-addon-manager Implementation Guide

Version: 1.0  
Date: 27 December 2024

## Overview

A cross-platform desktop application for discovering, installing, and managing ESO addons. Built with Tauri (Rust backend + React frontend) for native performance with a modern UI.

---

## Technology Stack

| Layer | Technology | Rationale |
|-------|------------|-----------|
| Framework | Tauri 2.x | Small binary (~10 MB), native performance, Rust security |
| Backend | Rust | Memory safety, async I/O, cross-platform |
| Frontend | React 18 + TypeScript | Component ecosystem, type safety |
| Styling | Tailwind CSS | Utility-first, consistent design |
| State | Zustand | Lightweight, TypeScript-native |
| HTTP | reqwest (Rust) | Async, connection pooling |
| Storage | SQLite (rusqlite) | Local database, no server needed |
| Packaging | Tauri bundler | Native installers per platform |

---

## Repository Structure

```
eso-addon-manager/
├── .github/
│   └── workflows/
│       ├── ci.yml              # Build + test on PR
│       ├── release.yml         # Build release binaries
│       └── update-index.yml    # Refresh bundled index
├── src-tauri/
│   ├── src/
│   │   ├── main.rs             # Application entry point
│   │   ├── lib.rs              # Library exports
│   │   ├── commands/
│   │   │   ├── mod.rs
│   │   │   ├── addons.rs       # Addon CRUD operations
│   │   │   ├── index.rs        # Index fetch/sync
│   │   │   ├── settings.rs     # User preferences
│   │   │   └── github.rs       # Custom repo tracking
│   │   ├── models/
│   │   │   ├── mod.rs
│   │   │   ├── addon.rs        # Addon data structures
│   │   │   ├── index.rs        # Index schema
│   │   │   └── settings.rs     # Settings schema
│   │   ├── services/
│   │   │   ├── mod.rs
│   │   │   ├── downloader.rs   # HTTP download + extraction
│   │   │   ├── installer.rs    # Addon installation logic
│   │   │   ├── scanner.rs      # Local addon discovery
│   │   │   └── database.rs     # SQLite operations
│   │   └── utils/
│   │       ├── mod.rs
│   │       ├── paths.rs        # Platform-specific paths
│   │       ├── manifest.rs     # TOC file parsing
│   │       └── zip.rs          # Archive extraction
│   ├── migrations/
│   │   └── 001_initial.sql     # Database schema
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── build.rs
├── src/
│   ├── components/
│   │   ├── layout/
│   │   │   ├── Sidebar.tsx
│   │   │   ├── Header.tsx
│   │   │   └── MainContent.tsx
│   │   ├── addons/
│   │   │   ├── AddonCard.tsx
│   │   │   ├── AddonList.tsx
│   │   │   ├── AddonDetail.tsx
│   │   │   └── InstalledList.tsx
│   │   ├── search/
│   │   │   ├── SearchBar.tsx
│   │   │   ├── FilterPanel.tsx
│   │   │   └── CategoryFilter.tsx
│   │   └── common/
│   │       ├── Button.tsx
│   │       ├── Modal.tsx
│   │       ├── Toast.tsx
│   │       └── ProgressBar.tsx
│   ├── hooks/
│   │   ├── useAddons.ts
│   │   ├── useIndex.ts
│   │   ├── useSettings.ts
│   │   └── useDownload.ts
│   ├── stores/
│   │   ├── addonStore.ts
│   │   ├── indexStore.ts
│   │   └── settingsStore.ts
│   ├── services/
│   │   └── tauri.ts            # Tauri command wrappers
│   ├── types/
│   │   ├── addon.ts
│   │   ├── index.ts
│   │   └── settings.ts
│   ├── App.tsx
│   ├── main.tsx
│   └── index.css
├── public/
│   └── icons/
├── package.json
├── tsconfig.json
├── tailwind.config.js
├── vite.config.ts
├── LICENSE
└── README.md
```

---

## Platform-Specific Paths

### ESO Addon Directory

```rust
// src-tauri/src/utils/paths.rs

use std::path::PathBuf;
use directories::BaseDirs;

pub fn get_eso_addon_path() -> Option<PathBuf> {
    let base = BaseDirs::new()?;
    
    #[cfg(target_os = "windows")]
    {
        // C:\Users\{user}\Documents\Elder Scrolls Online\live\AddOns
        let docs = base.document_dir()?;
        Some(docs.join("Elder Scrolls Online").join("live").join("AddOns"))
    }
    
    #[cfg(target_os = "macos")]
    {
        // ~/Documents/Elder Scrolls Online/live/AddOns
        let docs = base.document_dir()?;
        Some(docs.join("Elder Scrolls Online").join("live").join("AddOns"))
    }
    
    #[cfg(target_os = "linux")]
    {
        // Steam: ~/.steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns
        // Lutris: Similar Wine prefix path
        let home = base.home_dir();
        let steam_path = home.join(".steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns");
        
        if steam_path.exists() {
            return Some(steam_path);
        }
        
        // Fallback: prompt user for path
        None
    }
}

pub fn get_app_data_path() -> Option<PathBuf> {
    let base = BaseDirs::new()?;
    
    #[cfg(target_os = "windows")]
    {
        Some(base.data_local_dir()?.join("eso-addon-manager"))
    }
    
    #[cfg(target_os = "macos")]
    {
        Some(base.data_dir()?.join("eso-addon-manager"))
    }
    
    #[cfg(target_os = "linux")]
    {
        Some(base.data_dir()?.join("eso-addon-manager"))
    }
}
```

### SavedVariables Location

```rust
pub fn get_saved_variables_path() -> Option<PathBuf> {
    let addon_path = get_eso_addon_path()?;
    // Navigate up and into SavedVariables
    addon_path.parent()?.join("SavedVariables").into()
}
```

---

## Database Schema

### migrations/001_initial.sql

```sql
-- Installed addons tracking
CREATE TABLE IF NOT EXISTS installed_addons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT UNIQUE NOT NULL,
    name TEXT NOT NULL,
    installed_version TEXT NOT NULL,
    source_type TEXT NOT NULL,  -- 'index' | 'github' | 'local'
    source_repo TEXT,           -- GitHub repo if applicable
    installed_at TEXT NOT NULL, -- ISO 8601 timestamp
    updated_at TEXT NOT NULL,
    auto_update INTEGER DEFAULT 1,
    manifest_path TEXT NOT NULL
);

-- Custom GitHub repositories (not in index)
CREATE TABLE IF NOT EXISTS custom_repos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    repo TEXT UNIQUE NOT NULL,       -- owner/repo
    branch TEXT DEFAULT 'main',
    release_type TEXT DEFAULT 'release',
    added_at TEXT NOT NULL,
    last_checked TEXT
);

-- Index cache
CREATE TABLE IF NOT EXISTS index_cache (
    id INTEGER PRIMARY KEY,
    data TEXT NOT NULL,              -- Full JSON
    fetched_at TEXT NOT NULL,
    etag TEXT                        -- For conditional requests
);

-- Application settings
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Download history/queue
CREATE TABLE IF NOT EXISTS downloads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    addon_slug TEXT NOT NULL,
    version TEXT NOT NULL,
    status TEXT NOT NULL,            -- 'pending' | 'downloading' | 'extracting' | 'complete' | 'failed'
    progress REAL DEFAULT 0,
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_installed_slug ON installed_addons(slug);
CREATE INDEX IF NOT EXISTS idx_downloads_status ON downloads(status);
```

---

## Core Rust Commands

### src-tauri/src/commands/addons.rs

```rust
use crate::models::addon::{InstalledAddon, AddonInfo};
use crate::services::{database, installer, scanner};
use tauri::State;
use std::sync::Mutex;

pub struct AppState {
    pub db: Mutex<rusqlite::Connection>,
}

/// Get all installed addons
#[tauri::command]
pub async fn get_installed_addons(
    state: State<'_, AppState>
) -> Result<Vec<InstalledAddon>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_installed(&conn).map_err(|e| e.to_string())
}

/// Install an addon from the index
#[tauri::command]
pub async fn install_addon(
    slug: String,
    version: String,
    download_url: String,
    state: State<'_, AppState>,
    window: tauri::Window,
) -> Result<InstalledAddon, String> {
    // Emit progress events to frontend
    window.emit("download-progress", &DownloadProgress {
        slug: slug.clone(),
        status: "downloading".into(),
        progress: 0.0,
    }).ok();
    
    // Download
    let archive_path = installer::download_addon(&download_url, |progress| {
        window.emit("download-progress", &DownloadProgress {
            slug: slug.clone(),
            status: "downloading".into(),
            progress,
        }).ok();
    }).await.map_err(|e| e.to_string())?;
    
    // Extract
    window.emit("download-progress", &DownloadProgress {
        slug: slug.clone(),
        status: "extracting".into(),
        progress: 0.0,
    }).ok();
    
    let addon_path = crate::utils::paths::get_eso_addon_path()
        .ok_or("Could not find ESO addon directory")?;
    
    installer::extract_addon(&archive_path, &addon_path)
        .map_err(|e| e.to_string())?;
    
    // Update database
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let addon = database::insert_installed(
        &conn, 
        &slug, 
        &version, 
        "index",
        None
    ).map_err(|e| e.to_string())?;
    
    window.emit("download-progress", &DownloadProgress {
        slug: slug.clone(),
        status: "complete".into(),
        progress: 1.0,
    }).ok();
    
    Ok(addon)
}

/// Uninstall an addon
#[tauri::command]
pub async fn uninstall_addon(
    slug: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    
    // Get addon info
    let addon = database::get_installed_by_slug(&conn, &slug)
        .map_err(|e| e.to_string())?
        .ok_or("Addon not found")?;
    
    // Delete files
    installer::remove_addon(&addon.manifest_path)
        .map_err(|e| e.to_string())?;
    
    // Remove from database
    database::delete_installed(&conn, &slug)
        .map_err(|e| e.to_string())
}

/// Scan local addon directory for addons not tracked by the manager
#[tauri::command]
pub async fn scan_local_addons() -> Result<Vec<AddonInfo>, String> {
    let addon_path = crate::utils::paths::get_eso_addon_path()
        .ok_or("Could not find ESO addon directory")?;
    
    scanner::scan_directory(&addon_path)
        .map_err(|e| e.to_string())
}

/// Check for updates for all installed addons
#[tauri::command]
pub async fn check_updates(
    state: State<'_, AppState>
) -> Result<Vec<UpdateInfo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let installed = database::get_all_installed(&conn)
        .map_err(|e| e.to_string())?;
    
    let index = database::get_cached_index(&conn)
        .map_err(|e| e.to_string())?;
    
    let mut updates = Vec::new();
    
    for addon in installed {
        if let Some(index_entry) = index.addons.iter().find(|a| a.slug == addon.slug) {
            if let Some(release) = &index_entry.latest_release {
                if release.version != addon.installed_version {
                    updates.push(UpdateInfo {
                        slug: addon.slug,
                        name: addon.name,
                        current_version: addon.installed_version,
                        new_version: release.version.clone(),
                        download_url: release.download_url.clone(),
                    });
                }
            }
        }
    }
    
    Ok(updates)
}

#[derive(serde::Serialize, Clone)]
pub struct DownloadProgress {
    pub slug: String,
    pub status: String,
    pub progress: f64,
}

#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub slug: String,
    pub name: String,
    pub current_version: String,
    pub new_version: String,
    pub download_url: String,
}
```

### src-tauri/src/commands/github.rs

```rust
use crate::models::addon::CustomRepo;
use crate::services::database;
use tauri::State;
use super::addons::AppState;

/// Add a custom GitHub repository to track
#[tauri::command]
pub async fn add_custom_repo(
    repo: String,           // owner/repo format
    branch: Option<String>,
    release_type: Option<String>,
    state: State<'_, AppState>,
) -> Result<CustomRepo, String> {
    // Validate repo exists
    let client = reqwest::Client::new();
    let url = format!("https://api.github.com/repos/{}", repo);
    
    let response = client.get(&url)
        .header("User-Agent", "eso-addon-manager")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    if !response.status().is_success() {
        return Err(format!("Repository not found: {}", repo));
    }
    
    // Check for ESO addon manifest
    let contents_url = format!("https://api.github.com/repos/{}/contents", repo);
    let contents: Vec<GithubContent> = client.get(&contents_url)
        .header("User-Agent", "eso-addon-manager")
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    let has_manifest = contents.iter()
        .any(|f| f.name.ends_with(".txt") && f.content_type == "file");
    
    if !has_manifest {
        return Err("No addon manifest (.txt) found in repository root".into());
    }
    
    // Save to database
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::insert_custom_repo(
        &conn,
        &repo,
        branch.as_deref().unwrap_or("main"),
        release_type.as_deref().unwrap_or("release"),
    ).map_err(|e| e.to_string())
}

/// Get all custom tracked repositories
#[tauri::command]
pub async fn get_custom_repos(
    state: State<'_, AppState>
) -> Result<Vec<CustomRepo>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_custom_repos(&conn).map_err(|e| e.to_string())
}

/// Remove a custom repository
#[tauri::command]
pub async fn remove_custom_repo(
    repo: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    database::delete_custom_repo(&conn, &repo).map_err(|e| e.to_string())
}

#[derive(serde::Deserialize)]
struct GithubContent {
    name: String,
    #[serde(rename = "type")]
    content_type: String,
}
```

---

## TOC Manifest Parsing

### src-tauri/src/utils/manifest.rs

```rust
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize)]
pub struct AddonManifest {
    pub title: String,
    pub api_version: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub dependencies: Vec<String>,
    pub optional_dependencies: Vec<String>,
    pub saved_variables: Vec<String>,
    pub files: Vec<String>,
}

pub fn parse_manifest(path: &Path) -> Result<AddonManifest, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let mut meta: HashMap<String, String> = HashMap::new();
    let mut files: Vec<String> = Vec::new();
    
    for line in content.lines() {
        let line = line.trim();
        
        if line.starts_with("## ") {
            // Metadata line
            if let Some(colon_pos) = line.find(':') {
                let key = line[3..colon_pos].trim().to_lowercase();
                let value = line[colon_pos + 1..].trim().to_string();
                meta.insert(key, value);
            }
        } else if !line.is_empty() && !line.starts_with(";") && !line.starts_with("#") {
            // File reference
            files.push(line.to_string());
        }
    }
    
    Ok(AddonManifest {
        title: meta.get("title").cloned().unwrap_or_default(),
        api_version: meta.get("apiversion").cloned(),
        author: meta.get("author").cloned(),
        version: meta.get("version").cloned()
            .or_else(|| meta.get("adddonversion").cloned()),
        description: meta.get("description").cloned(),
        dependencies: parse_dependency_list(meta.get("dependson")),
        optional_dependencies: parse_dependency_list(meta.get("optionaldependson")),
        saved_variables: parse_dependency_list(meta.get("savedvariables")),
        files,
    })
}

fn parse_dependency_list(value: Option<&String>) -> Vec<String> {
    value
        .map(|s| s.split_whitespace().map(String::from).collect())
        .unwrap_or_default()
}

/// Find all manifest files in an addon directory
pub fn find_manifests(addon_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut manifests = Vec::new();
    
    if let Ok(entries) = fs::read_dir(addon_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "txt").unwrap_or(false) {
                // Check if it's actually a manifest (has ## Title:)
                if let Ok(content) = fs::read_to_string(&path) {
                    if content.contains("## Title:") {
                        manifests.push(path);
                    }
                }
            }
        }
    }
    
    manifests
}
```

---

## Frontend Architecture

### src/stores/addonStore.ts

```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

interface InstalledAddon {
  slug: string;
  name: string;
  installedVersion: string;
  sourceType: 'index' | 'github' | 'local';
  sourceRepo?: string;
  installedAt: string;
  updatedAt: string;
  autoUpdate: boolean;
}

interface DownloadProgress {
  slug: string;
  status: 'downloading' | 'extracting' | 'complete' | 'failed';
  progress: number;
}

interface AddonStore {
  installed: InstalledAddon[];
  downloads: Map<string, DownloadProgress>;
  loading: boolean;
  error: string | null;
  
  fetchInstalled: () => Promise<void>;
  installAddon: (slug: string, version: string, downloadUrl: string) => Promise<void>;
  uninstallAddon: (slug: string) => Promise<void>;
  checkUpdates: () => Promise<UpdateInfo[]>;
}

export const useAddonStore = create<AddonStore>((set, get) => ({
  installed: [],
  downloads: new Map(),
  loading: false,
  error: null,
  
  fetchInstalled: async () => {
    set({ loading: true, error: null });
    try {
      const installed = await invoke<InstalledAddon[]>('get_installed_addons');
      set({ installed, loading: false });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  
  installAddon: async (slug, version, downloadUrl) => {
    // Listen for progress updates
    const unlisten = await listen<DownloadProgress>('download-progress', (event) => {
      set((state) => {
        const downloads = new Map(state.downloads);
        downloads.set(event.payload.slug, event.payload);
        return { downloads };
      });
    });
    
    try {
      await invoke('install_addon', { slug, version, downloadUrl });
      await get().fetchInstalled();
    } finally {
      unlisten();
    }
  },
  
  uninstallAddon: async (slug) => {
    try {
      await invoke('uninstall_addon', { slug });
      set((state) => ({
        installed: state.installed.filter((a) => a.slug !== slug),
      }));
    } catch (e) {
      set({ error: String(e) });
    }
  },
  
  checkUpdates: async () => {
    return invoke<UpdateInfo[]>('check_updates');
  },
}));

interface UpdateInfo {
  slug: string;
  name: string;
  currentVersion: string;
  newVersion: string;
  downloadUrl: string;
}
```

### src/stores/indexStore.ts

```typescript
import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/core';

interface IndexAddon {
  slug: string;
  name: string;
  description: string;
  authors: string[];
  license: string;
  category: string;
  tags: string[];
  source: {
    type: 'github' | 'gitlab' | 'custom';
    repo: string;
    branch: string;
  };
  compatibility: {
    apiVersion: string;
    gameVersions: string[];
    requiredDependencies: string[];
    optionalDependencies: string[];
  };
  latestRelease?: {
    version: string;
    downloadUrl: string;
    publishedAt: string;
  };
}

interface IndexStore {
  addons: IndexAddon[];
  lastFetched: string | null;
  loading: boolean;
  error: string | null;
  
  // Filters
  searchQuery: string;
  selectedCategory: string | null;
  selectedTags: string[];
  
  // Actions
  fetchIndex: (force?: boolean) => Promise<void>;
  setSearchQuery: (query: string) => void;
  setCategory: (category: string | null) => void;
  toggleTag: (tag: string) => void;
  
  // Computed
  filteredAddons: () => IndexAddon[];
  categories: () => string[];
  allTags: () => string[];
}

export const useIndexStore = create<IndexStore>((set, get) => ({
  addons: [],
  lastFetched: null,
  loading: false,
  error: null,
  searchQuery: '',
  selectedCategory: null,
  selectedTags: [],
  
  fetchIndex: async (force = false) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<{ addons: IndexAddon[]; fetchedAt: string }>(
        'fetch_index',
        { force }
      );
      set({
        addons: result.addons,
        lastFetched: result.fetchedAt,
        loading: false,
      });
    } catch (e) {
      set({ error: String(e), loading: false });
    }
  },
  
  setSearchQuery: (query) => set({ searchQuery: query }),
  setCategory: (category) => set({ selectedCategory: category }),
  toggleTag: (tag) => set((state) => ({
    selectedTags: state.selectedTags.includes(tag)
      ? state.selectedTags.filter((t) => t !== tag)
      : [...state.selectedTags, tag],
  })),
  
  filteredAddons: () => {
    const { addons, searchQuery, selectedCategory, selectedTags } = get();
    
    return addons.filter((addon) => {
      // Search filter
      if (searchQuery) {
        const query = searchQuery.toLowerCase();
        const matches =
          addon.name.toLowerCase().includes(query) ||
          addon.description.toLowerCase().includes(query) ||
          addon.tags.some((t) => t.toLowerCase().includes(query));
        if (!matches) return false;
      }
      
      // Category filter
      if (selectedCategory && addon.category !== selectedCategory) {
        return false;
      }
      
      // Tags filter (addon must have ALL selected tags)
      if (selectedTags.length > 0) {
        if (!selectedTags.every((t) => addon.tags.includes(t))) {
          return false;
        }
      }
      
      return true;
    });
  },
  
  categories: () => {
    const cats = new Set(get().addons.map((a) => a.category));
    return Array.from(cats).sort();
  },
  
  allTags: () => {
    const tags = new Set(get().addons.flatMap((a) => a.tags));
    return Array.from(tags).sort();
  },
}));
```

---

## UI Components

### src/components/addons/AddonCard.tsx

```tsx
import { FC } from 'react';
import { useAddonStore } from '../../stores/addonStore';

interface Props {
  addon: {
    slug: string;
    name: string;
    description: string;
    authors: string[];
    category: string;
    latestRelease?: {
      version: string;
      downloadUrl: string;
    };
  };
}

export const AddonCard: FC<Props> = ({ addon }) => {
  const { installed, downloads, installAddon, uninstallAddon } = useAddonStore();
  
  const isInstalled = installed.some((i) => i.slug === addon.slug);
  const installedVersion = installed.find((i) => i.slug === addon.slug)?.installedVersion;
  const downloadState = downloads.get(addon.slug);
  
  const hasUpdate =
    isInstalled &&
    addon.latestRelease &&
    installedVersion !== addon.latestRelease.version;
  
  const handleInstall = async () => {
    if (!addon.latestRelease) return;
    await installAddon(
      addon.slug,
      addon.latestRelease.version,
      addon.latestRelease.downloadUrl
    );
  };
  
  const handleUninstall = async () => {
    await uninstallAddon(addon.slug);
  };
  
  return (
    <div className="bg-white dark:bg-gray-800 rounded-lg shadow p-4 hover:shadow-md transition-shadow">
      <div className="flex justify-between items-start">
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-lg truncate">{addon.name}</h3>
          <p className="text-sm text-gray-500 dark:text-gray-400">
            {addon.authors.join(', ')}
          </p>
        </div>
        <span className="px-2 py-1 text-xs rounded-full bg-gray-100 dark:bg-gray-700">
          {addon.category}
        </span>
      </div>
      
      <p className="mt-2 text-sm text-gray-600 dark:text-gray-300 line-clamp-2">
        {addon.description}
      </p>
      
      <div className="mt-4 flex items-center justify-between">
        <span className="text-sm text-gray-500">
          {addon.latestRelease?.version ?? 'No release'}
        </span>
        
        <div className="flex gap-2">
          {downloadState && downloadState.status !== 'complete' ? (
            <div className="flex items-center gap-2">
              <div className="w-24 h-2 bg-gray-200 rounded-full overflow-hidden">
                <div
                  className="h-full bg-blue-500 transition-all"
                  style={{ width: `${downloadState.progress * 100}%` }}
                />
              </div>
              <span className="text-xs text-gray-500">{downloadState.status}</span>
            </div>
          ) : isInstalled ? (
            <>
              {hasUpdate && (
                <button
                  onClick={handleInstall}
                  className="px-3 py-1 text-sm bg-green-500 hover:bg-green-600 text-white rounded"
                >
                  Update
                </button>
              )}
              <button
                onClick={handleUninstall}
                className="px-3 py-1 text-sm bg-red-500 hover:bg-red-600 text-white rounded"
              >
                Remove
              </button>
            </>
          ) : (
            <button
              onClick={handleInstall}
              disabled={!addon.latestRelease}
              className="px-3 py-1 text-sm bg-blue-500 hover:bg-blue-600 disabled:bg-gray-300 text-white rounded"
            >
              Install
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
```

### src/components/search/SearchBar.tsx

```tsx
import { FC, useState, useEffect } from 'react';
import { useIndexStore } from '../../stores/indexStore';
import { useDebouncedValue } from '../../hooks/useDebouncedValue';

export const SearchBar: FC = () => {
  const [input, setInput] = useState('');
  const setSearchQuery = useIndexStore((s) => s.setSearchQuery);
  const debouncedInput = useDebouncedValue(input, 200);
  
  useEffect(() => {
    setSearchQuery(debouncedInput);
  }, [debouncedInput, setSearchQuery]);
  
  return (
    <div className="relative">
      <input
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        placeholder="Search addons..."
        className="w-full px-4 py-2 pl-10 rounded-lg border border-gray-300 dark:border-gray-600 dark:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
      />
      <svg
        className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400"
        fill="none"
        stroke="currentColor"
        viewBox="0 0 24 24"
      >
        <path
          strokeLinecap="round"
          strokeLinejoin="round"
          strokeWidth={2}
          d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
        />
      </svg>
    </div>
  );
};
```

---

## GitHub Actions Workflows

### .github/workflows/release.yml

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      fail-fast: false
      matrix:
        include:
          - platform: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
          - platform: macos-latest
            target: x86_64-apple-darwin
          - platform: macos-latest
            target: aarch64-apple-darwin
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
    
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install Linux dependencies
        if: matrix.platform == 'ubuntu-22.04'
        run: |
          sudo apt-get update
          sudo apt-get install -y libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
      
      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: npm
      
      - name: Install frontend dependencies
        run: npm ci
      
      - name: Build Tauri app
        uses: tauri-apps/tauri-action@v0
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_KEY_PASSWORD }}
        with:
          tagName: ${{ github.ref_name }}
          releaseName: 'ESO Addon Manager ${{ github.ref_name }}'
          releaseBody: 'See CHANGELOG.md for release notes.'
          releaseDraft: true
          prerelease: false
          args: --target ${{ matrix.target }}
```

---

## Tauri Configuration

### src-tauri/tauri.conf.json

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "ESO Addon Manager",
  "identifier": "com.eso-addon-manager.app",
  "version": "0.1.0",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ESO Addon Manager",
        "width": 1200,
        "height": 800,
        "minWidth": 900,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; img-src 'self' https://avatars.githubusercontent.com; connect-src 'self' https://api.github.com https://*.github.io"
    }
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "linux": {
      "appimage": {
        "bundleMediaFramework": false
      }
    },
    "macOS": {
      "minimumSystemVersion": "10.15"
    }
  },
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/your-org/eso-addon-manager/releases/latest/download/latest.json"
      ],
      "pubkey": "YOUR_PUBLIC_KEY_HERE"
    }
  }
}
```

---

## Development Setup

### Prerequisites

- Rust 1.75+ (via rustup)
- Node.js 20+
- Platform-specific requirements:
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: WebView2 Runtime, Visual Studio Build Tools

### Quick Start

```bash
# Clone repository
git clone https://github.com/your-org/eso-addon-manager.git
cd eso-addon-manager

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### Project Scripts

```json
{
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "tauri": "tauri",
    "lint": "eslint src --ext .ts,.tsx",
    "test": "vitest"
  }
}
```

---

## Error Handling Strategy

### Rust Backend

```rust
// src-tauri/src/error.rs

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("Archive error: {0}")]
    Archive(#[from] zip::result::ZipError),
    
    #[error("Addon not found: {0}")]
    AddonNotFound(String),
    
    #[error("Invalid manifest: {0}")]
    InvalidManifest(String),
    
    #[error("ESO directory not found")]
    EsoDirectoryNotFound,
}

// Convert to string for Tauri command returns
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
}
```

### Frontend Error Display

```tsx
// src/components/common/ErrorToast.tsx

import { FC, useEffect } from 'react';

interface Props {
  message: string;
  onClose: () => void;
}

export const ErrorToast: FC<Props> = ({ message, onClose }) => {
  useEffect(() => {
    const timer = setTimeout(onClose, 5000);
    return () => clearTimeout(timer);
  }, [onClose]);
  
  return (
    <div className="fixed bottom-4 right-4 bg-red-500 text-white px-4 py-3 rounded-lg shadow-lg flex items-center gap-3">
      <span>{message}</span>
      <button onClick={onClose} className="hover:bg-red-600 rounded p-1">
        ✕
      </button>
    </div>
  );
};
```

---

## References

- [Tauri Documentation](https://tauri.app/v1/guides/)
- [Tauri 2.0 Migration Guide](https://v2.tauri.app/start/migrate/from-tauri-1/)
- [ESO API Documentation](https://wiki.esoui.com/API)
- [rusqlite Documentation](https://docs.rs/rusqlite/)
- [Zustand Documentation](https://docs.pmnd.rs/zustand/)
- [WowUp Architecture](https://github.com/WowUp/WowUp) — Reference implementation
