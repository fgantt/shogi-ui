# Bug Fix: AI Infinite Search Loop When Checkmated

## Bug Report Summary

**Reported**: October 8, 2025  
**Severity**: CRITICAL  
**Status**: Documented, Ready for Fix

### Symptom
When a human player checkmates the AI, the AI appears to search endlessly for a move instead of recognizing the checkmate condition and ending the game.

### Expected Behavior
1. AI recognizes it has no legal moves (checkmated)
2. Game over is detected
3. CheckmateModal appears showing the human player won
4. Game stops and no further moves can be made

### Actual Behavior
1. AI searches continuously or loops
2. No game over detection
3. Modal never appears
4. Console may show repeated search attempts

---

## Root Cause Analysis

### Primary Cause
**File**: `src/components/GamePage.tsx:450-455`

Checkmate detection code is commented out with a TODO:

```typescript
//TODO(feg): With the switch to tsshogi, need to determine checkmate and repetition from the newPosition object.
// if (newPosition.isCheckmate()) {
//   setWinner(newPosition.turn === 0 ? 'player2' : 'player1');
// } else if (newPosition.isRepetition()) {
//   setWinner('draw');
// }
```

### Contributing Factors

1. **No Controller-Level Detection**: `src/usi/controller.ts` doesn't check for game over
2. **Unhandled AI Resignation**: When search engine returns `null` or `"resign"`, it's not properly handled
3. **Missing Event Flow**: No `gameOver` event emitted from controller to UI

### System Flow (Current - Broken)

```
User makes move → AI in checkmate
↓
Search engine generates legal moves
↓
No legal moves found → returns None or "resign"
↓
Controller receives None/resign → ???
↓
UI never updated → No modal appears
↓
(If AI search continues to run → Infinite loop)
```

### System Flow (Expected - Fixed)

```
User makes move → AI in checkmate
↓
Search engine generates legal moves
↓
No legal moves found → returns None or "resign"
↓
Controller detects game over → emits gameOver event
↓
UI receives event → sets winner state
↓
CheckmateModal appears → Game stops
```

---

## The Fix (Simplified)

### Step 1: Detect Game Over in UI

Add this function to `GamePage.tsx`:

```typescript
function checkGameOver(position: ImmutablePosition): 'player1' | 'player2' | 'draw' | null {
  const isBlackTurn = position.sfen.includes(' b ');
  
  // Check if current player has any legal moves
  let hasLegalMoves = false;
  
  for (let row = 0; row < 9; row++) {
    for (let col = 0; col < 9; col++) {
      const square = Square.newByXY(col, row);
      if (!square) continue;
      
      const piece = position.board.at(square);
      if (!piece) continue;
      
      // Check if piece belongs to current player
      const belongsToCurrentPlayer = 
        (isBlackTurn && piece.color === Color.BLACK) ||
        (!isBlackTurn && piece.color === Color.WHITE);
      
      if (belongsToCurrentPlayer) {
        const moves = position.getLegalMovesFrom(square);
        if (moves.length > 0) {
          hasLegalMoves = true;
          break;
        }
      }
    }
    if (hasLegalMoves) break;
  }
  
  if (!hasLegalMoves) {
    // Current player has no legal moves - they lose
    return isBlackTurn ? 'player2' : 'player1';
  }
  
  return null;
}
```

### Step 2: Call It After Each Move

In the `onStateChanged` callback (around line 389), add:

```typescript
const onStateChanged = (newPosition: ImmutablePosition) => {
  setPosition(newPosition);
  setRenderKey(prev => prev + 1);
  
  const lastMoveData = controller.getLastMove();
  setLastMove(lastMoveData);
  
  // CHECK FOR GAME OVER - ADD THIS
  const gameOverResult = checkGameOver(newPosition);
  if (gameOverResult) {
    console.log('Game over detected:', gameOverResult);
    setWinner(gameOverResult);
  }
  
  // ... rest of existing code
};
```

### Step 3: Handle AI Resignation

In `controller.ts` (lines 41-58), update:

```typescript
engine.on('bestmove', ({ move: usiMove, sessionId: bestmoveSessionId }) => {
  // ADD THIS CHECK AT THE TOP
  if (usiMove === 'resign' || !usiMove) {
    console.log('AI resigned or has no legal moves');
    const isBlackTurn = this.record.position.sfen.includes(' b ');
    const winner = isBlackTurn ? 'player2' : 'player1';
    this.emit('gameOver', { winner, position: this.record.position });
    this.emitStateChanged();
    return;
  }
  
  // ... rest of existing code
});
```

### Step 4: Listen for GameOver Event

Add this `useEffect` in `GamePage.tsx` (around line 240):

```typescript
useEffect(() => {
  const handleGameOver = (data: { winner: 'player1' | 'player2' | 'draw' }) => {
    console.log('Game over event received:', data.winner);
    setWinner(data.winner);
  };

  controller.on('gameOver', handleGameOver);

  return () => {
    controller.off('gameOver', handleGameOver);
  };
}, [controller]);
```

---

## Testing the Fix

### Test Case 1: Human Checkmates AI

1. Start game: Human (Black) vs AI (White)
2. Set up a position where you can checkmate the AI
3. Deliver checkmate
4. **Expected**:
   - Console log: "Game over detected: player1"
   - CheckmateModal appears
   - Message: "Player 1 wins by checkmate!"
   - No infinite search loop

### Test Case 2: AI Checkmates Human

1. Start game: Human (Black) vs AI (White)
2. Allow AI to checkmate you
3. **Expected**:
   - Console log: "Game over detected: player2"
   - CheckmateModal appears
   - Message: "Player 2 wins by checkmate!"

### Test Case 3: Human vs Human

1. Start game: Human vs Human
2. Play to checkmate
3. **Expected**:
   - CheckmateModal appears with correct winner

### Verification Checklist

After implementing the fix, verify:

- [ ] No infinite search loop when AI is checkmated
- [ ] CheckmateModal appears immediately
- [ ] Correct winner is displayed
- [ ] "New Game" button works
- [ ] "Review Position" button works
- [ ] Console shows appropriate logging
- [ ] Game stops accepting moves after checkmate

---

## Debug Logging

Add these console logs to help diagnose issues:

### In checkGameOver():
```typescript
console.log('Checking game over for position:', position.sfen);
console.log('Current turn:', isBlackTurn ? 'Black' : 'White');
console.log('Has legal moves:', hasLegalMoves);
console.log('Game over result:', result);
```

### In onStateChanged():
```typescript
console.log('State changed, new position:', newPosition.sfen);
console.log('Checking for game over...');
const gameOverResult = checkGameOver(newPosition);
console.log('Game over result:', gameOverResult);
```

### In controller bestmove handler:
```typescript
console.log('Received bestmove:', usiMove);
if (usiMove === 'resign' || !usiMove) {
  console.log('AI has no moves, declaring game over');
}
```

---

## Known Edge Cases

### 1. Stalemate (No legal moves, not in check)
- In shogi, this is a **loss** for the player with no moves (not a draw like chess)
- The fix handles this correctly (returns winner)

### 2. AI Search Timeout
- If AI times out, it may return a partial result
- Current fix should handle this

### 3. Position Loaded from SFEN
- If loading a game in a checkmate position
- Fix should detect it immediately on load

---

## Related Files

### Files to Modify
- `src/components/GamePage.tsx` - Primary fix location
- `src/usi/controller.ts` - AI resignation handling

### Files to Reference
- `src/components/CheckmateModal.tsx` - Already exists, no changes needed
- `src/bitboards.rs` - Has checkmate detection (for reference)
- `src/search/search_integration.rs` - Search engine behavior (for reference)

### Documentation
- [Full Implementation Plan](../design/implementation/ENDGAME_DETECTION_IMPLEMENTATION_PLAN.md)
- [Implementation Tasks](../design/implementation/ENDGAME_DETECTION_TASKS.md)
- [Shogi Endgame Conditions](../SHOGI_ENDGAME_CONDITIONS.md)

---

## Estimated Time to Fix

- **Investigation**: 30 minutes
- **Implementation**: 2-3 hours
- **Testing**: 1-2 hours
- **Total**: 4-6 hours

---

## Next Steps

1. Review tsshogi API documentation for any built-in methods
2. Implement `checkGameOver()` function
3. Wire it to state updates
4. Test with known checkmate positions
5. Handle AI resignation case
6. Comprehensive testing in all game modes

---

## Success Criteria

- ✅ No infinite search loop when AI is checkmated
- ✅ CheckmateModal appears within 1 second of checkmate
- ✅ Correct winner displayed in modal
- ✅ Game stops accepting moves after checkmate
- ✅ Works in all game modes (H/H, H/AI, AI/AI)
- ✅ Console logging confirms detection

---

**Priority**: CRITICAL - Fix Immediately  
**Complexity**: Medium  
**Risk**: Low (isolated changes to game over detection)

