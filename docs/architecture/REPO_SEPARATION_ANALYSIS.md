# Repository Separation Analysis: UI and Engine

**Date:** December 2024  
**Status:** Analysis Document

## Executive Summary

This document analyzes the feasibility and approach for separating the Shogi UI and Engine into separate git repositories. The current architecture already treats the engine as a separate process (USI protocol), which makes separation more straightforward than a tightly-coupled architecture.

---

## Current Architecture

### Structure
- **Monorepo:** Single repository containing both UI and Engine
- **Engine Location:** `src/` (Rust crate: `shogi-engine`)
- **UI Location:** `src/components/`, `src/App.tsx`, etc. (React/TypeScript)
- **Tauri Bridge:** `src-tauri/` (Rust code that bridges UI and Engine)

### Integration Points

1. **Build Process:**
   - `package.json` includes `"build:engine": "cargo build --bin usi-engine --release"`
   - Engine is built as part of UI build process
   - Binary output: `target/release/usi-engine`

2. **Communication:**
   - Engine runs as **separate process** (already decoupled)
   - Communication via **USI protocol** over stdin/stdout
   - Tauri layer (`src-tauri/src/engine_manager.rs`) spawns engine process
   - No direct code dependencies between UI and Engine code

3. **Engine Path Resolution:**
   - Development: `target/release/usi-engine` (or `target/debug/usi-engine`)
   - Production: Bundled with Tauri app
   - Path resolution logic in `src-tauri/src/commands.rs::get_builtin_engine_path()`

### Key Files

**Engine:**
- `Cargo.toml` (root) - Engine crate definition
- `src/main.rs` - USI engine binary entry point
- `src/lib.rs` - Engine library
- All engine code in `src/` subdirectories

**UI:**
- `package.json` - Frontend dependencies and build scripts
- `src/components/` - React components
- `src-tauri/` - Tauri bridge layer

**Integration:**
- `src-tauri/src/engine_manager.rs` - Spawns and manages engine processes
- `src-tauri/src/commands.rs` - Tauri commands, including engine path resolution
- `src/utils/tauriEngine.ts` - Frontend engine communication utilities

---

## Separation Benefits

### Advantages

1. **Independent Versioning:**
   - Engine can be versioned separately (e.g., `shogi-engine v2.1.0`)
   - UI can be versioned separately (e.g., `shogi-ui v1.5.0`)
   - Clearer release cycles

2. **Independent Development:**
   - Engine developers don't need UI dependencies
   - UI developers don't need Rust toolchain
   - Faster CI/CD (only build what changed)

3. **Reusability:**
   - Engine can be used by other UIs/clients
   - Engine can be distributed as standalone tool
   - Easier to integrate into other projects

4. **Clearer Ownership:**
   - Separate issue trackers
   - Separate documentation
   - Separate release notes

5. **Smaller Repositories:**
   - Faster clones
   - Clearer project boundaries
   - Easier to understand codebase

### Challenges

1. **Distribution:**
   - Need to decide how UI gets engine binary
   - Options: Pre-built binaries, package manager, download on first run
   - Cross-platform considerations (Windows, macOS, Linux)

2. **Development Workflow:**
   - Developers working on both need to manage two repos
   - Testing integration requires both repos
   - More complex local development setup

3. **Version Compatibility:**
   - Need to ensure UI and Engine versions are compatible
   - May need compatibility matrix/documentation
   - Breaking changes require coordination

4. **Build Process:**
   - UI build process needs to obtain engine binary
   - CI/CD becomes more complex (two repos to build)
   - Release process requires coordination

---

## Separation Approach

### Option 1: Pre-built Binaries (Recommended)

**How it works:**
- Engine repo builds and releases binaries for each platform
- UI repo downloads/uses pre-built binaries
- Binaries stored in GitHub Releases or package registry

**Pros:**
- Simple for UI developers (no Rust toolchain needed)
- Fast UI builds (no engine compilation)
- Clear separation of concerns

**Cons:**
- Need to build binaries for all platforms
- Binary size in releases
- Need infrastructure for binary distribution

**Implementation:**
1. Engine repo: GitHub Actions builds binaries on release
2. Engine repo: Uploads binaries to GitHub Releases
3. UI repo: Downloads engine binary during build or first run
4. UI repo: Bundles engine binary with Tauri app

### Option 2: Package Manager

**How it works:**
- Engine published as package (e.g., npm package, cargo crate)
- UI installs engine as dependency
- Engine binary included in package

**Pros:**
- Standard distribution mechanism
- Version management via package manager
- Easy updates

**Cons:**
- Need to set up package publishing
- Binary size in package
- Platform-specific packages may be needed

**Implementation:**
1. Engine repo: Publishes to npm/crates.io with binaries
2. UI repo: Adds engine as dependency
3. UI repo: Uses engine binary from `node_modules` or similar

### Option 3: Git Submodule

**How it works:**
- Engine repo included as git submodule in UI repo
- UI builds engine from submodule during build

**Pros:**
- Always uses specific engine version
- Can modify engine locally if needed
- Simple version pinning

**Cons:**
- UI developers still need Rust toolchain
- Slower builds (compiles engine)
- Git submodule complexity

**Implementation:**
1. UI repo: Add engine as git submodule
2. UI repo: Build script compiles engine from submodule
3. UI repo: Uses compiled binary

### Option 4: Separate Download on First Run

**How it works:**
- UI app downloads engine binary on first launch
- Binary cached locally for subsequent runs
- Binary downloaded from CDN or GitHub Releases

**Pros:**
- Small initial app size
- Can update engine without app update
- Flexible version management

**Cons:**
- Requires internet connection on first run
- More complex error handling
- Need to handle download failures

**Implementation:**
1. Engine repo: Releases binaries to GitHub Releases
2. UI app: Checks for engine binary on startup
3. UI app: Downloads if missing or outdated
4. UI app: Caches binary locally

---

## Recommended Approach: Hybrid (Pre-built + Package Manager)

### Phase 1: Initial Separation

1. **Create Engine Repo:**
   - Move `src/` (engine code) to new repo
   - Move `Cargo.toml` (root) to new repo
   - Move engine-related docs to new repo
   - Set up CI/CD for building binaries

2. **Update UI Repo:**
   - Remove engine source code
   - Remove `build:engine` script (or make it download binary)
   - Update engine path resolution to use downloaded/bundled binary
   - Update documentation

3. **Distribution:**
   - Engine repo: GitHub Actions builds binaries on release
   - Engine repo: Uploads to GitHub Releases
   - UI repo: Downloads engine binary during build (or bundles pre-built)

### Phase 2: Package Manager Integration (Optional)

1. **Engine Repo:**
   - Publish engine as npm package (with binaries)
   - Or publish as cargo crate (if other Rust projects need it)

2. **UI Repo:**
   - Add engine as dependency
   - Use engine binary from package

---

## Migration Steps

### Step 1: Prepare Engine Repo

1. Create new repository: `shogi-engine`
2. Copy engine code:
   ```bash
   # In new engine repo
   cp -r <old-repo>/src .
   cp <old-repo>/Cargo.toml .
   cp <old-repo>/Cargo.lock .
   cp -r <old-repo>/benches .
   cp -r <old-repo>/tests .
   cp -r <old-repo>/examples .
   cp -r <old-repo>/config .
   cp -r <old-repo>/resources .
   ```
3. Update `Cargo.toml`:
   - Update package name if needed
   - Update repository URL
   - Remove any UI-specific features
4. Set up CI/CD:
   - GitHub Actions to build binaries for Windows, macOS, Linux
   - Upload binaries to GitHub Releases
5. Create engine-specific documentation:
   - README.md
   - API documentation
   - USI protocol documentation

### Step 2: Update UI Repo

1. Remove engine code:
   ```bash
   # In UI repo
   rm -rf src/bitboards src/search src/evaluation src/opening_book src/tablebase src/tuning src/types/*.rs
   rm Cargo.toml Cargo.lock
   rm -rf benches tests examples config resources
   ```
2. Update `package.json`:
   ```json
   {
     "scripts": {
       "build:engine": "node scripts/download-engine.js",  // Download instead of build
       // Or remove if bundling pre-built
     }
   }
   ```
3. Update `src-tauri/src/commands.rs`:
   - Update `get_builtin_engine_path()` to look for downloaded/bundled binary
   - Remove workspace root detection logic
4. Create download script (if using Option 1):
   ```javascript
   // scripts/download-engine.js
   // Downloads engine binary from GitHub Releases
   ```
5. Update Tauri build process:
   - Bundle engine binary with app
   - Or download during app initialization

### Step 3: Update Documentation

1. **Engine Repo:**
   - README.md with build instructions
   - USI protocol documentation
   - API documentation
   - Release notes

2. **UI Repo:**
   - Update README.md to reference engine repo
   - Update development setup instructions
   - Update build instructions

### Step 4: Update CI/CD

1. **Engine Repo:**
   - Build binaries on release
   - Run tests
   - Upload to GitHub Releases

2. **UI Repo:**
   - Download engine binary (or use bundled)
   - Build UI
   - Bundle engine binary with app

---

## Technical Considerations

### Engine Path Resolution

**Current:** Looks for engine in workspace `target/` directory

**After Separation:** Need to update to:
- Bundled binary location (production)
- Downloaded binary location (development)
- User-provided path (external engines)

**Code Changes:**
- `src-tauri/src/commands.rs::get_builtin_engine_path()` - Update path resolution
- `src-tauri/src/engine_manager.rs` - No changes needed (already uses path parameter)

### Tauri Bundle Configuration

**Current:** Engine built during Tauri build

**After Separation:** Need to:
- Include engine binary in Tauri bundle
- Update `tauri.conf.json` to include engine binary
- Or download engine on first run

**Configuration:**
```json
// tauri.conf.json
{
  "bundle": {
    "resources": [
      "engine-binary/usi-engine"  // Or platform-specific paths
    ]
  }
}
```

### Version Compatibility

**Recommendation:** Document compatibility matrix

**Example:**
| UI Version | Engine Version | Notes |
|------------|----------------|-------|
| 1.0.0      | 2.0.0+        | Requires USI protocol v1.0+ |
| 1.1.0      | 2.1.0+        | Requires new USI options |

### Development Workflow

**For developers working on both:**

1. Clone both repos
2. Build engine: `cd shogi-engine && cargo build --release`
3. Link engine to UI (symlink or copy binary)
4. Build UI: `cd shogi-ui && npm run tauri:dev`

**Or use a workspace script:**
```bash
# scripts/dev-setup.sh
# Clones both repos, builds engine, links to UI
```

---

## File Movement Checklist

### Move to Engine Repo

- [ ] `src/` (all engine source code)
- [ ] `Cargo.toml` (root)
- [ ] `Cargo.lock`
- [ ] `benches/`
- [ ] `tests/`
- [ ] `examples/`
- [ ] `config/` (engine configs)
- [ ] `resources/` (engine resources)
- [ ] `rustfmt.toml`
- [ ] Engine-specific documentation:
  - [ ] `docs/ENGINE_UTILITIES_GUIDE.md`
  - [ ] `docs/ENGINE_CONFIGURATION_GUIDE.md`
  - [ ] `docs/design/implementation/` (engine design docs)
  - [ ] `docs/performance/` (engine performance docs)
  - [ ] `docs/tuning/` (engine tuning docs)

### Keep in UI Repo

- [ ] `src/components/` (React components)
- [ ] `src/App.tsx`
- [ ] `src/main.tsx`
- [ ] `src-tauri/` (Tauri bridge)
- [ ] `package.json`
- [ ] `vite.config.ts`
- [ ] `public/` (UI assets)
- [ ] `dist/` (build output)
- [ ] UI-specific documentation:
  - [ ] `docs/user/` (user guides)
  - [ ] `docs/design/` (UI design docs)
  - [ ] `docs/development/` (UI development docs)

### Shared/Decide

- [ ] `docs/architecture/` - Split between repos or keep in UI?
- [ ] `docs/design/architecture/` - Split between repos or keep in UI?
- [ ] `README.md` - Create separate READMEs for each repo

---

## Risks and Mitigation

### Risk 1: Breaking Changes

**Risk:** Engine API changes break UI

**Mitigation:**
- Version engine releases
- Document breaking changes
- UI checks engine version on startup
- Maintain compatibility matrix

### Risk 2: Development Friction

**Risk:** Developers need to manage two repos

**Mitigation:**
- Provide setup scripts
- Document workflow clearly
- Consider monorepo tools (but defeats purpose)

### Risk 3: Binary Distribution Issues

**Risk:** Binaries not available or wrong platform

**Mitigation:**
- Automated CI/CD for all platforms
- Fallback to building from source (optional)
- Clear error messages

### Risk 4: Version Mismatch

**Risk:** UI and Engine versions incompatible

**Mitigation:**
- Version checking on startup
- Clear error messages
- Compatibility documentation

---

## Timeline Estimate

### Phase 1: Preparation (1-2 weeks)
- Create engine repo
- Set up CI/CD for engine
- Test binary distribution

### Phase 2: Migration (1-2 weeks)
- Move code to engine repo
- Update UI repo
- Update build processes
- Test integration

### Phase 3: Documentation (1 week)
- Update all documentation
- Create migration guide
- Update READMEs

### Phase 4: Testing (1 week)
- Test all platforms
- Test development workflow
- Test release process

**Total:** 4-6 weeks

---

## Decision Matrix

| Factor | Keep Monorepo | Separate Repos |
|--------|---------------|----------------|
| **Development Speed** | Faster (single repo) | Slower (two repos) |
| **Build Time** | Slower (builds both) | Faster (builds separately) |
| **Reusability** | Lower | Higher |
| **Clarity** | Lower | Higher |
| **CI/CD Complexity** | Simpler | More complex |
| **Version Management** | Coupled | Independent |
| **Onboarding** | Simpler | More complex |

---

## Recommendation

**Recommendation: Separate the repos**

**Rationale:**
1. Engine is already process-separated (USI protocol)
2. Clear separation of concerns
3. Better reusability
4. Independent versioning
5. Faster builds (UI doesn't need Rust toolchain)

**Approach:**
- Use **Option 1 (Pre-built Binaries)** for initial separation
- Consider **Option 2 (Package Manager)** for long-term distribution
- Provide clear documentation and tooling for developers

**Timeline:**
- Start with Phase 1 (Preparation)
- Complete migration over 4-6 weeks
- Maintain backward compatibility during transition

---

## Next Steps

If proceeding with separation:

1. **Review this document** with team
2. **Decide on distribution method** (pre-built binaries recommended)
3. **Create engine repository** and set up CI/CD
4. **Create migration plan** with specific dates
5. **Test separation** in a branch before merging
6. **Update documentation** as you go

---

## Questions to Answer

Before proceeding, consider:

1. **Distribution:** How will UI get engine binary?
   - Pre-built binaries (GitHub Releases)
   - Package manager (npm/cargo)
   - Build from source (git submodule)

2. **Versioning:** How to manage version compatibility?
   - Semantic versioning
   - Compatibility matrix
   - Version checking

3. **Development:** How to support developers working on both?
   - Setup scripts
   - Documentation
   - Local linking

4. **CI/CD:** How to coordinate builds?
   - Separate pipelines
   - Triggered builds
   - Release coordination

5. **Documentation:** Where to put shared docs?
   - Split between repos
   - Keep in UI repo
   - Separate docs repo

---

## References

- Current architecture: `docs/design/architecture/Universal-Shogi-Interface-Implementation.md`
- Engine utilities: `docs/ENGINE_UTILITIES_GUIDE.md`
- Tauri integration: `docs/development/GAMEPAGE_TAURI_INTEGRATION.md`

