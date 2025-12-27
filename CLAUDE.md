# ESO Addon Manager

## Project Overview

A cross-platform desktop application for discovering, installing, and managing Elder Scrolls Online (ESO) addons. Built with Tauri 2.x (Rust backend + React frontend) for native performance with a modern UI.

**Status**: SCAFFOLDED - Ready for Development
**License**: MIT
**Target**: Public release for ESO community

---

## Scaffolding Complete

The project structure has been fully scaffolded with:

- [x] Tauri 2.x project with React + TypeScript
- [x] Tailwind CSS 4 configuration
- [x] Rust backend modules (commands, models, services, utils)
- [x] SQLite database schema and migrations
- [x] React frontend with Zustand state management
- [x] TypeScript type definitions
- [x] Basic UI components (Sidebar, Header, AddonCard, etc.)
- [x] GitHub Actions CI/CD workflows

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
│   │   ├── addons/             # AddonCard
│   │   ├── search/             # SearchBar
│   │   └── common/             # Button, etc.
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
1. **GitHub Integration** - Track addons from custom GitHub repos (PRIORITY)
2. Core install/uninstall from index
3. Local addon scanning
4. Update checking

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
- `add_custom_repo` - Track a GitHub repo
- `get_custom_repos` - List tracked repos
- `remove_custom_repo` - Stop tracking a repo
- `get_github_repo_info` - Fetch repo metadata

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

---

## Addon Index

The addon index is hosted at:
- **GitHub**: https://github.com/brainsnorkel/eso-addon-index
- **JSON URL**: https://xop.co/eso-addon-index/
- **Client Context**: https://github.com/brainsnorkel/eso-addon-index/blob/main/docs/addon-manager-client-context.md

The app fetches this index automatically and caches it locally.

**Important**: The client context document records current usage of the index API and should be consulted when implementing or modifying index-related features.

---

## Next Steps

1. Implement GitHub repo validation and download
2. Add file dialog for custom ESO path selection
3. Implement auto-update mechanism
4. Add dependency resolution for addons
5. Create application icons

---

## References

- [Implementation Guide](eso-addon-manager-implementation.md) - Full technical specification
- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [ESO API Documentation](https://wiki.esoui.com/API)
- [ESOUI (addon source)](https://www.esoui.com/)
