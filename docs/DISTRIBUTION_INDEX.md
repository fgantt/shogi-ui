# Distribution Documentation Index

**Everything you need to package and distribute Shogi Vibe.**

## 🚀 Start Here

### Never Packaged Before?
👉 **[Packaging Quick Start](PACKAGING_QUICK_START.md)** - Get your first release out in 15 minutes

### Just Want to Build?
👉 **[Where Are My Installers?](WHERE_ARE_MY_INSTALLERS.md)** - Find your built files after running the build

### Building From Source?
👉 **[BUILDING.md](../BUILDING.md)** - Quick reference for developers

---

## 📚 Complete Documentation

### Core Guides

| Document | What It Covers | When to Use |
|----------|---------------|-------------|
| [**Packaging Quick Start**](PACKAGING_QUICK_START.md) | Fast track to your first release | First time packaging |
| [**Distribution Guide**](DISTRIBUTION_GUIDE.md) | Complete packaging documentation | Need detailed instructions |
| [**Where Are My Installers?**](WHERE_ARE_MY_INSTALLERS.md) | Finding your built files | Can't find installers |
| [**Building Guide**](../BUILDING.md) | Developer build reference | Setting up dev environment |

### Process Guides

| Document | What It Covers | When to Use |
|----------|---------------|-------------|
| [**Release Checklist**](../RELEASE_CHECKLIST.md) | Step-by-step release process | Creating a new release |
| [**Changelog Template**](../CHANGELOG.md) | Version history tracking | Documenting changes |

### Automation

| File | What It Does | When to Use |
|------|-------------|-------------|
| `scripts/update-version.sh` | Updates versions across all files | Before each release |
| `.github/workflows/release.yml.example` | Automates multi-platform builds | Want CI/CD automation |

---

## 🎯 Quick Navigation by Goal

### "I want to create my first release"
1. [Packaging Quick Start](PACKAGING_QUICK_START.md)
2. [Release Checklist](../RELEASE_CHECKLIST.md)
3. [Where Are My Installers?](WHERE_ARE_MY_INSTALLERS.md)

### "I need to understand the full process"
1. [Distribution Guide](DISTRIBUTION_GUIDE.md) - Read cover to cover

### "I want to automate releases"
1. [Distribution Guide - CI/CD Section](DISTRIBUTION_GUIDE.md#cicd-automation)
2. `.github/workflows/release.yml.example`

### "I want to code sign my app"
1. [Distribution Guide - Code Signing Section](DISTRIBUTION_GUIDE.md#code-signing)

### "I want to set up auto-updates"
1. [Distribution Guide - Auto-Updates Section](DISTRIBUTION_GUIDE.md#auto-updates)

### "I'm having build problems"
1. [Distribution Guide - Troubleshooting](DISTRIBUTION_GUIDE.md#troubleshooting)
2. [Building Guide](../BUILDING.md)

---

## 📦 Quick Commands Reference

```bash
# Development
npm run tauri:dev                    # Run app in dev mode

# Building
npm run tauri:build                  # Build installers for your OS

# Version Management
npm run version:update 1.0.0         # Update version in all files

# Finding Installers
ls src-tauri/target/release/bundle/  # List all built installers

# Creating Release
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0
```

---

## 🎓 Learning Path

### Beginner Path
1. Read [Packaging Quick Start](PACKAGING_QUICK_START.md) (15 min)
2. Run `npm run tauri:build` (15 min)
3. Test your installer (5 min)
4. Share it! 🎉

**Total time: ~35 minutes**

### Intermediate Path
1. Read [Distribution Guide](DISTRIBUTION_GUIDE.md) (30 min)
2. Set up code signing (varies)
3. Create GitHub Release
4. Share with users

**Total time: 1-2 hours (plus certificate setup)**

### Advanced Path
1. Read full [Distribution Guide](DISTRIBUTION_GUIDE.md)
2. Set up GitHub Actions (`.github/workflows/release.yml.example`)
3. Configure auto-updates
4. Set up code signing for all platforms
5. Automate everything!

**Total time: 3-5 hours (one-time setup)**

---

## 📋 Documentation Features

All guides include:
- ✅ Step-by-step instructions
- ✅ Copy-paste commands
- ✅ Real examples
- ✅ Troubleshooting sections
- ✅ Platform-specific guidance
- ✅ Size/time estimates

---

## 🔍 Find Information By Topic

### Building & Compilation
- [Building Guide](../BUILDING.md)
- [Distribution Guide - Building Section](DISTRIBUTION_GUIDE.md#quick-start)
- [Packaging Quick Start](PACKAGING_QUICK_START.md)

### Installers & Packages
- [Where Are My Installers?](WHERE_ARE_MY_INSTALLERS.md)
- [Distribution Guide - Platform Packages](DISTRIBUTION_GUIDE.md#platform-specific-packages)

### Code Signing
- [Distribution Guide - Code Signing](DISTRIBUTION_GUIDE.md#code-signing)

### Distribution Channels
- [Distribution Guide - Distribution Channels](DISTRIBUTION_GUIDE.md#distribution-channels)

### Automation
- [Distribution Guide - CI/CD](DISTRIBUTION_GUIDE.md#cicd-automation)
- `.github/workflows/release.yml.example`

### Updates
- [Distribution Guide - Auto-Updates](DISTRIBUTION_GUIDE.md#auto-updates)

### Troubleshooting
- [Distribution Guide - Troubleshooting](DISTRIBUTION_GUIDE.md#troubleshooting)
- [Where Are My Installers? - Troubleshooting](WHERE_ARE_MY_INSTALLERS.md#troubleshooting)

---

## 📱 Platform-Specific Information

### macOS
- Universal binaries (Intel + Apple Silicon) ✅
- DMG creation: [Distribution Guide](DISTRIBUTION_GUIDE.md#macos)
- Code signing: [Distribution Guide - macOS Signing](DISTRIBUTION_GUIDE.md#macos-code-signing)
- Notarization: [Distribution Guide - macOS Signing](DISTRIBUTION_GUIDE.md#macos-code-signing)

### Windows
- MSI and NSIS installers: [Distribution Guide](DISTRIBUTION_GUIDE.md#windows)
- Code signing: [Distribution Guide - Windows Signing](DISTRIBUTION_GUIDE.md#windows-code-signing)
- SmartScreen: [Distribution Guide - Troubleshooting](DISTRIBUTION_GUIDE.md#troubleshooting)

### Linux
- Multiple formats (deb, rpm, AppImage): [Distribution Guide](DISTRIBUTION_GUIDE.md#linux)
- AppImage (universal): [Where Are My Installers?](WHERE_ARE_MY_INSTALLERS.md#-linux)
- Package managers: [Distribution Guide](DISTRIBUTION_GUIDE.md#linux)

---

## 🛠 Tools & Scripts

### Version Management
```bash
npm run version:update 1.0.0
```
Updates version in:
- `package.json`
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `Cargo.toml`

### GitHub Actions (Example)
`.github/workflows/release.yml.example`
- Auto-builds for macOS, Windows, Linux
- Creates GitHub Releases
- Uploads installers automatically

### Changelog
`CHANGELOG.md`
- Track changes between versions
- Keep a Changelog format
- Templates included

---

## 💡 Tips & Best Practices

### Before Release
- ✅ Test on clean system
- ✅ Update version numbers
- ✅ Update CHANGELOG
- ✅ Run full test suite
- ✅ Check installer works

### Distribution
- ✅ Use GitHub Releases for free hosting
- ✅ Provide installers for all platforms
- ✅ Write clear release notes
- ✅ Include screenshots

### Long Term
- ✅ Set up automated releases (GitHub Actions)
- ✅ Code sign for production
- ✅ Consider auto-updates
- ✅ Monitor download statistics

---

## 📞 Getting Help

### Documentation
- Start with [Packaging Quick Start](PACKAGING_QUICK_START.md)
- Full details in [Distribution Guide](DISTRIBUTION_GUIDE.md)
- Troubleshooting in each guide

### External Resources
- **Tauri Docs:** https://tauri.app/
- **Tauri Discord:** https://discord.gg/tauri
- **GitHub Issues:** Report bugs

### Common Issues
See [Distribution Guide - Troubleshooting](DISTRIBUTION_GUIDE.md#troubleshooting) for:
- Build failures
- Code signing issues
- Installer problems
- Platform-specific errors

---

## 🎉 Success Checklist

You're ready to distribute when:

- [x] App builds successfully (`npm run tauri:build`)
- [x] Installers are created (check `bundle/` directory)
- [x] You've tested the installer
- [x] Version numbers are updated
- [x] CHANGELOG is updated
- [x] You have a distribution plan (GitHub/website/etc.)

**Ready?** Go to [Packaging Quick Start](PACKAGING_QUICK_START.md)!

---

## 📖 Document Summaries

### [Packaging Quick Start](PACKAGING_QUICK_START.md) (~5 min read)
**Best for:** First-time users  
**Covers:** Fast path to your first release in 15 minutes  
**Contains:** Step-by-step guide, common questions, quick reference

### [Distribution Guide](DISTRIBUTION_GUIDE.md) (~20 min read)
**Best for:** Complete understanding  
**Covers:** Everything about packaging and distribution  
**Contains:** Detailed instructions, all platforms, advanced topics

### [Where Are My Installers?](WHERE_ARE_MY_INSTALLERS.md) (~3 min read)
**Best for:** Finding your built files  
**Covers:** Exact locations, file formats, what to share  
**Contains:** File paths, platform breakdown, commands

### [Building Guide](../BUILDING.md) (~5 min read)
**Best for:** Developer setup  
**Covers:** Prerequisites, development, production builds  
**Contains:** Commands, troubleshooting, quick reference

### [Release Checklist](../RELEASE_CHECKLIST.md) (~10 min read)
**Best for:** Creating releases  
**Covers:** Complete release process  
**Contains:** Checklist, testing steps, post-release tasks

---

## 🚀 Next Steps

1. **First Time?**  
   Start with [Packaging Quick Start](PACKAGING_QUICK_START.md)

2. **Want Details?**  
   Read [Distribution Guide](DISTRIBUTION_GUIDE.md)

3. **Ready to Release?**  
   Follow [Release Checklist](../RELEASE_CHECKLIST.md)

4. **Need Help?**  
   Check troubleshooting in any guide

---

**Happy distributing!** Your Tauri app is ready to reach users worldwide. 🌍

