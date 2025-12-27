# ESO Addon Manager

A cross-platform desktop application for discovering, installing, and managing Elder Scrolls Online (ESO) addons. Built with Tauri 2.x (Rust backend + React frontend) for native performance with a modern UI.

**Status**: In Development
**License**: MIT

## Features

- **Browse & Install** - Discover addons from the community index
- **GitHub Integration** - Track and install addons directly from GitHub repositories
- **Update Management** - Check for and apply addon updates
- **Local Scanning** - Detect manually installed addons
- **Cross-Platform** - Native builds for Windows, macOS, and Linux (~10 MB)

## Technology Stack

| Layer | Technology |
|-------|------------|
| Framework | Tauri 2.x |
| Backend | Rust |
| Frontend | React 18 + TypeScript |
| Styling | Tailwind CSS 4 |
| State | Zustand |
| Storage | SQLite |

## Installation

### Download

Pre-built binaries will be available on the [Releases](https://github.com/brainsnorkel/eso-addon-manager/releases) page.

### Build from Source

#### Prerequisites

- Rust 1.75+ (via [rustup](https://rustup.rs/))
- Node.js 20+
- Platform dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: WebView2 Runtime, Visual Studio Build Tools

#### Build Commands

```bash
# Clone the repository
git clone https://github.com/brainsnorkel/eso-addon-manager.git
cd eso-addon-manager

# Install frontend dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## ESO AddOns Directory

The app auto-detects your ESO installation, or you can configure it manually:

| Platform | Default Path |
|----------|-------------|
| Windows | `C:\Users\{user}\Documents\Elder Scrolls Online\live\AddOns` |
| macOS | `~/Documents/Elder Scrolls Online/live/AddOns` |
| Linux (Steam) | `~/.steam/steam/steamapps/compatdata/306130/pfx/drive_c/users/steamuser/Documents/Elder Scrolls Online/live/AddOns` |

## Addon Index

This app uses the [ESO Addon Index](https://github.com/brainsnorkel/eso-addon-index) - a curated list of ESO addons with metadata for easy discovery and installation.

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [ESOUI](https://www.esoui.com/) - Primary addon source
- [Tauri](https://tauri.app/) - Desktop app framework
- ESO addon community
