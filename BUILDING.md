# Quick Build Reference

## For End Users

### Download Pre-Built Installers
Visit [Releases](https://github.com/yourusername/shogi-game/releases) to download installers for:
- **macOS**: `.dmg` file
- **Windows**: `.exe` or `.msi` installer
- **Linux**: `.deb`, `.AppImage`, or `.rpm`

## For Developers

### Prerequisites
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Node.js (20+)
# Download from https://nodejs.org/ or use nvm

# Install dependencies
npm install
```

### Development
```bash
# Run in development mode (with hot reload)
npm run tauri:dev
```

### Production Build
```bash
# Build release installers for your platform
npm run tauri:build

# Output location:
# src-tauri/target/release/bundle/
```

### Platform-Specific Builds
```bash
# macOS
npm run tauri:build

# Windows  
npm run tauri:build

# Linux
npm run tauri:build
```

**Note:** You can only build for your current operating system unless using Docker or CI/CD.

### Finding Your Built Files

After running `npm run tauri:build`, look in `src-tauri/target/release/bundle/`:

```
src-tauri/target/release/bundle/
├── dmg/                 (macOS)
│   └── Shogi Vibe_0.1.0_universal.dmg
├── macos/              (macOS)
│   └── Shogi Vibe.app
├── msi/                (Windows)
│   └── Shogi Vibe_0.1.0_x64_en-US.msi
├── nsis/               (Windows)
│   └── Shogi Vibe_0.1.0_x64-setup.exe
├── deb/                (Linux)
│   └── shogi-vibe_0.1.0_amd64.deb
├── appimage/           (Linux)
│   └── shogi-vibe_0.1.0_amd64.AppImage
└── rpm/                (Linux)
    └── shogi-vibe-0.1.0-1.x86_64.rpm
```

### Troubleshooting

**Build fails?**
- Make sure all prerequisites are installed
- Run `cargo clean` and try again
- Check [DISTRIBUTION_GUIDE.md](docs/DISTRIBUTION_GUIDE.md) for detailed help

**Build too slow?**
- First build takes 5-15 minutes (compiles all dependencies)
- Subsequent builds: 1-3 minutes (incremental)
- Use `sccache` for faster rebuilds

### More Information
See [docs/DISTRIBUTION_GUIDE.md](docs/DISTRIBUTION_GUIDE.md) for:
- Code signing
- App store distribution  
- Auto-updates
- CI/CD setup
- Optimization tips

