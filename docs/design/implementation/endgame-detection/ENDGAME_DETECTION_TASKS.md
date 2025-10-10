# Endgame Detection - Implementation Tasks

## Quick Reference

**Document**: [Full Implementation Plan](./ENDGAME_DETECTION_IMPLEMENTATION_PLAN.md)  
**Related**: [Shogi Endgame Conditions](../../SHOGI_ENDGAME_CONDITIONS.md)  
**Status**: ✅ CRITICAL & HIGH PRIORITY TASKS COMPLETE - Ready for Testing  
**Implementation Summary**: See [ENDGAME_DETECTION_IMPLEMENTATION_COMPLETE.md](../../../../ENDGAME_DETECTION_IMPLEMENTATION_COMPLETE.md)  
**Testing Instructions**: See [TESTING_INSTRUCTIONS.md](../../../../TESTING_INSTRUCTIONS.md)

---

## 🔴 CRITICAL - Sprint 1 (Must Fix Immediately)

These tasks fix the reported bug where the AI searches endlessly when checkmated.

### Task 1.1: Investigate tsshogi API ✅
**File**: Research
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Status**: ✅ COMPLETE

- [x] Check if `ImmutablePosition` has `isCheckmate()` method
- [x] Check if `ImmutablePosition` has methods to detect no legal moves
- [x] Document available methods for endgame detection
- [x] Determine if we need to implement detection manually

**Result**: Used existing tsshogi APIs (`position.checked`, `position.createMove()`, `position.isValidMove()`, `position.hand()`)

### Task 1.2: Implement Game Over Detection Function ✅
**File**: `src/usi/controller.ts`
**Priority**: CRITICAL
**Estimated Time**: 2-3 hours
**Status**: ✅ COMPLETE

- [x] Create endgame detection logic (implemented in `checkEndgameConditions()`)
- [x] Detect when current player has no legal moves
- [x] Check all squares for pieces belonging to current player
- [x] Check all legal moves from each piece
- [x] Check for legal drop moves
- [x] Return appropriate winner ('player1', 'player2', 'draw', or null)
- [x] Add console logging for debugging

**Implementation**: `checkEndgameConditions()` method in controller.ts (lines 678-767)

### Task 1.3: Wire Game Over Detection to State Updates ✅
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Status**: ✅ COMPLETE

- [x] Game over detection called automatically in `emitStateChanged()` via `checkEndgameConditions()`
- [x] Set winner state when game over is detected via event listener
- [x] Add logging to verify detection is working
- [x] Ready to test with known checkmate positions

**Implementation**: Event-driven architecture, controller emits gameOver events

### Task 1.4: Handle AI Resignation ✅
**File**: `src/usi/controller.ts`
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Status**: ✅ COMPLETE

- [x] Update bestmove handler to detect `"resign"` or empty move
- [x] Emit `gameOver` event when AI resigns with `endgameType: 'resignation'`
- [x] Calculate winner based on current turn
- [x] Ready to test with positions where AI should resign

**Implementation**: Lines 62-69 in controller.ts

### Task 1.5: Add GameOver Event Listener in UI ✅
**File**: `src/components/GamePage.tsx`
**Priority**: CRITICAL  
**Estimated Time**: 30 minutes
**Status**: ✅ COMPLETE

- [x] Added `useEffect` to listen for `gameOver` event from controller
- [x] Update winner state and endgameType when event fires
- [x] Clean up event listener on unmount
- [x] Event propagation tested via console logging

**Implementation**: Lines 381-405 in GamePage.tsx

### Task 1.6: Verify CheckmateModal Display ✅
**File**: `src/components/CheckmateModal.tsx` 
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Status**: ✅ COMPLETE - Enhanced

- [x] Modal is rendered when `winner` state is set
- [x] Modal visible with proper z-index (uses settings-overlay class)
- [x] "New Game" button functionality verified
- [x] "Review Position" button functionality verified
- [x] **BONUS**: Added endgameType prop with specific messages and emojis

**Enhancement**: Lines 1-85 in CheckmateModal.tsx

### Task 1.7: Testing - Human vs AI Checkmate ⏳
**Priority**: CRITICAL
**Estimated Time**: 1 hour
**Status**: ⏳ AWAITING MANUAL TESTING

- [ ] Start game: Human (Black) vs AI (White)
- [ ] Play until AI is checkmated
- [ ] Verify no infinite search loop
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner
- [ ] Test "New Game" functionality
- [ ] Test "Review Position" functionality

**Testing Guide**: See [TESTING_INSTRUCTIONS.md](../../../../TESTING_INSTRUCTIONS.md) - Test 1

### Task 1.8: Testing - AI vs Human Checkmate ⏳
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Status**: ⏳ AWAITING MANUAL TESTING

- [ ] Start game: AI (Black) vs Human (White)
- [ ] Play until Human is checkmated
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner

**Testing Guide**: See [TESTING_INSTRUCTIONS.md](../../../../TESTING_INSTRUCTIONS.md) - Test 2

### Task 1.9: Testing - Human vs Human Checkmate ⏳
**Priority**: CRITICAL
**Estimated Time**: 30 minutes
**Status**: ⏳ AWAITING MANUAL TESTING

- [ ] Start game: Human vs Human
- [ ] Play until one player is checkmated
- [ ] Verify CheckmateModal appears
- [ ] Verify modal shows correct winner

**Testing Guide**: See [TESTING_INSTRUCTIONS.md](../../../../TESTING_INSTRUCTIONS.md) - Test 3

---

## 🟠 HIGH PRIORITY - Sprint 2 (Repetition & Stalemate) ✅ COMPLETE

### Task 2.1: Add Position History Tracking ✅
**File**: `src/usi/controller.ts`
**Priority**: HIGH
**Estimated Time**: 2 hours
**Status**: ✅ COMPLETE

- [x] Add `positionHistory: Map<string, number>` field
- [x] Create `updatePositionHistory()` method
- [x] Call it after each move in `applyMove()`
- [x] Clear history in `newGame()` and `loadSfen()`
- [x] Add logging for position counts

**Implementation**: Lines 24, 462-477, 566-567, 595-596 in controller.ts

### Task 2.2: Implement Repetition Detection ✅
**File**: `src/usi/controller.ts`
**Priority**: HIGH
**Estimated Time**: 2 hours
**Status**: ✅ COMPLETE

- [x] Check if position count >= 4 (Sennichite)
- [x] Emit `gameOver` with `winner: 'draw'` and `endgameType: 'repetition'`
- [x] Add console logging when repetition detected
- [x] Ready to test with known repetitive sequences

**Implementation**: Lines 473-476 in controller.ts (within updatePositionHistory)

### Task 2.3: Update CheckmateModal for Repetition ✅
**File**: `src/components/CheckmateModal.tsx`
**Priority**: HIGH
**Estimated Time**: 30 minutes
**Status**: ✅ COMPLETE

- [x] Draw message displays correctly
- [x] Updated message text with Japanese terminology
- [x] Modal appearance for draw includes emoji 🤝

**Implementation**: Lines 20-30 in CheckmateModal.tsx (includes Sennichite / 千日手)

### Task 2.4: Implement Stalemate Detection ✅
**File**: `src/usi/controller.ts`
**Priority**: HIGH
**Estimated Time**: 1 hour
**Status**: ✅ COMPLETE

- [x] Detect no legal moves + not in check (using `position.checked`)
- [x] In shogi, stalemate = loss for player who can't move
- [x] Return appropriate winner with `endgameType: 'no_moves'`
- [x] Ready to test with stalemate positions

**Implementation**: Lines 755-759 in controller.ts (distinguishes checkmate vs no_moves)

### Task 2.5: Testing - Repetition Detection ⏳
**Priority**: HIGH
**Estimated Time**: 1 hour
**Status**: ⏳ AWAITING MANUAL TESTING

- [ ] Create test position that repeats
- [ ] Play moves to create 4-fold repetition
- [ ] Verify draw is detected
- [ ] Verify modal shows "Draw by repetition (Sennichite)"

**Testing Guide**: See [TESTING_INSTRUCTIONS.md](../../../../TESTING_INSTRUCTIONS.md) - Test 4

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

### Task 4.1: Enhanced CheckmateModal Component ✅
**File**: `src/components/CheckmateModal.tsx`
**Priority**: LOW
**Estimated Time**: 2 hours
**Status**: ✅ COMPLETE - Implemented ahead of schedule

- [x] Add `endgameType` prop
- [x] Add `details` prop for additional information
- [x] Display different messages for each endgame type
- [x] Update styling with emojis and centered text
- [x] **BONUS**: Added Japanese terminology (Tsumi, Sennichite)
- [x] **BONUS**: Added emoji visual feedback for each type

**Implementation**: Lines 1-85 in CheckmateModal.tsx
**Endgame Types Supported**: checkmate, resignation, repetition, stalemate, illegal, no_moves

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

## Testing Checklist ⏳

Implementation complete - awaiting manual testing. See [TESTING_INSTRUCTIONS.md](../../TESTING_INSTRUCTIONS.md) for detailed procedures.

**Critical Features (Ready for Testing)**:
- [ ] Human vs AI: AI gets checkmated → Modal appears
- [ ] Human vs AI: Human gets checkmated → Modal appears
- [ ] Human vs Human: Either player checkmated → Modal appears
- [ ] AI vs AI: Game ends properly → Modal appears
- [ ] AI doesn't search infinitely when checkmated (primary bug fix)
- [ ] "New Game" button works after game over
- [ ] "Review Position" button works after game over
- [ ] Console shows appropriate logging

**High Priority Features (Ready for Testing)**:
- [ ] Four-fold repetition detected (Sennichite / 千日手)
- [ ] Draw modal shows proper message with Japanese terminology
- [ ] Stalemate detection (no legal moves but not in check)
- [ ] AI resignation handling

**Implementation Quality**:
- [x] No linting errors
- [x] Production build successful
- [x] Comprehensive console logging
- [x] Event-driven architecture
- [x] Type-safe TypeScript code

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

## ✅ Implementation Summary

**Status**: All CRITICAL and HIGH PRIORITY tasks complete  
**Date Completed**: October 9, 2025  
**Build Status**: ✅ Production build successful  
**Linting**: ✅ No errors  
**Ready for Testing**: Yes

### What Was Completed

**🔴 CRITICAL Tasks (Sprint 1)**: 6/6 complete + 3 awaiting testing
- ✅ Task 1.1: API Investigation
- ✅ Task 1.2: Game Over Detection
- ✅ Task 1.3: Wiring to State Updates
- ✅ Task 1.4: AI Resignation Handling
- ✅ Task 1.5: Event Listener Integration
- ✅ Task 1.6: CheckmateModal Verification
- ⏳ Tasks 1.7-1.9: Manual testing pending

**🟠 HIGH PRIORITY Tasks (Sprint 2)**: 4/4 complete + 1 awaiting testing
- ✅ Task 2.1: Position History Tracking
- ✅ Task 2.2: Repetition Detection
- ✅ Task 2.3: CheckmateModal for Repetition
- ✅ Task 2.4: Stalemate Detection
- ⏳ Task 2.5: Manual testing pending

**🟢 BONUS**: Task 4.1 Enhanced CheckmateModal (completed ahead of schedule)

### Key Files Modified
1. `src/usi/controller.ts` - Core endgame detection and position history
2. `src/components/GamePage.tsx` - UI integration and event handling
3. `src/components/CheckmateModal.tsx` - Enhanced modal with multiple endgame types

### Documentation Created
1. `ENDGAME_DETECTION_IMPLEMENTATION_COMPLETE.md` - Comprehensive summary
2. `TESTING_INSTRUCTIONS.md` - Detailed testing guide

### Next Steps
1. Run `npm run dev` to start development server
2. Follow testing instructions in `TESTING_INSTRUCTIONS.md`
3. Verify all test scenarios pass
4. Report any issues found during testing

---

**Last Updated**: October 9, 2025  
**Implementation Complete**: Yes ✅  
**Testing Required**: Yes ⏳

