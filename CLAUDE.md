# ESO Addon Manager

## Project Overview

A cross-platform desktop application for discovering, installing, and managing Elder Scrolls Online (ESO) addons. Built with Tauri 2.x (Rust backend + React frontend) for native performance with a modern UI.

**Status**: IN DEVELOPMENT - Core Features Complete
**License**: MIT
**Target**: Public release for ESO community

---

## Current Features

- [x] Tauri 2.x project with React + TypeScript
- [x] Tailwind CSS 4 configuration
- [x] Rust backend modules (commands, models, services, utils)
- [x] SQLite database schema and migrations
- [x] React frontend with Zustand state management
- [x] TypeScript type definitions
- [x] Basic UI components (Sidebar, Header, AddonCard, etc.)
- [x] GitHub Actions CI/CD workflows
- [x] Multi-source download with jsDelivr CDN fallback
- [x] GitHub repository tracking with branch/release selection
- [x] Dependency status highlighting (installed/available/missing)
- [x] Automatic addon scanning and import

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Framework | Tauri 2.x | Desktop app framework (~10 MB binary) |
| Backend | Rust | Memory safety, async I/O, cross-platform |
| Frontend | React 18 + TypeScript | Component ecosystem, type safety |
| Styling | Tailwind CSS 4 | Utility-first CSS |
| State | Zustand | Lightweight TypeScript-native state |
| HTTP | reqwest (Rust) | Async HTTP with connection pooling |
| Storage | SQLite (rusqlite) | Local database |
| Packaging | Tauri bundler | Native installers per platform |

---

## Repository Structure

```
eso-addon-manager/
├── .github/workflows/          # CI/CD workflows
│   ├── ci.yml                  # Build + test on PR
│   └── release.yml             # Build release binaries
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   │   ├── addons.rs       # Install/uninstall/scan
│   │   │   ├── github.rs       # Custom repo tracking
│   │   │   ├── index.rs        # Index fetch/cache
│   │   │   └── settings.rs     # User preferences
│   │   ├── models/             # Data structures
│   │   │   ├── addon.rs        # InstalledAddon, UpdateInfo
│   │   │   ├── index.rs        # AddonIndex, IndexAddon
│   │   │   └── settings.rs     # AppSettings
│   │   ├── services/           # Business logic
│   │   │   ├── database.rs     # SQLite operations
│   │   │   ├── downloader.rs   # HTTP downloads
│   │   │   ├── installer.rs    # Archive extraction
│   │   │   └── scanner.rs      # Local addon discovery
│   │   ├── utils/              # Helpers
│   │   │   ├── manifest.rs     # TOC file parsing
│   │   │   ├── paths.rs        # Platform paths
│   │   │   └── zip.rs          # Archive handling
│   │   ├── error.rs            # Custom error types
│   │   ├── state.rs            # AppState
│   │   └── lib.rs              # Main entry
│   └── migrations/
│       └── 001_initial.sql     # Database schema
├── src/                        # React frontend
│   ├── components/
│   │   ├── layout/             # Sidebar, Header
│   │   ├── addons/             # AddonCard, DependencyDialog
│   │   ├── github/             # AddRepoModal
│   │   ├── search/             # SearchBar
│   │   └── common/             # Button, UpdateBanner
│   ├── stores/                 # Zustand stores
│   │   ├── addonStore.ts
│   │   ├── indexStore.ts
│   │   ├── githubStore.ts
│   │   └── settingsStore.ts
│   ├── services/
│   │   └── tauri.ts            # Tauri command wrappers
│   ├── types/                  # TypeScript definitions
│   ├── App.tsx                 # Main app with views
│   └── index.css               # Tailwind imports
├── public/                     # Static assets
└── eso-addon-manager-implementation.md
```

---

## Quick Start

### Prerequisites

- Rust 1.75+ (via rustup)
- Node.js 20+
- Platform dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: WebView2 Runtime, Visual Studio Build Tools

### Development Commands

```bash
# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Rust checks
cargo check --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

---

## Platform-Specific ESO Paths

| Platform | ESO AddOns Directory |
|----------|---------------------|
| Windows | `C:\Users\{user}\Documents\Elder Scrolls Online\live\AddOns` |
| macOS | `~/Documents/Elder Scrolls Online/live/AddOns` |
| Linux (Steam) | `~/.steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns` |

---

## Development Priorities

Current focus order:
1. ~~**GitHub Integration** - Track addons from custom GitHub repos~~ ✅ DONE
2. **Submit to Index** - Allow users to submit tested GitHub addons to the index
3. Core install/uninstall from index
4. Local addon scanning
5. Update checking

---

## Tauri Commands (Backend API)

### Addon Commands
- `get_installed_addons` - List all installed addons
- `install_addon` - Download and install an addon
- `uninstall_addon` - Remove an addon
- `scan_local_addons` - Discover untracked addons
- `check_updates` - Find available updates
- `get_addon_directory` / `set_addon_directory` - ESO path management

### GitHub Commands
- `add_custom_repo` - Track a GitHub repo (with release type and branch)
- `get_custom_repos` - List tracked repos
- `remove_custom_repo` - Stop tracking a repo
- `get_github_repo_info` - Fetch repo metadata
- `get_github_repo_preview` - Get full preview (info + branches + releases)
- `list_github_branches` - List available branches for a repo
- `install_from_github` - Install addon from tracked repo
- `get_github_release` - Get latest release info

### Index Commands
- `fetch_index` - Get/refresh addon index
- `get_cached_index` - Get cached index
- `get_index_stats` - Index statistics

### Settings Commands
- `get_settings` / `update_settings` / `reset_settings`

---

## Coding Conventions

### Rust Backend
- Use `thiserror` for custom error types
- Async functions with `tokio`
- Commands return `Result<T, String>` for Tauri
- Use `#[cfg(target_os = "...")]` for platform code
- Drop mutex guards before await points

### React Frontend
- Functional components with TypeScript
- Zustand for state management
- Tailwind CSS for styling
- Use `@tauri-apps/api` for backend calls

### Git Workflow
- Commit after each meaningful change
- Conventional commit messages
- All commits include Claude Code attribution
- **Always run lint and tests before pushing**:
  ```bash
  # Rust checks (required before push)
  cargo fmt --manifest-path src-tauri/Cargo.toml
  cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
  cargo test --manifest-path src-tauri/Cargo.toml

  # Frontend checks
  npm run build
  npm run lint
  ```

### Version Bumps
When bumping the version, update all three files and restart the app:
```bash
# Files to update (all must match):
# - src-tauri/Cargo.toml (version = "X.Y.Z")
# - package.json ("version": "X.Y.Z")
# - src-tauri/tauri.conf.json ("version": "X.Y.Z")

# After version bump, restart the dev server to pick up changes:
lsof -ti:1420 | xargs kill 2>/dev/null; npm run tauri dev
```

---

## Addon Index

The addon index is hosted at:
- **GitHub**: https://github.com/brainsnorkel/eso-addon-index
- **JSON URL**: https://xop.co/eso-addon-index/index.json
- **Client Context**: https://github.com/brainsnorkel/eso-addon-index/blob/main/docs/addon-manager-client-context.md

The app fetches this index automatically and caches it locally.

**Important**: The client context document records current usage of the index API and should be consulted when implementing or modifying index-related features.

### Multi-Source Downloads (jsDelivr CDN)

The index provides multiple download sources per addon with automatic fallback:

```json
"download_sources": [
  {
    "type": "jsdelivr",
    "url": "https://cdn.jsdelivr.net/gh/Owner/Repo@tag/path/",
    "note": "CDN - no rate limits, CORS-friendly"
  },
  {
    "type": "github_archive",
    "url": "https://github.com/Owner/Repo/archive/refs/tags/tag.zip",
    "note": "Direct GitHub ZIP archive"
  }
]
```

**Benefits of jsDelivr CDN**:
- No rate limiting (GitHub has ~60 requests/hour unauthenticated)
- Global edge caching for faster downloads
- CORS-friendly for browser-based tools
- Works in restricted networks where GitHub may be blocked

**Implementation**:
- Backend prefers `github_archive` sources (ZIP format)
- Falls back through available sources on failure
- Legacy `download_url` field used as final fallback for backwards compatibility

### Slug Normalization

The index uses hyphen-separated slugs (e.g., `libaddonmenu-2-0`), but addon manifests may use dot notation (e.g., `LibAddonMenu-2.0`). The client normalizes slugs for matching:

```typescript
// Normalize: lowercase + replace dots with hyphens
const normalizeSlug = (s: string) => s.toLowerCase().replace(/\./g, '-');
// "LibAddonMenu-2.0" → "libaddonmenu-2-0"
```

This ensures dependencies declared in manifests correctly match index entries.

---

## Next Steps

1. ~~Implement GitHub repo validation and download~~ ✅ DONE
2. ~~Multi-source download with jsDelivr CDN~~ ✅ DONE
3. ~~Dependency status highlighting~~ ✅ DONE
4. Add file dialog for custom ESO path selection
5. Implement auto-update mechanism
6. Create application icons

---

## Dependency Resolution Implementation Plan

### Overview
When installing an addon, automatically resolve and install required dependencies from the index.

### Index Data Structure
Dependencies are stored as slug references in each addon's data:
```json
{
  "slug": "some-addon",
  "required_dependencies": ["libaddonmenu", "libchatmessage"],
  "optional_dependencies": ["libdebuglogger"]
}
```

### Resolution Algorithm

1. **Build Dependency Tree**
   - Parse `required_dependencies` array from the addon being installed
   - For each dependency slug, look up in the cached index
   - Recursively resolve nested dependencies
   - Detect circular dependencies

2. **Classify Dependencies**
   - **Resolved**: Found in index with valid download URL
   - **Already Installed**: Check against installed addons database
   - **Unresolved**: Slug not found in index (external dependency)

3. **User Confirmation Flow**
   - Display list of dependencies to be installed
   - Show warnings for unresolved dependencies
   - Suggest ESOUI/Minion for missing external dependencies
   - Allow user to proceed or cancel

4. **Installation Order**
   - Install dependencies before the main addon
   - Use topological sort for correct order
   - Handle partial failures gracefully

### Backend Changes (Rust)

**New service**: `src-tauri/src/services/resolver.rs`
```rust
pub struct DependencyResult {
    pub resolved: Vec<ResolvedDependency>,
    pub already_installed: Vec<String>,
    pub unresolved: Vec<String>,
}

pub fn resolve_dependencies(slug: &str, index: &AddonIndex, installed: &[InstalledAddon]) -> DependencyResult
```

**New command**: `resolve_addon_dependencies`
- Input: addon slug
- Output: `DependencyResult` for UI display

**Modified command**: `install_addon`
- Add optional `install_dependencies: bool` parameter
- If true, resolve and install all dependencies first

### Frontend Changes (React)

**New component**: `DependencyDialog`
- Shows resolved dependencies with checkboxes
- Warns about unresolved dependencies
- Confirm/Cancel buttons

**Store updates**: `addonStore.ts`
- Add `resolveDependencies(slug)` action
- Track dependency installation progress

### Error Handling
- Missing dependency in index → Show slug, suggest ESOUI
- Dependency download fails → Continue with warning, don't block main addon
- Circular dependency → Detect and break cycle

---

## References

- [Implementation Guide](eso-addon-manager-implementation.md) - Full technical specification
- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [ESO API Documentation](https://wiki.esoui.com/API)
- [ESOUI (addon source)](https://www.esoui.com/)
