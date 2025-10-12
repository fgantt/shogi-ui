# Final Fix: React Strict Mode Double Initialization

## Date
October 9, 2025

## Problem Identified

Thanks to comprehensive logging, we identified the **exact root cause**:

### What The Logs Showed

1. ✅ **Only ONE controller** exists (`CTRL-1-xwtfmnmy1`)
2. ❌ **React component mounts TWICE** (React Strict Mode)
3. ❌ **Both mounts initialize the game** because `gameInitializedRef` was being **reset** between mounts

### The Sequence of Events

```
1. First Mount:
   [COMPONENT] GamePage MOUNTED
   gameInitializedRef.current: false
   → PROCEEDING with initialization
   → gameInitializedRef.current = true
   → controller.newGame() starts (async)
   → [SEQ-1] requestEngineMove START (first search)

2. React Strict Mode Cleanup:
   [COMPONENT] GamePage UNMOUNTING
   → return () => { gameInitializedRef.current = false }  ← BUG!

3. Second Mount:
   [COMPONENT] GamePage MOUNTED
   gameInitializedRef.current: false  ← Reset by cleanup!
   → PROCEEDING with initialization ← SHOULDN'T HAPPEN!
   → gameInitializedRef.current = true
   → controller.newGame() starts AGAIN (async)
   → [SEQ-1] requestEngineMove START (second search)

4. First Search Completes:
   [SEQ-4] bestmove: 5c5d
   Record: Turn=White ← Correct
   ✓ Move applied successfully
   Record: Turn=Black ← Correct

5. Second Search Completes:
   [SEQ-4] bestmove: 5c5d ← Same move!
   Record: Turn=Black ← Already changed!
   ✗ createMoveByUSI returns null ← Wrong turn!
   → GAME OVER modal appears
```

## Root Cause

The `useEffect` cleanup function was **resetting `gameInitializedRef.current` to `false`** between mounts:

```typescript
return () => {
  gameInitializedRef.current = false;  // ← THIS WAS THE BUG!
};
```

In React Strict Mode (development only), components mount → unmount → mount again. The cleanup runs between the two mounts, resetting the ref, so the second mount sees `false` and initializes again.

## The Fix

**Remove the ref reset from the cleanup function:**

```typescript
// BEFORE (buggy):
return () => {
  gameInitializedRef.current = false;  // Reset on unmount
};

// AFTER (fixed):
// DON'T reset the ref in cleanup - it should persist across React Strict Mode re-mounts
// Only reset when actually starting a new game via handleStartGame
```

The ref now persists across React Strict Mode re-mounts. It only gets reset when the user explicitly starts a new game via `handleStartGame()`.

## Why This Works

### React Strict Mode Behavior

In development, React Strict Mode:
1. Mounts component
2. **Immediately unmounts it** (runs cleanup)
3. **Mounts it again**

This helps catch bugs related to missing cleanup logic.

### How Refs Work

`useRef` returns the **same object** across re-renders and re-mounts. The value persists in the ref object itself, not in component state.

### The Solution

By NOT resetting the ref in cleanup:
- First mount: Sets `gameInitializedRef.current = true`
- Cleanup runs: Ref stays `true` (no reset)
- Second mount: Sees `true`, **skips initialization** ✅

## Testing

After this fix, you should see:

```
[COMPONENT-xxx] GamePage MOUNTED
[COMPONENT-xxx] gameInitializedRef.current: false
[COMPONENT-xxx] PROCEEDING with initialization
[COMPONENT-xxx] Set gameInitializedRef to true

[COMPONENT-xxx] GamePage UNMOUNTING

[COMPONENT-xxx] GamePage MOUNTED
[COMPONENT-xxx] gameInitializedRef.current: true  ← Persisted!
[COMPONENT-xxx] SKIPPING initialization - already initialized  ← Fixed!

[CTRL-1-xxx] [SEQ-1] requestEngineMove START  ← Only ONE!
[CTRL-1-xxx] [SEQ-4] bestmove received
[CTRL-1-xxx] [SEQ-5] applyMove result: SUCCESS
[CTRL-1-xxx] [SEQ-7] Next player is HUMAN, waiting
```

**Expected result**: NO second search, NO checkmate modal!

## Files Modified

- `/Users/fgantt/projects/vibe/shogi-game/worktrees/usi/src/components/GamePage.tsx`
  - Removed `gameInitializedRef.current = false;` from cleanup function
  - Added logging to track ref changes

## Impact

This fix ensures:
- ✅ Only ONE game initialization per navigation
- ✅ Only ONE AI search request
- ✅ No duplicate move attempts
- ✅ No false checkmate detection
- ✅ Proper state synchronization

## Production Note

This bug **only affects development mode** (React Strict Mode). In production builds, React Strict Mode is disabled, so the component only mounts once. However, it's still important to fix it properly for development.

## Key Lesson

**Be careful with `useRef` cleanup in React Strict Mode:**
- Refs persist across re-mounts by design
- Resetting refs in cleanup defeats their purpose
- Only reset refs when logically appropriate (e.g., user-initiated new game)

## Related Files

Previous investigation documentation:
- `docs/architecture/STATE_MANAGEMENT_ARCHITECTURE.md`
- `docs/architecture/INVESTIGATION_PLAN.md`
- `docs/architecture/CURRENT_PROBLEM_ANALYSIS.md`
- `INVESTIGATION_READY.md`
- `TEST_INSTRUCTIONS.md`
- `REACT_STRICT_MODE_FIX.md` (earlier incorrect fix attempt)

This fix supersedes the earlier `REACT_STRICT_MODE_FIX.md` which set the ref before the async call but didn't address the cleanup issue.

