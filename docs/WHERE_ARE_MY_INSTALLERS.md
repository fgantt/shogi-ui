# Where Are My Installers? 📦

After running `npm run tauri:build`, here's exactly where to find your installers:

## Location

```
src-tauri/target/release/bundle/
```

## What You'll Find (by Platform)

### 🍎 macOS

```
src-tauri/target/release/bundle/
│
├── dmg/
│   └── Shogi Vibe_0.1.0_universal.dmg    👈 SHARE THIS FILE
│       (20-30 MB - Works on Intel & Apple Silicon)
│
└── macos/
    └── Shogi Vibe.app                     (App bundle - already in DMG)
```

**What to distribute:** The `.dmg` file  
**File size:** ~20-30 MB  
**Works on:** macOS 10.13+ (Intel and Apple Silicon)

---

### 🪟 Windows

```
src-tauri/target/release/bundle/
│
├── nsis/
│   └── Shogi Vibe_0.1.0_x64-setup.exe    👈 SHARE THIS (Recommended)
│       (15-25 MB - Modern installer)
│
└── msi/
    └── Shogi Vibe_0.1.0_x64_en-US.msi    👈 OR THIS
        (15-25 MB - Traditional installer)
```

**What to distribute:** Either `.exe` or `.msi` (or both!)  
**File size:** ~15-25 MB each  
**Works on:** Windows 7+

**Which one?**
- `.exe` (NSIS) - More modern, better UX
- `.msi` - Traditional Windows Installer, familiar to IT admins

---

### 🐧 Linux

```
src-tauri/target/release/bundle/
│
├── appimage/
│   └── shogi-vibe_0.1.0_amd64.AppImage   👈 SHARE THIS (Best compatibility)
│       (20-35 MB - Run anywhere, no install needed)
│
├── deb/
│   └── shogi-vibe_0.1.0_amd64.deb        👈 For Debian/Ubuntu users
│       (20-30 MB)
│
└── rpm/
    └── shogi-vibe-0.1.0-1.x86_64.rpm     👈 For Fedora/RHEL users
        (20-30 MB)
```

**What to distribute:**
- **AppImage** - Universal, works on ANY Linux distro
- **Deb** - For Debian, Ubuntu, Mint, Pop!_OS, etc.
- **RPM** - For Fedora, RHEL, CentOS, openSUSE, etc.

**Pro tip:** Share the AppImage for maximum compatibility.

---

## Quick Commands

### Navigate to installers:
```bash
cd src-tauri/target/release/bundle/
```

### List all installers:
```bash
find src-tauri/target/release/bundle/ -type f \( -name "*.dmg" -o -name "*.exe" -o -name "*.msi" -o -name "*.deb" -o -name "*.rpm" -o -name "*.AppImage" \)
```

### Copy to desktop (macOS example):
```bash
cp src-tauri/target/release/bundle/dmg/*.dmg ~/Desktop/
```

---

## File Naming Convention

Tauri uses this format:
```
{Product Name}_{Version}_{Architecture}.{Extension}

Examples:
- Shogi Vibe_0.1.0_universal.dmg        (macOS)
- Shogi Vibe_0.1.0_x64-setup.exe        (Windows)
- shogi-vibe_0.1.0_amd64.AppImage       (Linux)
```

---

## What About Other Files?

You'll also see:

```
src-tauri/target/release/
│
├── bundle/           👈 Your installers are here
├── shogi-engine      (The engine binary - already in installers)
├── shogi-vibe        (The app executable - already in installers)
└── ...               (Build artifacts - you don't need these)
```

**Only share files from the `bundle/` directory!**

---

## Installer Sizes Explained

Why are they this size?

Your installers include:
- ✅ Your React UI (~2-5 MB)
- ✅ Rust engine binary (~8-15 MB)
- ✅ Tauri runtime (~3-5 MB)
- ✅ WebView2 (Windows only, downloaded separately)
- ✅ All your assets (images, sounds, themes)

**This is impressive!** Compare to Electron apps (often 150+ MB).

---

## Testing Your Installers

### macOS
```bash
# Open the DMG
open src-tauri/target/release/bundle/dmg/*.dmg
```

### Windows
```powershell
# Run the installer
.\src-tauri\target\release\bundle\nsis\Shogi Vibe_0.1.0_x64-setup.exe
```

### Linux (AppImage)
```bash
# Make executable and run
chmod +x src-tauri/target/release/bundle/appimage/*.AppImage
./src-tauri/target/release/bundle/appimage/*.AppImage
```

---

## Troubleshooting

### "I don't see the bundle/ directory"

**Cause:** Build hasn't run or failed

**Solution:**
```bash
npm run tauri:build
```

Wait for "Finished" message.

---

### "Bundle/ exists but is empty"

**Cause:** Build failed partway through

**Solution:**
```bash
cargo clean
npm run tauri:build
```

---

### "I only see some platforms"

**Normal!** You can only build for your current OS:
- On macOS → Get macOS installers
- On Windows → Get Windows installers
- On Linux → Get Linux installers

To build for all platforms, use GitHub Actions or CI/CD.

---

## Quick Upload to GitHub

```bash
# 1. Go to your GitHub repository
# 2. Click "Releases" → "Create a new release"
# 3. Choose a tag (e.g., v1.0.0)
# 4. Drag and drop your installers from bundle/
# 5. Add release notes
# 6. Click "Publish release"

# Or use GitHub CLI:
gh release create v1.0.0 \
  src-tauri/target/release/bundle/dmg/*.dmg \
  --title "Shogi Vibe v1.0.0" \
  --notes "First release!"
```

---

## Summary

1. **Build:** `npm run tauri:build`
2. **Find:** `src-tauri/target/release/bundle/`
3. **Share:** Upload the installer(s) to GitHub or your website
4. **Done!** Users download and install

**That's it!** 🎉

---

## Next Steps

- [📦 Quick Start Guide](PACKAGING_QUICK_START.md) - Step-by-step instructions
- [📚 Full Distribution Guide](DISTRIBUTION_GUIDE.md) - Complete documentation
- [✅ Release Checklist](../RELEASE_CHECKLIST.md) - Don't forget anything

---

**Still confused?** Just run:
```bash
npm run tauri:build && ls -lh src-tauri/target/release/bundle/
```

You'll see your installers! Share them with the world! 🚀

