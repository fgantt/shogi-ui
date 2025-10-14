# AI Move Fix - Tauri Mode

## Problem

After the WASM removal, AI players were not responding to user moves in Tauri mode. The game would hang after the user made a move, waiting indefinitely for the AI response.

## Root Cause

The `ShogiController` had automatic engine move request logic built in:
- When `handleUserMove()` was called, it would automatically call `requestEngineMove()` if the next player was AI
- However, `requestEngineMove()` was deprecated and did nothing (just logged a warning)
- This prevented the `GamePage`'s `useEffect` hook from triggering, which was supposed to handle AI moves in Tauri mode

## Solution

### 1. Added Auto-Move Disable Flag

Added a `disableAutoEngineMove` flag to `ShogiController`:

```typescript
private disableAutoEngineMove = false; // Flag to disable automatic engine move requests (for Tauri mode)
```

### 2. Public Method to Control Flag

```typescript
public setDisableAutoEngineMove(disable: boolean): void {
  this.disableAutoEngineMove = disable;
  console.log(`[${this.instanceId}] Auto engine moves ${disable ? 'disabled' : 'enabled'}`);
}
```

### 3. Updated Controller Logic

Modified places where `requestEngineMove()` was automatically called:

**In `handleUserMove()`:**
```typescript
// Only request AI move if auto engine moves are enabled (not using Tauri engines)
if (!this.gameOver && this.isCurrentPlayerAI() && !this.disableAutoEngineMove) {
  this.requestEngineMove();
}
```

**In `newGame()`:**
```typescript
// Check if the first player is AI and request move (only if auto engine moves enabled - not in Tauri mode)
if (this.isCurrentPlayerAI() && !this.disableAutoEngineMove) {
  this.requestEngineMove();
}
```

### 4. GamePage Activates Flag

In `GamePage.tsx`, when using Tauri engines:

```typescript
// Disable automatic engine move requests in controller (Tauri handles it externally)
controller.setDisableAutoEngineMove(true);

await initializeTauriEngines(updatedSettings);
```

## Architecture

### Tauri Mode (Current)
```
User makes move
    ↓
controller.handleUserMove(move)
    ↓
controller.applyMove(move) 
    ↓
controller.emitStateChanged()
    ↓
GamePage useEffect (position changed)
    ↓
Check if AI's turn
    ↓
requestTauriEngineMove(engineId)
    ↓
Send USI commands to Tauri engine
    ↓
Listen for bestmove event
    ↓
controller.handleUserMove(aiMove)
```

### Non-Tauri Mode (Deprecated, kept for compatibility)
```
User makes move
    ↓
controller.handleUserMove(move)
    ↓
controller.applyMove(move)
    ↓
controller.emitStateChanged()
    ↓
controller.requestEngineMove() [if enabled]
    ↓
[Would communicate with WASM worker - now does nothing]
```

## Result

✅ AI now responds correctly to user moves in Tauri mode  
✅ No conflicts between controller auto-moves and GamePage engine management  
✅ Clean separation: Controller handles game state, GamePage handles Tauri engine communication  
✅ No deprecated methods actively interfering with game flow  

## Testing

To verify the fix works:

1. Start a new game with Player 1 = Human, Player 2 = AI
2. Make a move as Player 1
3. AI should respond within ~1 second
4. Check browser console - should see:
   - `Auto engine moves disabled` (when game starts)
   - USI commands being sent to engine
   - `bestmove` response from engine
   - AI move being applied

## Files Modified

- `src/usi/controller.ts` - Added disable flag and conditional logic
- `src/components/GamePage.tsx` - Activate disable flag for Tauri mode

