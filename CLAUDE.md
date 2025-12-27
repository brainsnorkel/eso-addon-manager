# ESO Addon Manager

## Project Overview

A cross-platform desktop application for discovering, installing, and managing Elder Scrolls Online (ESO) addons. Built with Tauri 2.x (Rust backend + React frontend) for native performance with a modern UI.

**Status**: IN DEVELOPMENT
**License**: MIT
**Target**: Public release for ESO community

---

## Technology Stack

| Layer | Technology | Purpose |
|-------|------------|---------|
| Framework | Tauri 2.x | Desktop app framework (~10 MB binary) |
| Backend | Rust | Memory safety, async I/O, cross-platform |
| Frontend | React 18 + TypeScript | Component ecosystem, type safety |
| Styling | Tailwind CSS | Utility-first CSS |
| State | Zustand | Lightweight TypeScript-native state |
| HTTP | reqwest (Rust) | Async HTTP with connection pooling |
| Storage | SQLite (rusqlite) | Local database |
| Packaging | Tauri bundler | Native installers per platform |

---

## Repository Structure

```
eso-addon-manager/
├── .github/workflows/          # CI/CD workflows
├── src-tauri/                  # Rust backend
│   ├── src/
│   │   ├── commands/           # Tauri command handlers
│   │   ├── models/             # Data structures
│   │   ├── services/           # Business logic
│   │   └── utils/              # Helpers (paths, manifest, zip)
│   └── migrations/             # SQLite migrations
├── src/                        # React frontend
│   ├── components/             # UI components
│   ├── hooks/                  # Custom React hooks
│   ├── stores/                 # Zustand stores
│   ├── services/               # Tauri command wrappers
│   └── types/                  # TypeScript types
├── public/                     # Static assets
└── eso-addon-manager-implementation.md  # Full implementation guide
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

# Run tests
npm test                    # Frontend tests
cargo test                  # Rust tests (from src-tauri/)

# Linting
npm run lint               # ESLint
cargo clippy              # Rust linting (from src-tauri/)
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

## Key Implementation Details

### TOC Manifest Parsing

ESO addon manifests are `.txt` files with metadata:
```
## Title: AddonName
## APIVersion: 101041
## Author: AuthorName
## Version: 1.0.0
## DependsOn: LibAddonMenu-2.0
```

### Database Schema

Key tables:
- `installed_addons` - Tracks installed addons with version, source
- `custom_repos` - Custom GitHub repos not in main index
- `index_cache` - Cached addon index with ETag for conditional requests
- `settings` - User preferences
- `downloads` - Download queue and history

### Tauri Commands

Commands exposed to frontend:
- `get_installed_addons` / `install_addon` / `uninstall_addon`
- `add_custom_repo` / `get_custom_repos` / `remove_custom_repo`
- `fetch_index` / `check_updates`
- `scan_local_addons`

---

## Coding Conventions

### Rust Backend
- Use `thiserror` for custom error types
- Async functions with `tokio`
- Commands return `Result<T, String>` for Tauri compatibility
- Use `#[cfg(target_os = "...")]` for platform-specific code

### React Frontend
- Functional components with TypeScript
- Zustand for state management (no Redux)
- Tailwind CSS for styling (no CSS modules)
- Use `@tauri-apps/api` for backend communication

### Git Workflow
- Commit after each meaningful code change
- Use conventional commit messages
- All commits include Claude Code attribution

---

## Testing

### Rust Tests
```bash
cd src-tauri
cargo test
```

### Frontend Tests
```bash
npm test          # Run vitest
npm run test:ui   # With UI
```

---

## Known Challenges

1. **Linux Path Detection**: Steam/Lutris Wine prefix paths vary; may need user prompt
2. **Addon Dependencies**: Some addons require libraries (LibAddonMenu, etc.)
3. **Perfected vs Non-Perfected**: Gear sets have "Perfected" variants

---

## References

- [Implementation Guide](eso-addon-manager-implementation.md) - Full technical specification
- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [ESO API Documentation](https://wiki.esoui.com/API)
- [ESOUI (addon source)](https://www.esoui.com/)
