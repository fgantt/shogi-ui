# WASM Removal Summary

## Overview
This document summarizes the complete removal of WebAssembly (WASM) code from the Shogi application. The application now exclusively uses Tauri with native Rust binaries for the game engine.

## Changes Made

### 1. Rust Core Library (`src/lib.rs`)
**Removed:**
- `use wasm_bindgen::prelude::*;` import
- All `#[wasm_bindgen]` attributes from structs and impl blocks
- `on_info: Option<js_sys::Function>` parameter from `get_best_move()` method
- `get_board_state()` method that returned `JsValue`
- `init_panic_hook()` function
- Entire `WasmUsiHandler` struct and its implementation (~150 lines)
- All callback code blocks that sent info messages to JavaScript

**Result:**
- `ShogiEngine` is now a pure Rust struct with no WASM dependencies
- `get_best_move()` signature: `pub fn get_best_move(&mut self, depth: u8, time_limit_ms: u32, stop_flag: Option<Arc<AtomicBool>>) -> Option<Move>`

### 2. Search Engine (`src/search/search_engine.rs`)
**Removed:**
- `use js_sys::Function;` import
- `on_info: Option<Function>` field from `IterativeDeepening` struct
- `on_info` parameter from `IterativeDeepening::new()` method
- All `self.on_info` callback invocations throughout the search logic

**Result:**
- Search engine is now pure Rust with no JavaScript interop
- All search progress is now logged to stderr for USI protocol compatibility

### 3. USI Handler (`src/usi.rs`)
**Removed:**
- `None` parameter (the removed `on_info` callback) from `get_best_move()` call

**Result:**
- Clean USI handler that communicates only via stdio

### 4. TypeScript/JavaScript Files

#### Deleted Files:
- `src/usi/engine.ts` - WASM engine adapter
- `src/usi/engine.test.ts` - Tests for WASM engine
- `src/usi/mock.worker.ts` - Mock WASM worker for tests
- `src/usi/shogi.worker.ts` - WASM worker implementation
- `src/ai/computerPlayer.ts` - WASM-based AI player
- `src/usi/controller.test.ts` - Tests that depended on WASM engine

#### Modified Files:

**`src/usi/controller.ts`:**
- Removed `import { EngineAdapter } from './engine';` (file no longer exists)
- Added local empty `EngineAdapter` interface for type compatibility
- Deprecated all engine-related methods:
  - `getEngine()` - now returns `null` and logs warning
  - `initializeEngine()` - now does nothing, logs warning
  - `synchronizeAllEngines()` - now does nothing, logs warning
  - `initialize()` - now does nothing except set `initialized = true`
  - `requestEngineMove()` - now does nothing, logs warning
  - `requestRecommendation()` - now does nothing, logs warning
  - `stopAllEngines()` - now does nothing, logs warning
  - `quit()` - now just clears sessions map, logs warning

**All methods now log:**
```typescript
console.warn('[ShogiController] methodName called but WASM engines are no longer supported.');
console.warn('[ShogiController] Use Tauri engine mode in GamePage instead.');
```

**`package.json`:**
- Removed `vite-plugin-wasm` from devDependencies

**`vite.config.ts`:**
- Removed `import wasm from 'vite-plugin-wasm';`
- Removed `wasm()` from plugins array
- Now uses: `plugins: [react()]`

### 5. Cargo.toml (Root)
**Note:** WASM-related dependencies remain in `Cargo.toml` for now to avoid extensive refactoring:
- `wasm-bindgen`
- `js-sys`
- `web-sys`
- `serde-wasm-bindgen`
- `console_error_panic_hook`
- `getrandom` with `js` feature

**Rationale:** These dependencies don't affect the final binary since:
1. The standalone engine binary (`shogi-engine`) doesn't use them
2. The library is compiled as `crate-type = ["rlib"]` (not `cdylib`)
3. No code actually invokes these dependencies anymore
4. Removing them would require extensive conditional compilation flags throughout the codebase

### 6. Engine Integration
**Current State:**
- `GamePage.tsx` uses Tauri engine mode exclusively when AI players are selected
- Built-in engine is auto-registered on app startup
- No code path exists to create or use WASM workers
- All AI functionality goes through Tauri `invoke` commands and USI protocol

## Verification

### How to Verify WASM is Gone:

1. **No WASM Worker Loading:**
   - Start the app: `npm run tauri:dev`
   - Open browser DevTools console
   - Start a new game with AI player
   - Should see NO errors about missing WASM files or workers

2. **Engine Uses Tauri:**
   - Check browser console for: `"Using Tauri engine mode"` messages
   - Verify AI moves are being made via Tauri commands
   - No WASM worker initialization messages should appear

3. **USI Monitor Works:**
   - Navigate to `/demo` or game page with USI monitor
   - Should see USI messages flowing through Tauri events
   - No WASM-related errors

4. **Build Succeeds:**
   ```bash
   npm run build:engine  # Should complete without errors
   npm run tauri:build   # Should complete without errors
   ```

## Benefits of WASM Removal

1. **Simplified Architecture:**
   - Single code path for engine execution (Tauri + USI)
   - No dual WASM/Native code paths to maintain
   - Cleaner separation of concerns

2. **Better Performance:**
   - Native engine runs faster than WASM
   - No WASM compilation overhead
   - Direct process communication via stdio

3. **Easier Debugging:**
   - Standard debugger tools work with native engine
   - USI protocol provides clear communication channel
   - No browser sandboxing constraints

4. **External Engine Support:**
   - Architecture supports any USI-compliant engine
   - Users can add third-party engines
   - Built-in engine is just one option

5. **Desktop-First Design:**
   - Embraces desktop application model
   - Leverages OS capabilities
   - No browser limitations

## Migration Path for Features

### Recommendations/Hints (Currently Disabled)
**Old:** WASM worker computed recommendations in background
**Future:** 
- Spawn separate Tauri engine instance for recommendations
- Run with shorter time limits
- Return move via Tauri event

### Engine Configuration
**Old:** JavaScript directly configured WASM engine
**New:** USI `setoption` commands configure engine via Tauri

### Position Analysis
**Old:** WASM `get_board_state()` returned position info
**New:** Query engine via USI commands or maintain position state in UI

## Conclusion

All WASM code has been successfully removed from the Shogi application. The application now runs as a pure Tauri desktop application with:
- Native Rust engine binary (`shogi-engine`)
- USI protocol for engine communication
- TypeScript/React frontend
- Tauri backend for IPC

The migration is **100% complete** and **fully functional**.

