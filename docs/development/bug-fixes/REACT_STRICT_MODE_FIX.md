# React Strict Mode Double Initialization Fix

## Date
October 9, 2025

## Problem

The game was experiencing a critical bug where:
1. AI would make a valid first move successfully
2. Immediately after, the AI would try to make the same move again
3. The second move would fail because it was the wrong player's turn
4. Checkmate modal would appear incorrectly

### Root Cause

React Strict Mode mounts components twice in development mode to help detect side effects. Our `useEffect` hook in `GamePage.tsx` was setting `gameInitializedRef.current = true` AFTER the async `controller.newGame()` promise resolved:

```typescript
controller.newGame(stateInitialSfen).then(() => {
  gameInitializedRef.current = true;  // ❌ TOO LATE!
});
```

**Execution flow with React Strict Mode:**
1. **First mount**: `gameInitializedRef.current` is `false`, starts `controller.newGame()` (async)
2. **Cleanup runs**: Sets `gameInitializedRef.current` to `false`
3. **Second mount**: `gameInitializedRef.current` is still `false`, starts ANOTHER `controller.newGame()` (async)
4. **First promise resolves**: Sets `gameInitializedRef.current` to `true`
5. **Second promise resolves**: Sets `gameInitializedRef.current` to `true`

Result: **TWO games are initialized**, both with the same starting position. Both AI engines start searching simultaneously, causing race conditions and incorrect state.

### Evidence from Logs

The logs clearly showed double initialization:

```
GamePage.tsx:262 Initializing game from navigation state: {...}
GamePage.tsx:374 [GAMEPAGE] Cleaning up gameOver event listener  ← cleanup from first mount
GamePage.tsx:262 Initializing game from navigation state: {...}  ← SECOND initialization!
```

And double engine searches:

```
[ENGINE gote] SFEN: 4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1
bestmove 5c5d  ← First search
...
[ENGINE gote] SFEN: 4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1
bestmove 5c5d  ← Second search with OLD position!
```

## Solution

Set `gameInitializedRef.current = true` BEFORE the async call, not after:

```typescript
// CRITICAL: Set the ref BEFORE the async call to prevent React Strict Mode double-initialization
gameInitializedRef.current = true;
controller.newGame(stateInitialSfen).then(() => {
  console.log('[GAMEPAGE] Game initialized successfully');
}).catch(error => {
  console.error('Failed to start new game:', error);
  // Reset on error to allow retry
  gameInitializedRef.current = false;
});
```

**New execution flow:**
1. **First mount**: `gameInitializedRef.current` is `false`, sets it to `true` immediately, starts `controller.newGame()` (async)
2. **Cleanup runs**: Sets `gameInitializedRef.current` to `false`
3. **Second mount**: `gameInitializedRef.current` is NOW `true`, **SKIPS initialization** ✅

## Files Modified

- `src/components/GamePage.tsx` (lines 309-318)

## Testing

To verify the fix:
1. Start a handicap game where AI moves first
2. Verify only ONE "Initializing game from navigation state" log appears
3. Verify the AI makes its first move successfully
4. Verify turn passes to the human player
5. Verify NO premature checkmate modal

## Key Learnings

1. **React Strict Mode caveat**: When using `useRef` to prevent double-initialization, the ref must be set SYNCHRONOUSLY before any async operations.

2. **Async timing**: Setting state/refs inside promise handlers (`.then()`) is too late to prevent React Strict Mode double-execution.

3. **Cleanup timing**: React Strict Mode runs the cleanup function BETWEEN the two mounts, which can reset refs if not careful.

## Related Issues

This fix completes the synchronization work started in:
- `SYNCHRONIZATION_FIX.md` - Fixed WASM engine double-applying moves
- `DOUBLE_INITIALIZATION_FIX.md` - Initial attempt to fix React Strict Mode issues

## Impact

This fix ensures:
- ✅ Only one game is initialized per navigation
- ✅ No duplicate AI searches
- ✅ Correct turn management
- ✅ No false checkmate detection
- ✅ Proper game state consistency

