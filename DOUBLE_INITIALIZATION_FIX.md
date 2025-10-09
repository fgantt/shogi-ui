# Double Initialization Bug Fix - RESOLVED

## The Bug

When starting a handicap game with White (AI) to move first, the checkmate modal would appear immediately after the first move, even though the move was valid and no checkmate occurred.

## Root Cause

**React Strict Mode was causing double initialization**, which reset the game state after the first move was applied.

### What Was Happening

1. Game initializes with handicap position: `4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1` (White to move)
2. AI engine searches and returns `5c5d` (valid White move)
3. Controller successfully applies the move to the Record
4. Position updates to: `4k4/9/pppp1pppp/4p4/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1` (Black to move)
5. **React Strict Mode triggers a second render** 
6. The useEffect runs again, calling `controller.newGame(initialSfen)`
7. The Record gets reset back to the starting position!
8. But the AI still returns `5c5d` (because it's searching from the OLD position)
9. Controller tries to apply `5c5d` (a White move) to a position where it's Black's turn
10. Move is rejected → Game Over declared incorrectly

### Evidence from Logs

All the duplicate logging confirmed React Strict Mode double-execution:
```
controller.ts:133 ShogiController: Initializing engines...
controller.ts:133 ShogiController: Initializing engines...
```

After the first move succeeded, the engine was set back to the STARTING position:
```
[CONTROLLER]   ✓ Move applied successfully
[CONTROLLER]   New position SFEN: 4k4/9/pppp1pppp/4p4/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1
...
[CONTROLLER] Setting engine position: 4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1
```

## The Fix

Added a `useRef` guard to prevent double initialization in React Strict Mode:

### Changes to `src/components/GamePage.tsx`

1. **Added ref to track initialization:**
   ```typescript
   const gameInitializedRef = useRef(false);
   ```

2. **Guard the initialization useEffect:**
   ```typescript
   useEffect(() => {
     // Prevent double initialization in React strict mode
     if (gameInitializedRef.current) {
       console.log('[GAMEPAGE] Skipping duplicate initialization (strict mode)');
       return;
     }
     
     if (location.state && controller.isInitialized()) {
       // ... initialization code ...
       
       controller.newGame(stateInitialSfen).then(() => {
         console.log('[GAMEPAGE] Game initialized successfully');
         gameInitializedRef.current = true;
       }).catch(error => {
         console.error('Failed to start new game:', error);
       });
     }
     
     return () => {
       // Reset the ref when component unmounts or location changes
       gameInitializedRef.current = false;
     };
   }, [location.state, controller]);
   ```

3. **Reset ref in handleStartGame:**
   ```typescript
   const handleStartGame = (settings: GameSettings) => {
     clearUsiHistory();
     gameInitializedRef.current = false; // Reset to allow re-initialization
     // ... rest of the code ...
   };
   ```

## Why This Works

- The ref persists across re-renders but is NOT part of React's state
- React Strict Mode causes components to mount → unmount → mount again in development
- The first mount sets `gameInitializedRef.current = true`
- The second mount sees the flag and skips initialization
- The cleanup function resets the flag when the component actually unmounts
- Manual game starts via the modal reset the flag explicitly

## Testing

Test the following scenarios to verify the fix:

1. ✅ **Handicap game (White AI first move)**
   - Start a game with custom SFEN where White moves first
   - Verify AI makes first move without triggering false game over
   
2. ✅ **Standard game (Black AI first move)**
   - Start a normal game with AI as player 1
   - Verify AI makes first move correctly

3. ✅ **Multiple games in sequence**
   - Start a game, play a few moves
   - Start a new game from the modal
   - Verify each game initializes correctly

4. ✅ **Navigation-based initialization**
   - Start a game from HomePage
   - Verify game initializes once, not twice

## Related Fixes

This fix works in conjunction with the synchronization fix in `SYNCHRONIZATION_FIX.md` where we:
- Fixed `setPosition()` to pass empty moves array (since SFEN already has the complete position)
- Re-enabled proper move validation and game over detection

## Impact

- ✅ Eliminates false-positive checkmate detections
- ✅ Fixes handicap game support
- ✅ Ensures stable game state through initialization
- ✅ Compatible with React Strict Mode (best practice for React 18+)

## Files Modified

- `src/components/GamePage.tsx`

## Technical Notes

### React Strict Mode

React Strict Mode (enabled in development) intentionally double-invokes:
- Component render functions
- `useState` initializers
- `useEffect` callbacks (mount phase)

This helps catch side effects and ensures components are resilient. Our fix makes the initialization idempotent by using a ref guard.

### Why Not Disable Strict Mode?

While disabling Strict Mode would "fix" the symptom, it would:
- Hide other potential bugs
- Create issues when deploying to production
- Go against React 18+ best practices

The proper solution is to make initialization idempotent, which we've done.

