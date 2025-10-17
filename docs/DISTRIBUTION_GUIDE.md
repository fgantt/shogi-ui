# Distribution Guide for Shogi Vibe

This guide covers how to package and distribute Shogi Vibe to end users.

## Table of Contents
- [Quick Start](#quick-start)
- [Platform-Specific Packages](#platform-specific-packages)
- [Code Signing](#code-signing)
- [Distribution Channels](#distribution-channels)
- [Auto-Updates](#auto-updates)
- [CI/CD Automation](#cicd-automation)
- [Troubleshooting](#troubleshooting)

## Quick Start

### Building Release Packages

To create distributable installers for your current platform:

```bash
npm run tauri:build
```

This will:
1. Build the Rust engine in release mode
2. Build the frontend with Vite
3. Create platform-specific installers in `src-tauri/target/release/bundle/`

### Build Time

- **First build**: 5-15 minutes (compiles all dependencies)
- **Subsequent builds**: 1-3 minutes (incremental compilation)

### Output Location

All installers are created in: `src-tauri/target/release/bundle/`

## Platform-Specific Packages

### macOS

**Generated Files:**
- `dmg/Shogi Vibe_0.1.0_universal.dmg` - Universal installer (Intel + Apple Silicon)
- `macos/Shogi Vibe.app` - Application bundle

**Distribution:**
1. Share the `.dmg` file directly
2. Users drag the app to Applications folder
3. First launch requires right-click → Open (Gatekeeper)

**File Size:** ~15-30 MB (depending on optimization)

**Requirements:**
- macOS 10.13 or later
- No additional dependencies needed

### Windows

**Generated Files:**
- `msi/Shogi Vibe_0.1.0_x64_en-US.msi` - Windows Installer
- `nsis/Shogi Vibe_0.1.0_x64-setup.exe` - NSIS installer

**Distribution:**
1. Share either `.msi` or `.exe` installer
2. Users run the installer
3. May show SmartScreen warning (unsigned apps)

**File Size:** ~10-25 MB

**Requirements:**
- Windows 7 or later
- WebView2 runtime (auto-installed if missing)

### Linux

**Generated Files:**
- `deb/shogi-vibe_0.1.0_amd64.deb` - Debian/Ubuntu package
- `appimage/shogi-vibe_0.1.0_amd64.AppImage` - Universal Linux executable
- `rpm/shogi-vibe-0.1.0-1.x86_64.rpm` - Red Hat/Fedora package

**Distribution:**

**For .deb (Debian/Ubuntu):**
```bash
sudo dpkg -i shogi-vibe_0.1.0_amd64.deb
```

**For AppImage (any Linux):**
```bash
chmod +x shogi-vibe_0.1.0_amd64.AppImage
./shogi-vibe_0.1.0_amd64.AppImage
```

**For .rpm (Fedora/RHEL):**
```bash
sudo rpm -i shogi-vibe-0.1.0-1.x86_64.rpm
```

**File Size:** ~15-35 MB

## Cross-Platform Building

### Building for Multiple Platforms

To build for a specific platform:

```bash
npm run tauri build -- --target x86_64-pc-windows-msvc    # Windows
npm run tauri build -- --target x86_64-apple-darwin        # macOS Intel
npm run tauri build -- --target aarch64-apple-darwin       # macOS Apple Silicon
npm run tauri build -- --target x86_64-unknown-linux-gnu   # Linux
```

**Note:** You can only build for your current OS platform, unless using:
- **macOS → Windows/Linux:** Use Docker or CI/CD
- **Windows → macOS/Linux:** Use Docker or CI/CD
- **Linux → Windows/macOS:** Use Docker or CI/CD

### Universal macOS Build

Tauri automatically creates universal binaries (Intel + Apple Silicon) on macOS.

## Code Signing

### Why Code Sign?

- **macOS:** Prevents Gatekeeper warnings
- **Windows:** Prevents SmartScreen warnings
- **Trust:** Users know the app is from you

### macOS Code Signing

**Requirements:**
- Apple Developer Account ($99/year)
- Developer ID Application certificate

**Setup:**

1. Get certificate from Apple Developer Portal
2. Import to Keychain
3. Update `tauri.conf.json`:

```json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "entitlements": null,
      "exceptionDomain": "",
      "providerShortName": null
    }
  }
}
```

**Notarization** (required for macOS 10.15+):

```bash
# After building
xcrun notarytool submit "src-tauri/target/release/bundle/dmg/Shogi Vibe_0.1.0_universal.dmg" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Staple the notarization
xcrun stapler staple "src-tauri/target/release/bundle/dmg/Shogi Vibe_0.1.0_universal.dmg"
```

### Windows Code Signing

**Requirements:**
- Code signing certificate from trusted CA (Digicert, Sectigo, etc.)
- Certificate costs $100-$400/year

**Setup:**

Update `tauri.conf.json`:

```json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

Or sign after building:

```powershell
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /a "Shogi Vibe_0.1.0_x64-setup.exe"
```

## Distribution Channels

### Direct Download (Easiest)

**Setup:**
1. Host installers on your website or GitHub Releases
2. Create download page with platform detection
3. Users download and install

**Example GitHub Release:**
```bash
# Tag a release
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# Upload installers to GitHub Releases
# (can be automated with GitHub Actions)
```

### GitHub Releases (Recommended)

- Free hosting
- Automatic version tracking
- Download statistics
- Perfect for open source

### App Stores

#### Mac App Store
- **Cost:** $99/year Apple Developer account
- **Pros:** Built-in updates, trusted source
- **Cons:** Review process, sandboxing requirements

#### Microsoft Store
- **Cost:** $19 one-time fee
- **Pros:** Built-in updates, trusted source
- **Cons:** Review process, certification

#### Snap Store (Linux)
- **Cost:** Free
- **Pros:** Auto-updates, sandboxed
- **Cons:** Snap controversy in Linux community

#### Flathub (Linux)
- **Cost:** Free
- **Pros:** Popular in Linux community
- **Cons:** Requires Flatpak packaging

## Auto-Updates

Tauri supports automatic updates using the Tauri Updater plugin.

### Setup Auto-Updates

1. **Install the updater plugin:**

```bash
cd src-tauri
cargo add tauri-plugin-updater
```

2. **Update `tauri.conf.json`:**

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/yourusername/shogi-game/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

3. **Generate signing keys:**

```bash
npm run tauri signer generate -- -w ~/.tauri/myapp.key
```

4. **Create update manifest** (`latest.json`):

```json
{
  "version": "0.2.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2025-10-17T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/.../Shogi-Vibe_0.2.0_aarch64.app.tar.gz"
    },
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../Shogi-Vibe_0.2.0_x64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/.../Shogi-Vibe_0.2.0_x64-setup.nsis.zip"
    }
  }
}
```

## CI/CD Automation

### GitHub Actions (Recommended)

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    strategy:
      matrix:
        platform:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: macos-latest
            target: universal-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc

    runs-on: ${{ matrix.platform.os }}

    steps:
      - uses: actions/checkout@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20

      - name: Setup Rust
        uses: dtolnay/rust-toolchain@stable

      - name: Install dependencies (Ubuntu only)
        if: matrix.platform.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf

      - name: Install frontend dependencies
        run: npm ci

      - name: Build
        run: npm run tauri:build

      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform.os }}-build
          path: src-tauri/target/release/bundle/

      - name: Create Release
        uses: softprops/action-gh-release@v1
        if: startsWith(github.ref, 'refs/tags/')
        with:
          files: src-tauri/target/release/bundle/**/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## Optimization Tips

### Reduce Binary Size

1. **Enable LTO and strip debug symbols:**

Already configured in `Cargo.toml`:
```toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true  # Add this line
```

2. **Use UPX compression (optional):**

```bash
upx --best --lzma src-tauri/target/release/shogi-vibe
```

**Warning:** May trigger antivirus false positives

### Improve Build Times

1. **Use sccache for compilation caching:**

```bash
cargo install sccache
export RUSTC_WRAPPER=sccache
```

2. **Parallel builds:**

```bash
npm run tauri build -- --bundles dmg -j $(nproc)
```

## Testing Before Release

### Test Checklist

- [ ] Build succeeds on all target platforms
- [ ] App launches without errors
- [ ] All features work correctly
- [ ] No console errors or warnings
- [ ] Installer works correctly
- [ ] Uninstaller works correctly
- [ ] App icon displays properly
- [ ] About dialog shows correct version
- [ ] Updates work (if implemented)

### Beta Testing

Consider releasing to a small group first:

1. Create pre-release on GitHub
2. Share with beta testers
3. Gather feedback
4. Fix issues
5. Create official release

## Troubleshooting

### "App is damaged" on macOS

**Cause:** Gatekeeper blocking unsigned apps

**Solution:**
1. Code sign the app, OR
2. Users can run: `xattr -cr /Applications/Shogi\ Vibe.app`

### Windows SmartScreen Warning

**Cause:** App not code signed

**Solution:**
1. Code sign with trusted certificate, OR
2. Users click "More info" → "Run anyway"

### Linux Dependencies Missing

**Cause:** Missing system libraries

**Solution:** Install webkit2gtk:
```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-dev

# Fedora
sudo dnf install webkit2gtk4.0-devel

# Arch
sudo pacman -S webkit2gtk
```

### Build Fails - Out of Memory

**Solution:**
1. Reduce parallel jobs: `npm run tauri build -- -j 1`
2. Disable LTO temporarily in Cargo.toml
3. Use swap space on Linux

### Large Bundle Size

**Check:**
1. Are you including unnecessary assets?
2. Is LTO enabled? (`lto = true`)
3. Are debug symbols stripped? (`strip = true`)
4. Consider compressing assets

## Distribution Checklist

Before releasing:

- [ ] Update version in `tauri.conf.json`
- [ ] Update version in `Cargo.toml`
- [ ] Update version in `package.json`
- [ ] Update CHANGELOG
- [ ] Test on all platforms
- [ ] Code sign (if applicable)
- [ ] Create release notes
- [ ] Tag git release
- [ ] Build installers
- [ ] Upload to distribution channel
- [ ] Announce release

## Getting Help

- **Tauri Docs:** https://tauri.app/
- **Tauri Discord:** https://discord.gg/tauri
- **GitHub Issues:** Report bugs and feature requests

## Example Distribution Flow

### For Free/Open Source:

1. Build with `npm run tauri:build`
2. Create GitHub Release with tag (e.g., `v0.1.0`)
3. Upload installers to GitHub Release
4. Share release link

### For Commercial:

1. Build with `npm run tauri:build`
2. Code sign all installers
3. Notarize macOS app
4. Upload to your website
5. Consider app store distribution
6. Set up auto-updates
7. Market to users!

---

**Next Steps:**
- Try building: `npm run tauri:build`
- Check output in `src-tauri/target/release/bundle/`
- Test the installer on your machine
- Share with friends for testing!

