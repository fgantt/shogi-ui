# Endgame Detection - Current Status and Issues

**Date**: October 8, 2025  
**Status**: ⚠️ BLOCKED BY SYNCHRONIZATION ISSUES  
**Priority**: CRITICAL

---

## 🎯 Original Goal

Fix the bug where AI searches endlessly when checkmated without displaying the CheckmateModal.

---

## 📊 What Was Attempted

### Fixes Implemented

1. ✅ **Created `checkGameOver()` function** - UI-level detection (had false positives, disabled)
2. ✅ **Added CheckmateModal to JSX** - Modal now renders when winner state is set
3. ✅ **Handle AI resignation** - Detects when AI returns `"resign"`
4. ✅ **Added gameOver event flow** - Controller → UI event propagation
5. ✅ **Fixed Rust panic** - Added early legal move check before search
6. ✅ **Move validation** - Check if `applyMove()` succeeds (revealed sync issues)
7. ✅ **Comprehensive logging** - Track all state changes

---

## 🐛 Issues Discovered

### Issue #1: WASM/tsshogi Synchronization Problem (CRITICAL)

**Symptom**: WASM engine and tsshogi Record have different board positions

**Evidence**:
```
[CONTROLLER] createMoveByUSI returned: _Move {...}  ← Move object created
[CONTROLLER] record.append result: false            ← But rejected by Record
```

**Root Cause**: The WASM engine's internal board state diverges from the tsshogi Record

**Example**:
- WASM thinks: White to move, returns `6d6e`
- tsshogi Record thinks: Black to move
- `record.append()` rejects the move as illegal

**Position shows**: `9/9/ppp4pp/3ppP+B2/8k/2P3R1P/PP1PP1PP1/9/LNSGKGSNL w 2P 1`
- This position has issues (missing pieces, wrong turn)

### Issue #2: WASM Memory Corruption (CRITICAL)

**Symptom**: Runtime errors with memory access out of bounds

**Evidence**:
```
RuntimeError: memory access out of bounds
at dlmalloc::dlmalloc::Dlmalloc<A>::malloc::hff958f53bae633e3
at shogi_engine::moves::MoveGenerator::generate_quiescence_moves
```

**Impact**: WASM engine crashes during move generation or cleanup

### Issue #3: False Checkmate Detection

**Symptom**: CheckmateModal appeared on moves 1-3 when no checkmate exists

**Root Cause**: 
- `checkGameOver()` function too slow/buggy (disabled)
- Move application failures incorrectly triggered game over (now disabled)

---

## 🚨 Current State

### What Works ✅
- CheckmateModal component exists and renders
- Event flow from controller to UI
- AI resignation detection (when it returns "resign")
- Comprehensive logging for debugging
- Build system functioning

### What's Broken ❌
- **WASM/tsshogi synchronization** ← BLOCKING ISSUE
- **WASM memory corruption** ← SERIOUS BUG
- Moves being rejected by Record
- Game state inconsistency
- Endgame detection unreliable

### What's Disabled ⚠️
- UI-level `checkGameOver()` function (had false positives)
- Game-over on move failure (revealed sync issues)

---

## 🔍 Root Cause Analysis

### The Core Problem

The system has **two separate game state representations**:

1. **WASM Engine State** (Rust `BitboardBoard`)
   - Used for move search and evaluation
   - Has its own internal position

2. **tsshogi Record** (TypeScript/WASM)
   - Official game record
   - Used for UI display and move validation

**These two states are NOT synchronized properly**, leading to:
- WASM generates moves for one position
- tsshogi validates moves against a different position
- Moves get rejected even though they're valid for WASM's position

### Why This Happened

Looking at the flow:
1. User/AI makes move
2. `controller.applyMove()` updates tsshogi Record
3. Engines are supposed to be synchronized via `setPosition()`
4. **BUT**: Synchronization is failing somewhere
5. Engines drift out of sync with Record
6. Invalid moves proposed, crashes occur

---

## 🛠️ Recommended Path Forward

### Option A: Fix the Synchronization (PROPER FIX)

**What to do**:
1. Investigate why `engine.setPosition()` isn't working
2. Ensure engines sync after every move
3. Add validation that engine position matches Record
4. Fix WASM memory issues

**Pros**: Solves root cause, enables proper endgame detection
**Cons**: Time-consuming, touches core engine architecture
**Estimated Time**: 2-3 days

### Option B: Use Only tsshogi for Endgame Detection (WORKAROUND)

**What to do**:
1. Remove reliance on WASM engine for move validation
2. Implement checkmate detection using only tsshogi API
3. Check if current player has legal moves by testing all pieces
4. Optimize the checking logic

**Pros**: Avoids sync issues, simpler
**Cons**: Doesn't fix underlying sync problem
**Estimated Time**: 4-6 hours

### Option C: Minimal Fix - Resign Detection Only (QUICKEST)

**What to do**:
1. Keep current code (AI resignation detection)
2. Document sync issues as known bug
3. CheckmateModal only appears when AI explicitly resigns
4. Fix sync issues later

**Pros**: Game is playable, minimal risk
**Cons**: Doesn't detect checkmate in Human vs Human games
**Estimated Time**: 1 hour (documentation only)

---

## 💡 Immediate Recommendation

### Step 1: Document the Issues (Now)
- ✅ This document
- Create bug report for sync issues
- Update implementation plan

### Step 2: Choose Path Forward
Based on your priorities:
- **Need quick fix**: Option C (resign detection only)
- **Need full functionality**: Option A (fix synchronization)
- **Need middle ground**: Option B (tsshogi-only detection)

### Step 3: Test Current State
Even with issues, the game should:
- ✅ Play normally (moves work via UI)
- ✅ Show modal when AI resigns
- ❌ Not show modal randomly (fixed!)
- ⚠️ May have WASM crashes (tolerable for now)

---

## 📝 Technical Details for Fixing Sync Issues

### Where to Look

1. **Engine Synchronization**: `src/usi/controller.ts:112-126`
   ```typescript
   private async synchronizeAllEngines(currentSfen: string, moves: string[])
   ```

2. **Position Setting**: `src/usi/controller.ts:423`
   ```typescript
   engine.setPosition(currentSfen, moves);
   ```

3. **WASM Handler**: `src/lib.rs:453-` 
   ```rust
   pub fn handle_position(&mut self, parts: &[&str])
   ```

### What to Verify

- [ ] Are moves in correct USI format?
- [ ] Is SFEN being parsed correctly by WASM?
- [ ] Are engines receiving position updates?
- [ ] Is the move history being preserved?
- [ ] Are there race conditions in async synchronization?

### Debug Logging Needed

Add to `engine.setPosition()`:
```typescript
console.log('[ENGINE] setPosition called:', { sfen, moves });
```

Add to WASM `handle_position()`:
```rust
debug_log(&format!("WASM received position: sfen={}, moves={:?}", sfen, moves));
```

---

## 🎯 Success Criteria (When Fixed)

### Minimum Viable
- [ ] No false positive checkmate modals
- [ ] Modal appears when AI is actually checkmated
- [ ] No WASM crashes during normal play
- [ ] Moves apply correctly

### Full Success
- [ ] Checkmate detected in all game modes (H/H, H/AI, AI/AI)
- [ ] No synchronization issues
- [ ] No WASM memory errors
- [ ] Repetition detection works
- [ ] Performance is acceptable

---

## 📚 Related Documentation

- [Endgame Conditions Reference](docs/SHOGI_ENDGAME_CONDITIONS.md)
- [Implementation Plan](docs/design/implementation/endgame-detection/ENDGAME_DETECTION_IMPLEMENTATION_PLAN.md)
- [Task List](docs/design/implementation/endgame-detection/ENDGAME_DETECTION_TASKS.md)
- [Bug Fix Guide](docs/development/bug-fixes/BUG_FIX_INFINITE_SEARCH_LOOP.md)

---

## 🤔 Questions to Answer

1. **Is the synchronization issue recent or long-standing?**
   - Test: Does the game work correctly without AI?
   - Test: Do Human vs Human games work?

2. **Which state is correct - WASM or tsshogi?**
   - The UI shows tsshogi's state
   - If UI looks correct, tsshogi is right
   - If UI shows wrong position, both are wrong

3. **Can we trust the WASM engine at all?**
   - If memory corruption is widespread, we may need to rebuild it
   - Or switch to tsshogi-only solution

---

**Next Steps**: Choose an option (A, B, or C) and proceed accordingly.

**Current Workaround**: Game is playable, checkmate modal only appears on explicit resignation (reliable), no false positives.

