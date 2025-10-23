# WASM to Tauri Cleanup Plan

**Date:** December 2024  
**Status:** In Progress  
**Goal:** Complete removal of WASM-related code and dependencies after successful Tauri migration

## Overview

This document outlines the systematic cleanup of WebAssembly (WASM) code and dependencies from the Shogi Game codebase after the successful migration to Tauri. The application now runs as a native desktop application with USI engine support, making WASM components obsolete.

## Current State

✅ **Completed:**
- Tauri migration is complete and functional
- USI engine integration is working
- Main application flow no longer uses WASM
- App runs as native desktop application

❌ **Needs Cleanup:**
- WASM dependencies remain in Cargo.toml
- WASM-related files and directories exist
- Documentation references outdated WASM architecture
- Deprecated code paths remain

---

## Phase 1: Remove WASM Dependencies and Files (High Priority)

### 1.1 Remove WASM Package Directories
- [x] Delete `pkg/` directory (contains compiled WASM artifacts)
- [x] Delete `pkg-bundler/` directory (contains bundled WASM artifacts)
- [x] Verify no build scripts reference these directories

### 1.2 Clean Up Cargo.toml Dependencies
- [x] Remove `wasm-bindgen = "0.2"` from main Cargo.toml
- [x] Remove `js-sys = "0.3"` from main Cargo.toml
- [x] Remove `web-sys = { version = "0.3", features = ["console", "Window", "Performance"] }` from main Cargo.toml
- [x] Remove `serde-wasm-bindgen = "0.6"` from main Cargo.toml
- [x] Remove `console_error_panic_hook = "0.1"` from main Cargo.toml
- [x] Remove `getrandom = { version = "0.2", features = ["js"] }` from main Cargo.toml
- [x] Test that the project still compiles after dependency removal
- [x] Update any conditional compilation flags that reference WASM

### 1.3 Remove WASM-Specific Source Files
- [x] Delete `src/evaluation/wasm_compatibility.rs`
- [x] Delete `src/search/wasm_compatibility.rs`
- [x] Delete `src/search/wasm_transposition_table.rs`
- [x] Delete `src/search/wasm_benchmarks.rs`
- [x] Delete `src/usi/engine.ts` (WASM engine adapter)
- [x] Update any imports that reference these deleted files
- [x] Test compilation after file removal

---

## Phase 2: Clean Up Deprecated Code (Medium Priority)

### 2.1 Remove Unused TauriShogiController
- [x] Delete `src/usi/tauriController.ts` (unused/abandoned implementation)
- [x] Verify no imports or references to TauriShogiController exist
- [x] Test that application still works after removal

### 2.2 Simplify ShogiController
- [x] Remove all deprecated WASM methods from `src/usi/controller.ts`:
  - [x] Remove `getEngine()` method
  - [x] Remove `initializeEngine()` method
  - [x] Remove `synchronizeAllEngines()` method
  - [x] Remove `requestEngineMove()` method
  - [x] Remove `requestRecommendation()` method
  - [x] Remove `stopAllEngines()` method
  - [x] Remove `quit()` method
- [x] Remove WASM-related imports and interfaces
- [x] Simplify controller to only handle game state management
- [x] Update any code that calls these deprecated methods
- [x] Test that game functionality still works with Tauri engines
- [x] Fix runtime errors caused by removed methods (App.tsx, GamePage.tsx)

### 2.3 Clean Up TypeScript/JavaScript Files
- [x] Remove any remaining WASM worker references
- [x] Update imports that reference deleted WASM files
- [x] Remove unused WASM-related type definitions
- [x] Test TypeScript compilation

---

## Phase 3: Archive Documentation (Low Priority)

### 3.1 Move WASM Documentation to Archive
- [x] Move `docs/design/architecture/README_WASM_IMPLEMENTATION.md` to `docs/archive/`
- [x] Move `docs/design/architecture/WEBASSEMBLY_INTEGRATION_GUIDE.md` to `docs/archive/`
- [x] Move `docs/design/architecture/WEBASSEMBLY_BITBOARDS_IMPLEMENTATION.md` to `docs/archive/`
- [x] Move `docs/MOVE_ORDERING_WASM.md` to `docs/archive/`
- [x] Move `docs/EVALUATION_CACHE_WASM.md` to `docs/archive/`
- [x] Move `docs/design/implementation/evaluation-optimizations/tapered-evaluation/WASM_COMPATIBILITY_GUIDE.md` to `docs/archive/`
- [x] Move `docs/design/implementation/evaluation-optimizations/tapered-evaluation/TASK_3_5_COMPLETION_SUMMARY.md` to `docs/archive/`
- [x] Move `docs/design/implementation/evaluation-optimizations/tapered-evaluation/PHASE_3_COMPLETION_SUMMARY.md` to `docs/archive/`
- [x] Move `docs/design/implementation/evaluation-optimizations/tapered-evaluation/IMPLEMENTATION_COMPLETE.md` to `docs/archive/`
- [x] Move `docs/design/implementation/evaluation-caching/PHASE_3_MEDIUM_LOW_PRIORITY_COMPLETION_SUMMARY.md` to `docs/archive/`
- [x] Move `docs/design/implementation/bitboard-optimizations/bit-scanning/BIT_SCANNING_OPTIMIZATION_TASKS.md` to `docs/archive/`
- [x] Move `docs/design/implementation/bitboard-optimizations/bit-scanning/BIT_SCANNING_OPTIMIZATION_DESIGN.md` to `docs/archive/`

### 3.2 Update Remaining Documentation
- [x] Update `docs/README.md` to remove WASM references
- [x] Update `docs/architecture/` files to reflect Tauri-only architecture
- [x] Update API documentation to focus on Tauri/USI integration
- [x] Remove WASM references from development guides

---

## Phase 4: Build Configuration Cleanup (Low Priority)

### 4.1 Clean Up Build Scripts
- [x] Review `build.sh` for WASM-specific build steps
- [x] Remove or update any WASM-related build configurations
- [x] Ensure build scripts work with Tauri-only architecture

### 4.2 Clean Up Test Files
- [x] Remove test files that specifically test WASM functionality
- [x] Update integration tests to use Tauri engines instead
- [x] Ensure all tests pass after cleanup

---

## Phase 5: Final Verification (High Priority)

### 5.1 Build Verification
- [x] Run `npm run build:engine` - should complete without errors
- [x] Run `npm run tauri:build` - should complete without errors
- [x] Verify no WASM-related build warnings or errors

### 5.2 Runtime Verification
- [x] Start the app: `npm run tauri:dev`
- [x] Open browser DevTools console
- [x] Start a new game with AI player
- [x] Verify NO errors about missing WASM files or workers
- [x] Check console for "Using Tauri engine mode" messages
- [x] Verify AI moves are being made via Tauri commands
- [x] Test USI Monitor functionality

### 5.3 Code Quality Verification
- [x] Run `npm run lint` - should pass without errors
- [x] Run `npm run type-check` - should pass without errors
- [x] Run `npm run test` - should pass without errors
- [x] Verify no unused imports or dead code

---

## Benefits of This Cleanup

1. **Reduced Binary Size**: Removing WASM dependencies will reduce the final binary size
2. **Simplified Maintenance**: No more dual code paths to maintain
3. **Clearer Architecture**: Documentation will accurately reflect the current Tauri-only architecture
4. **Better Performance**: No unused dependencies in the build process
5. **Reduced Confusion**: New developers won't be confused by outdated WASM references
6. **Cleaner Codebase**: Easier to understand and maintain

---

## Risk Assessment

- **Low Risk**: Removing WASM package directories and archiving documentation
- **Medium Risk**: Removing WASM dependencies (may require some code adjustments)
- **High Risk**: Modifying the ShogiController (should be done carefully with testing)

---

## Notes

- All documentation files are moved to `docs/archive/` instead of being deleted
- Test each phase thoroughly before moving to the next
- Keep backups of important files before making changes
- If any step fails, investigate and fix before proceeding

---

## Progress Tracking

**Phase 1 Progress:** 15/15 tasks completed ✅  
**Phase 2 Progress:** 12/12 tasks completed ✅  
**Phase 3 Progress:** 15/15 tasks completed ✅  
**Phase 4 Progress:** 4/4 tasks completed ✅  
**Phase 5 Progress:** 8/8 tasks completed ✅  

**Overall Progress:** 54/58 tasks completed (93%)
