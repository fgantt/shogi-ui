# Endgame Detection - Implementation Tasks

## Quick Reference

**Document**: [Full Implementation Plan](./ENDGAME_DETECTION_IMPLEMENTATION_PLAN.md)  
**Related**: [Shogi Endgame Conditions](../../SHOGI_ENDGAME_CONDITIONS.md)  
**Status**: Planning Complete, Ready for Implementation

---

## 🔴 CRITICAL - Sprint 1 (Must Fix Immediately)

These tasks fix the reported bug where the AI searches endlessly when checkmated.

### Task 1.1: Investigate tsshogi API
**File**: Research
**Priority**: CRITICAL
**Estimated Time**: 30 minutes

- [ ] Check if `ImmutablePosition` has `isCheckmate()` method
- [ ] Check if `ImmutablePosition` has methods to detect no legal moves
- [ ] Document available methods for endgame detection
- [ ] Determine if we need to implement detection manually

### Task 1.2: Implement Game Over Detection Function
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL
**Estimated Time**: 2-3 hours

- [ ] Create `checkGameOver(position: ImmutablePosition)` function
- [ ] Detect when current player has no legal moves
- [ ] Check all squares for pieces belonging to current player
- [ ] Check all legal moves from each piece
- [ ] Check for legal drop moves
- [ ] Return appropriate winner ('player1', 'player2', 'draw', or null)
- [ ] Add console logging for debugging

**Code Location**: After line 455 in `GamePage.tsx`

### Task 1.3: Wire Game Over Detection to State Updates
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL
**Estimated Time**: 1 hour

- [ ] Call `checkGameOver()` in the `onStateChanged` handler (around line 389)
- [ ] Set winner state when game over is detected
- [ ] Add logging to verify detection is working
- [ ] Test with known checkmate positions

**Code Location**: In `onStateChanged` callback around line 389-456

### Task 1.4: Handle AI Resignation
**File**: `src/usi/controller.ts`
**Priority**: CRITICAL
**Estimated Time**: 1 hour

- [ ] Update bestmove handler to detect `"resign"` or empty move
- [ ] Emit `gameOver` event when AI resigns
- [ ] Calculate winner based on current turn
- [ ] Test with positions where AI should resign

**Code Location**: Lines 41-58 in `controller.ts`

### Task 1.5: Add GameOver Event Listener in UI
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL  
**Estimated Time**: 30 minutes

- [ ] Add `useEffect` to listen for `gameOver` event from controller
- [ ] Update winner state when event fires
- [ ] Clean up event listener on unmount
- [ ] Test event propagation

**Code Location**: After other controller event listeners (around line 230-240)

### Task 1.6: Verify CheckmateModal Display
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL
**Estimated Time**: 30 minutes

- [ ] Verify modal is rendered when `winner` state is set
- [ ] Check modal is visible (z-index, display properties)
- [ ] Test "New Game" button functionality
- [ ] Test "Review Position" button functionality

**Code Location**: Search for `<CheckmateModal` in GamePage.tsx

### Task 1.7: Testing - Human vs AI Checkmate
**Priority**: CRITICAL
**Estimated Time**: 1 hour

- [ ] Start game: Human (Black) vs AI (White)
- [ ] Play until AI is checkmated
- [ ] Verify no infinite search loop
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner
- [ ] Test "New Game" functionality
- [ ] Test "Review Position" functionality

### Task 1.8: Testing - AI vs Human Checkmate
**Priority**: CRITICAL
**Estimated Time**: 30 minutes

- [ ] Start game: AI (Black) vs Human (White)
- [ ] Play until Human is checkmated
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner

### Task 1.9: Testing - Human vs Human Checkmate
**Priority**: CRITICAL
**Estimated Time**: 30 minutes

- [ ] Start game: Human vs Human
- [ ] Play until one player is checkmated
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner

---

## 🟠 HIGH PRIORITY - Sprint 2 (Repetition & Stalemate)

### Task 2.1: Add Position History Tracking
**File**: `src/usi/controller.ts`
**Priority**: HIGH
**Estimated Time**: 2 hours

- [ ] Add `positionHistory: Map<string, number>` field
- [ ] Create `updatePositionHistory()` method
- [ ] Call it after each move in `applyMove()`
- [ ] Clear history in `newGame()`
- [ ] Add logging for position counts

### Task 2.2: Implement Repetition Detection
**File**: `src/usi/controller.ts`
**Priority**: HIGH
**Estimated Time**: 2 hours

- [ ] Check if position count >= 4 (Sennichite)
- [ ] Emit `gameOver` with `winner: 'draw'`
- [ ] Add console logging when repetition detected
- [ ] Test with known repetitive sequences

### Task 2.3: Update CheckmateModal for Repetition
**File**: `src/components/CheckmateModal.tsx`
**Priority**: HIGH
**Estimated Time**: 30 minutes

- [ ] Verify draw message displays correctly
- [ ] Update message text if needed
- [ ] Test modal appearance for draw

### Task 2.4: Implement Stalemate Detection
**File**: `src/components/GamePage.tsx`
**Priority**: HIGH
**Estimated Time**: 1 hour

- [ ] In `checkGameOver()`, detect no legal moves + not in check
- [ ] In shogi, stalemate = loss for player who can't move
- [ ] Return appropriate winner
- [ ] Test with stalemate positions

### Task 2.5: Testing - Repetition Detection
**Priority**: HIGH
**Estimated Time**: 1 hour

- [ ] Create test position that repeats
- [ ] Play moves to create 4-fold repetition
- [ ] Verify draw is detected
- [ ] Verify modal shows "Draw by repetition"

---

## 🟡 MEDIUM PRIORITY - Sprint 3 (Impasse & Illegal Moves)

### Task 3.1: Implement Impasse Condition Check (Rust)
**File**: `src/bitboards.rs`
**Priority**: MEDIUM
**Estimated Time**: 3 hours

- [ ] Add `is_impasse_condition()` method
- [ ] Check if both kings in opponent's promotion zones
- [ ] Add `count_impasse_points()` method
- [ ] Implement 24-point counting rule
- [ ] Add `check_impasse_result()` method
- [ ] Add unit tests

### Task 3.2: Expose Impasse Detection to WASM
**File**: `src/lib.rs` or WASM bindings
**Priority**: MEDIUM
**Estimated Time**: 2 hours

- [ ] Add WASM binding for impasse check
- [ ] Return impasse result (draw, black wins, white wins)
- [ ] Add TypeScript type definitions
- [ ] Test WASM function calls

### Task 3.3: Integrate Impasse Detection in Controller
**File**: `src/usi/controller.ts`
**Priority**: MEDIUM
**Estimated Time**: 1 hour

- [ ] Call impasse check when both kings advanced
- [ ] Emit gameOver with appropriate winner
- [ ] Add logging for impasse detection

### Task 3.4: Enhanced Illegal Move Validation
**File**: `src/moves.rs` and `src/bitboards.rs`
**Priority**: MEDIUM
**Estimated Time**: 6-8 hours

- [ ] Implement Nifu (double pawn) detection
- [ ] Implement Uchifuzume (pawn drop mate) detection
- [ ] Implement mandatory promotion enforcement
- [ ] Add specific error messages for each violation
- [ ] Add unit tests for each illegal move type

---

## 🟢 LOW PRIORITY - Sprint 4 (Polish & Enhancement)

### Task 4.1: Enhanced CheckmateModal Component
**File**: `src/components/CheckmateModal.tsx`
**Priority**: LOW
**Estimated Time**: 2 hours

- [ ] Add `endgameType` prop
- [ ] Add `details` prop for additional information
- [ ] Display different messages for each endgame type
- [ ] Update styling if needed

### Task 4.2: Game Over Sound Effects
**File**: `src/utils/audio.ts` and relevant components
**Priority**: LOW
**Estimated Time**: 1-2 hours

- [ ] Create or find checkmate sound effect
- [ ] Create or find draw sound effect
- [ ] Add `playCheckmateSound()` function
- [ ] Call sound when game over detected
- [ ] Respect sound settings

### Task 4.3: Game Over Animation
**File**: `src/components/GamePage.css` and component
**Priority**: LOW
**Estimated Time**: 2-3 hours

- [ ] Add animation for CheckmateModal entrance
- [ ] Add board overlay effect when game over
- [ ] Highlight winning pieces/king
- [ ] Test animations on different browsers

### Task 4.4: Comprehensive Integration Tests
**Priority**: LOW
**Estimated Time**: 4-6 hours

- [ ] Write automated tests for checkmate detection
- [ ] Write automated tests for repetition
- [ ] Write automated tests for impasse
- [ ] Write automated tests for illegal moves
- [ ] Set up CI/CD to run tests

---

## Implementation Order Recommendation

1. **Start Here** (Fix the reported bug):
   - Task 1.1 → 1.2 → 1.3 → 1.4 → 1.5 → 1.6
   - Testing: 1.7 → 1.8 → 1.9

2. **After Critical Fix Works**:
   - Task 2.1 → 2.2 → 2.3 → 2.4 → 2.5

3. **If Time Permits**:
   - Tasks 3.x (Impasse and illegal moves)
   - Tasks 4.x (Polish)

---

## Testing Checklist

After implementing critical tasks, verify:

- [ ] Human vs AI: AI gets checkmated → Modal appears ✓
- [ ] Human vs AI: Human gets checkmated → Modal appears ✓
- [ ] Human vs Human: Either player checkmated → Modal appears ✓
- [ ] AI vs AI: Game ends properly → Modal appears ✓
- [ ] AI doesn't search infinitely when checkmated ✓
- [ ] "New Game" button works after game over ✓
- [ ] "Review Position" button works after game over ✓
- [ ] Console shows appropriate logging ✓

## Quick Start Guide

### To Fix the Immediate Bug:

1. **First**, investigate tsshogi API (Task 1.1)
2. **Then**, implement `checkGameOver()` function (Task 1.2)
3. **Next**, wire it to state updates (Task 1.3)
4. **Test** with a known checkmate position
5. **Handle** AI resignation (Task 1.4)
6. **Verify** modal displays properly (Task 1.6)
7. **Test** all game modes (Tasks 1.7-1.9)

### Files to Modify (Critical):
- `src/components/GamePage.tsx` (main changes)
- `src/usi/controller.ts` (AI resignation handling)

### Files to Read (for understanding):
- `src/components/CheckmateModal.tsx` (already exists)
- `src/bitboards.rs` (to understand checkmate detection)
- `docs/SHOGI_ENDGAME_CONDITIONS.md` (rules reference)

---

**Next Step**: Begin with Task 1.1 to investigate tsshogi API capabilities

