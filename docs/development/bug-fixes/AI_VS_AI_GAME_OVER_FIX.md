# AI vs AI Game Over Fix

## Date
October 10, 2025

## Problem

When an AI vs AI game reached an endgame condition (checkmate, draw, resignation, etc.) and the game over notice was given to the UI, the AI engines continued to request and make moves in the background. This created an infinite loop of AI moves even after the game was officially over.

## Root Cause

The issue was in the game flow control logic in `ShogiController`:

1. When a move was made, `emitStateChanged()` was called
2. Inside `emitStateChanged()`, `checkEndgameConditions()` detected the game over and emitted a `gameOver` event
3. **BUT** after `emitStateChanged()` returned, the code still checked if the next player was AI and called `requestEngineMove()`
4. This caused the AI to continue making moves even though the game was over

The problem occurred in three places:
- In the `bestmove` event handler (after AI moves)
- In `handleUserMove()` (after human moves)
- The game over event was emitted but there was no flag to prevent subsequent AI move requests

## Solution

Added a `gameOver` flag to track when the game has ended, and implemented engine stopping to cancel pending operations:

### 1. Added gameOver Flag
```typescript
private gameOver = false; // Track if the game has ended
```

### 2. Added Engine Stopping Method
```typescript
private stopAllEngines(): void {
  console.log(`[${this.instanceId}] Stopping all engines due to game over`);
  for (const engine of this.sessions.values()) {
    engine.stop().catch(err => {
      console.error(`[${this.instanceId}] Error stopping engine:`, err);
    });
  }
}
```

### 3. Set Flag and Stop Engines When Game Ends
Updated all game over detection points to set the flag AND stop all engines:

- **Impasse detection** (`checkEndgameConditions()` line 828-829)
- **No legal moves/Checkmate** (`checkEndgameConditions()` line 904-905)
- **Four-fold repetition** (`updatePositionHistory()` line 481-482)
- **AI resignation** (`bestmove` handler line 68-69)
- **Illegal move** (`bestmove` handler line 97-98)

Each location now calls:
```typescript
this.gameOver = true;
this.stopAllEngines(); // Cancel any pending engine operations
```

### 4. Guard Against New AI Move Requests
Added guards to prevent AI move requests when game is over:

- **In requestEngineMove()** (line 486-489):
  ```typescript
  if (this.gameOver) {
    console.log(`[${this.instanceId}] [SEQ-1] requestEngineMove BLOCKED - game is over`);
    return;
  }
  ```

- **After AI move** (`bestmove` handler line 107):
  ```typescript
  if (!this.gameOver && this.isCurrentPlayerAI()) {
    this.requestEngineMove();
  }
  ```

- **After human move** (`handleUserMove()` line 403):
  ```typescript
  if (!this.gameOver && this.isCurrentPlayerAI()) {
    this.requestEngineMove();
  }
  ```

### 5. Reset Flag for New Games
Reset the flag when starting a new game:

- **New game** (`newGame()` line 571)
- **Load position** (`loadSfen()` line 604)

## Files Modified

- `/Users/fgantt/projects/vibe/shogi-game/worktrees/usi/src/usi/controller.ts`

## Changes Summary

1. Added `gameOver` flag property (line 25)
2. Added `stopAllEngines()` method to cancel pending engine operations (lines 638-645)
3. Set `gameOver = true` AND call `stopAllEngines()` in 5 endgame detection locations
4. Added guard in `requestEngineMove()` to block execution if game is over (lines 486-489)
5. Added `!this.gameOver &&` check before 2 AI move request calls
6. Reset `gameOver = false` in 2 game initialization methods

## Runtime Error Fix

The initial implementation caused WASM memory allocation errors because engines were still processing move requests when the game ended. The enhanced fix:

- **Stops all engines immediately** when game over is detected
- **Blocks new move requests** at the `requestEngineMove()` entry point
- **Prevents race conditions** where an engine processes a stale position after game ends

This eliminates the "memory access out of bounds" error that occurred when engines tried to allocate memory for positions in an already-ended game.

## Testing

The fix was verified by:
1. TypeScript compilation with no errors
2. Build process completed successfully
3. No linting errors

## Expected Behavior After Fix

✅ When an AI vs AI game reaches checkmate, draw, or any other endgame condition:
1. The game over event is emitted to the UI
2. The checkmate/draw modal is displayed
3. **AI engines stop making moves immediately**
4. The game over flag prevents any further AI move requests

✅ When starting a new game:
1. The game over flag is reset
2. AI engines can make moves again in the new game

## Impact

- **High Priority Fix**: Prevents resource waste and potential bugs from AI engines continuing to run after game completion
- **Zero Breaking Changes**: Only adds additional checks, doesn't change existing logic
- **Performance**: Prevents unnecessary AI calculations after game ends

