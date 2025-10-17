# Changelog

All notable changes to Shogi Vibe will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial Tauri-based desktop application
- High-performance Rust engine with USI protocol
- Multiple AI difficulty levels (Easy, Medium, Hard)
- Advanced opening book system
- Beautiful UI with multiple themes
- Drag-and-drop and click-to-move piece controls
- Visual indicators for legal moves, check, and last move
- Sound effects for piece movements
- Move history tracking
- Undo functionality

### Changed
- Migrated from WebAssembly to native Tauri architecture
- Engine now runs as standalone binary via USI protocol
- Improved performance with native compilation

### Fixed
- N/A (initial release)

## [0.1.0] - YYYY-MM-DD

### Added
- Initial release of Shogi Vibe
- Full implementation of Shogi rules
- AI opponent with opening book
- Cross-platform support (macOS, Windows, Linux)
- Theme customization
- Audio settings

---

## Template for New Releases

```markdown
## [X.Y.Z] - YYYY-MM-DD

### Added
- New features go here

### Changed
- Changes to existing functionality go here

### Deprecated
- Features that will be removed in future releases go here

### Removed
- Features that were removed go here

### Fixed
- Bug fixes go here

### Security
- Security-related changes go here
```

---

## Version History Format

- **Added**: New features
- **Changed**: Changes in existing functionality
- **Deprecated**: Soon-to-be removed features
- **Removed**: Removed features
- **Fixed**: Bug fixes
- **Security**: Security improvements

[Unreleased]: https://github.com/yourusername/shogi-game/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yourusername/shogi-game/releases/tag/v0.1.0

