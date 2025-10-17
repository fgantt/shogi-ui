# Packaging Quick Start Guide

**Get your Shogi Vibe app into users' hands in 3 simple steps!**

## TL;DR - Fastest Way to Distribute

```bash
# 1. Build installers
npm run tauri:build

# 2. Find installers in:
# src-tauri/target/release/bundle/

# 3. Share them!
# Upload to GitHub Releases, your website, or send directly to users
```

That's it! 🎉

---

## Step-by-Step First Release

### 1. **Prepare Your App** (5 minutes)

Update version numbers in all files:

```bash
# Use the helper script
npm run version:update 1.0.0

# Or update manually:
# - src-tauri/tauri.conf.json
# - src-tauri/Cargo.toml
# - package.json
# - Cargo.toml
```

### 2. **Build Installers** (5-15 minutes for first build)

```bash
npm run tauri:build
```

☕ Grab a coffee. First build compiles all dependencies and takes 5-15 minutes.  
Subsequent builds are much faster (1-3 minutes).

### 3. **Find Your Installers**

Navigate to: `src-tauri/target/release/bundle/`

You'll find installers for your platform:

**On macOS:**
```
bundle/
├── dmg/
│   └── Shogi Vibe_0.1.0_universal.dmg  ← Share this!
└── macos/
    └── Shogi Vibe.app
```

**On Windows:**
```
bundle/
├── nsis/
│   └── Shogi Vibe_0.1.0_x64-setup.exe  ← Share this!
└── msi/
    └── Shogi Vibe_0.1.0_x64_en-US.msi  ← Or this!
```

**On Linux:**
```
bundle/
├── appimage/
│   └── shogi-vibe_0.1.0_amd64.AppImage  ← Share this (universal)
├── deb/
│   └── shogi-vibe_0.1.0_amd64.deb       ← For Debian/Ubuntu
└── rpm/
    └── shogi-vibe-0.1.0-1.x86_64.rpm    ← For Fedora/RHEL
```

### 4. **Test the Installer**

Before sharing, test it yourself:

1. Run the installer
2. Launch the app
3. Play a quick game
4. Make sure everything works

### 5. **Distribute to Users**

Choose your distribution method:

#### Option A: GitHub Releases (Recommended)

```bash
# Create a git tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# Then on GitHub:
# 1. Go to Releases → Create new release
# 2. Select tag v1.0.0
# 3. Upload your installers
# 4. Write release notes
# 5. Publish!
```

#### Option B: Direct Distribution

Just send the installer file to your users! They can:
- **macOS**: Open the `.dmg`, drag to Applications
- **Windows**: Run the `.exe` installer
- **Linux**: Install with their package manager or run `.AppImage`

---

## What About Other Platforms?

### "I'm on macOS, how do I build for Windows?"

You have three options:

1. **Use GitHub Actions** (recommended)
   - Set up the included workflow
   - Push a tag
   - GitHub builds for all platforms automatically
   - See: `.github/workflows/release.yml.example`

2. **Use a Windows/Linux machine**
   - Build on each platform natively

3. **Use CI/CD services**
   - Travis CI, CircleCI, etc.

**Note:** You can only build natively for your current OS without virtualization/CI.

---

## Common Questions

### "Do I need to sign the app?"

**For testing/personal use:** No, it works fine unsigned.

**For wide distribution:**
- **macOS**: Yes, users get scary warnings without signing ($99/year)
- **Windows**: Recommended, reduces SmartScreen warnings ($100-400/year)
- **Linux**: Not needed

See [Distribution Guide](DISTRIBUTION_GUIDE.md) for signing instructions.

### "How big are the installers?"

Typical sizes:
- **macOS**: 20-30 MB (universal binary)
- **Windows**: 15-25 MB
- **Linux**: 20-35 MB

These include:
- Your app
- Rust engine
- All UI assets
- System libraries

### "What if users can't install?"

**macOS users seeing "App is damaged":**
```bash
# Tell them to run:
xattr -cr "/Applications/Shogi Vibe.app"
```

**Windows SmartScreen warning:**
- Click "More info" → "Run anyway"
- Or code sign your app

**Linux missing dependencies:**
```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.0-dev

# Or use the AppImage (no dependencies needed)
```

### "How do I update the app later?"

**Simple way:**
1. Update version numbers
2. Build new installers
3. Upload to GitHub Releases (or your site)
4. Users download and reinstall

**Advanced way:**
- Set up Tauri auto-updates
- Users get automatic update notifications
- See [Distribution Guide](DISTRIBUTION_GUIDE.md) for setup

---

## Size Optimization

Want smaller installers? Try these:

```toml
# In Cargo.toml [profile.release]
strip = true        # Remove debug symbols
lto = true          # Link-time optimization (already enabled)
opt-level = "z"     # Optimize for size instead of "3" for speed
```

Then rebuild:
```bash
npm run tauri:build
```

**Warning:** Size optimization may slightly reduce performance.

---

## Build Once, Distribute Everywhere

### macOS Universal Binary

Tauri automatically creates **universal binaries** on macOS, which work on:
- Intel Macs (x86_64)
- Apple Silicon Macs (M1/M2/M3 - aarch64)

One `.dmg` for all Macs! 🎉

### Windows Installers

You get two installers:
- **`.msi`** - Traditional Windows Installer
- **`.exe`** - Modern NSIS installer

Both work fine. The `.exe` is usually preferred by users.

### Linux Options

- **`.AppImage`** - Works on ANY Linux (no installation needed)
- **`.deb`** - For Debian, Ubuntu, Mint, Pop!_OS
- **`.rpm`** - For Fedora, RHEL, CentOS, openSUSE

**Pro tip:** Share the AppImage for maximum compatibility.

---

## Checklist for First Release

- [ ] Test your app thoroughly
- [ ] Update version numbers (`npm run version:update 1.0.0`)
- [ ] Update CHANGELOG.md
- [ ] Run `npm run tauri:build`
- [ ] Test the installer on a clean system (if possible)
- [ ] Create GitHub Release (or upload to your site)
- [ ] Write release notes
- [ ] Share with users!
- [ ] Celebrate! 🎉

---

## Next Steps

- **First time?** Follow this quick start guide above
- **Want automation?** See [.github/workflows/release.yml.example](.github/workflows/release.yml.example)
- **Need details?** Read the full [Distribution Guide](DISTRIBUTION_GUIDE.md)
- **Ready to release?** Use the [Release Checklist](../RELEASE_CHECKLIST.md)

---

## Getting Help

- **Tauri Documentation:** https://tauri.app/
- **Tauri Discord:** https://discord.gg/tauri
- **GitHub Issues:** Report bugs on your repository

---

## Quick Reference

```bash
# Development
npm run tauri:dev              # Run in dev mode

# Building
npm run tauri:build            # Build release installers

# Version management
npm run version:update 1.0.0   # Update all version numbers

# Finding installers
ls -la src-tauri/target/release/bundle/

# Releasing
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

---

**Happy distributing!** 🚀

Your users will love the native performance and easy installation of your Tauri app.

