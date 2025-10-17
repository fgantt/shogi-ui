# Release Checklist

Use this checklist when preparing a new release of Shogi Vibe.

## Pre-Release Preparation

### Version Updates
- [ ] Update version in `src-tauri/tauri.conf.json`
- [ ] Update version in `src-tauri/Cargo.toml`
- [ ] Update version in `package.json`
- [ ] Update version in root `Cargo.toml`

### Documentation
- [ ] Update CHANGELOG.md with new features and fixes
- [ ] Review and update README.md if needed
- [ ] Check that all documentation is current

### Code Quality
- [ ] Run linter and fix issues: `npm run lint`
- [ ] Run type checks: `npm run type-check`
- [ ] Run tests (if applicable): `npm test`
- [ ] Check for compiler warnings in Rust code

## Testing

### Functionality Testing
- [ ] Test all game features work correctly
- [ ] Test AI at all difficulty levels
- [ ] Test drag-and-drop functionality
- [ ] Test click-to-move functionality
- [ ] Test piece promotion
- [ ] Test undo functionality
- [ ] Test new game functionality
- [ ] Test settings persistence
- [ ] Test sound effects (on/off)
- [ ] Test all themes

### UI/UX Testing
- [ ] Check all visual themes render correctly
- [ ] Verify all UI elements are accessible
- [ ] Test window resizing
- [ ] Check that all icons/images load
- [ ] Verify no console errors

### Platform-Specific Testing
- [ ] Test on macOS (if building for Mac)
- [ ] Test on Windows (if building for Windows)
- [ ] Test on Linux (if building for Linux)

## Building

### Local Build
- [ ] Clean previous builds: `cargo clean`
- [ ] Build release: `npm run tauri:build`
- [ ] Verify build completed successfully
- [ ] Check bundle size is reasonable

### Test Installers
- [ ] Install from the generated installer
- [ ] Launch the app
- [ ] Verify app name and version are correct
- [ ] Test uninstaller (if applicable)

## Code Signing (If Applicable)

### macOS
- [ ] Code sign the app bundle
- [ ] Notarize with Apple
- [ ] Staple notarization ticket
- [ ] Verify DMG opens without warnings

### Windows
- [ ] Sign the installer with certificate
- [ ] Verify no SmartScreen warnings

## Release

### Git
- [ ] Commit all version changes
- [ ] Create and push version tag: `git tag -a v1.0.0 -m "Release v1.0.0"`
- [ ] Push tag: `git push origin v1.0.0`

### GitHub Release
- [ ] Create new release on GitHub
- [ ] Use version tag (e.g., v1.0.0)
- [ ] Add release title: "Shogi Vibe v1.0.0"
- [ ] Write release notes describing changes
- [ ] Upload all installers:
  - [ ] macOS .dmg
  - [ ] Windows .exe
  - [ ] Windows .msi
  - [ ] Linux .deb
  - [ ] Linux .AppImage
  - [ ] Linux .rpm
- [ ] Mark as pre-release if applicable
- [ ] Publish release

### Update Manifest (If Using Auto-Updates)
- [ ] Generate/update latest.json
- [ ] Upload to update server
- [ ] Test auto-update functionality

## Post-Release

### Verification
- [ ] Download installers from release page
- [ ] Test installation from downloads
- [ ] Verify download links work
- [ ] Check release appears in GitHub releases

### Communication
- [ ] Announce release (if applicable)
- [ ] Update website/documentation
- [ ] Post on social media (if applicable)
- [ ] Notify users/testers

### Monitoring
- [ ] Monitor for user-reported issues
- [ ] Check download statistics
- [ ] Watch for crash reports

## Version Number Format

Use semantic versioning: `MAJOR.MINOR.PATCH`

- **MAJOR**: Incompatible API changes
- **MINOR**: New functionality (backwards compatible)
- **PATCH**: Bug fixes (backwards compatible)

Examples:
- `1.0.0` - Initial release
- `1.1.0` - Added new feature
- `1.1.1` - Fixed bug
- `2.0.0` - Major rewrite/breaking changes

## Quick Commands

```bash
# Update versions (do this manually in each file)
# - src-tauri/tauri.conf.json -> "version"
# - src-tauri/Cargo.toml -> [package] version
# - package.json -> "version"
# - Cargo.toml -> [package] version

# Clean and build
cargo clean
npm run tauri:build

# Create and push tag
git add .
git commit -m "Release v1.0.0"
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin main
git push origin v1.0.0

# Sign macOS (example)
xcrun notarytool submit "bundle/dmg/Shogi Vibe_1.0.0_universal.dmg" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Sign Windows (example)
signtool sign /tr http://timestamp.digicert.com /td sha256 /fd sha256 /a "bundle/nsis/Shogi Vibe_1.0.0_x64-setup.exe"
```

## Troubleshooting

### Build Fails
- Clean and rebuild: `cargo clean && npm run tauri:build`
- Check Rust version: `rustc --version`
- Check Node version: `node --version`
- Update dependencies: `npm install` and `cargo update`

### Installer Issues
- Verify all files are included in bundle
- Check bundle configuration in tauri.conf.json
- Review build logs for warnings

### Code Signing Issues
- Verify certificate is valid and not expired
- Check certificate thumbprint/identity
- Ensure proper permissions for signing

## Notes

- Always test on clean systems when possible
- Keep installers for historical reference
- Document any platform-specific issues
- Consider beta testing for major releases

