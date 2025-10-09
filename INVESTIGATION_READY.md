# Investigation Instrumentation Complete

## Date
October 9, 2025

## Summary

All investigation instrumentation from `INVESTIGATION_PLAN.md` steps 2-4 has been implemented. The application is now ready for systematic testing to identify the root cause of state desynchronization.

---

## Implemented Features

### ✅ Step 2: Instance Tracking

**ShogiController** (`src/usi/controller.ts`)
- Static instance counter tracks total controllers created
- Unique instanceId for each controller
- Instance number and ID logged on creation
- All console.log statements now prefixed with `[CTRL-N-xxxxx]`

**WasmEngineAdapter** (`src/usi/engine.ts`)
- Static instance counter tracks total adapters created
- Unique instanceId for each adapter (includes session name)
- Instance number and ID logged on creation
- All console.log statements now prefixed with `[ADAPTER-session-N-xxxxx]`

**GamePage Component** (`src/components/GamePage.tsx`)
- Unique componentId for each component instance
- Mount/unmount lifecycle logging
- Navigation effect logging with skip/proceed indicators

### ✅ Step 3: State Snapshot Logging

**logRecordState() method added to Controller**
- Logs complete record state at any point
- Shows: SFEN, Turn, Move count, Player types, isCurrentPlayerAI result
- Called at key decision points

**Logging added to:**
- `applyMove()` - BEFORE and AFTER states
- `requestEngineMove()` - SEQ-1 state snapshot
- `bestmove handler` - SEQ-4 and SEQ-5 state snapshots

### ✅ Step 4: Sequence Logging

**Complete sequence tracking in AI move flow:**
- `[SEQ-1]` - requestEngineMove START + state
- `[SEQ-2]` - Calling engine.setPosition
- `[SEQ-3]` - Calling engine.go
- `[SEQ-4]` - bestmove received + state
- `[SEQ-5]` - applyMove result + state
- `[SEQ-6]` - Checking if next player is AI
- `[SEQ-7]` - Next player IS AI / is HUMAN decision

**WASM Engine already has:**
- `[HANDLE_POSITION] STATE SET - VERIFICATION` - after parsing SFEN
- `[GET_BEST_MOVE] CALLED - ENGINE INTERNAL STATE` - at search start

---

## What to Look For in Logs

### Normal Single-Controller Behavior

```
========================================
[CTRL-1-xxxxx] Controller created
[CTRL-1-xxxxx] Total controllers: 1
========================================

[ADAPTER-gote-1-xxxxx] Adapter created for session: gote
[ADAPTER-gote-1-xxxxx] Total adapters: 1

[ADAPTER-sente-2-xxxxx] Adapter created for session: sente
[ADAPTER-sente-2-xxxxx] Total adapters: 2

[COMPONENT-xxxxx] GamePage MOUNTED
[COMPONENT-xxxxx] Navigation effect triggered
[COMPONENT-xxxxx] gameInitializedRef.current: false
[COMPONENT-xxxxx] PROCEEDING with initialization
```

### 🚨 RED FLAG: Multiple Controllers

```
[CTRL-1-xxxxx] Controller created
[CTRL-1-xxxxx] Total controllers: 1

[CTRL-2-yyyyy] Controller created  ← ❌ SECOND CONTROLLER!
[CTRL-2-yyyyy] Total controllers: 2
```

If you see this, React Strict Mode is creating two controllers.

### 🚨 RED FLAG: Mismatched States

```
[CTRL-1-xxxxx] [SEQ-2] Calling engine.setPosition
[ADAPTER-gote-1-xxxxx] SFEN: 4k4/.../w - 1  ← White to move

[CTRL-1-xxxxx] [SEQ-4] bestmove received: 5c5d
[CTRL-1-xxxxx] SFEN: 4k4/.../b - 1  ← Now Black to move

[CTRL-1-xxxxx] [SEQ-1] requestEngineMove START
[CTRL-1-xxxxx] SFEN: 4k4/.../w - 1  ← ❌ BACK TO WHITE?!
```

This indicates state corruption or multiple instances.

### 🚨 RED FLAG: Wrong Instance Handling

```
[CTRL-1-xxxxx] [SEQ-2] Calling engine.setPosition
[ADAPTER-gote-1-xxxxx] Setting position: 4k4/.../w - 1

[CTRL-2-yyyyy] [SEQ-4] bestmove received: 5c5d  ← ❌ Different controller!
```

This means the bestmove went to the wrong controller instance.

---

## Testing Instructions

### Test 1: Check for Multiple Controllers

1. Open browser console
2. Clear console
3. Navigate to game page with handicap settings
4. Look for controller creation logs
5. **Expected**: ONE controller (`CTRL-1-xxxxx`)
6. **Problem**: TWO controllers (`CTRL-1` and `CTRL-2`)

### Test 2: Trace Single AI Move

1. Start a 10-piece handicap game (AI as White)
2. Watch for complete sequence: SEQ-1 → SEQ-7
3. **Expected sequence**:
   ```
   [CTRL-1-xxx] [SEQ-1] requestEngineMove START
   [CTRL-1-xxx] SFEN: 4k4/.../w - 1, Turn: White
   [CTRL-1-xxx] [SEQ-2] Calling engine.setPosition
   [ADAPTER-gote-1-xxx] Setting position
   [ADAPTER-gote-1-xxx] SFEN: 4k4/.../w - 1
   [CTRL-1-xxx] [SEQ-3] Calling engine.go
   [HANDLE_POSITION] STATE SET: player=White, fen=4k4/.../w -
   [GET_BEST_MOVE] ENGINE STATE: player=White, fen=4k4/.../w -
   [CTRL-1-xxx] [SEQ-4] bestmove received: 5c5d
   [CTRL-1-xxx] SFEN: 4k4/.../w - 1, Turn: White
   [CTRL-1-xxx] [SEQ-5] applyMove result: SUCCESS
   [CTRL-1-xxx] SFEN: 4k4/.../b - 1, Turn: Black
   [CTRL-1-xxx] [SEQ-6] Checking if next player is AI
   [CTRL-1-xxx] [SEQ-7] Next player is HUMAN, waiting
   ```

4. **Problem indicators**:
   - Different controller IDs in sequence
   - SFEN reverting to old state
   - SEQ-7 says "IS AI" when should be "HUMAN"
   - Second SEQ-1 before first sequence completes

### Test 3: Verify Engine Count

1. Count unique ADAPTER instance IDs
2. **Expected**: 2-4 adapters
   - ADAPTER-gote-1-xxx (main White engine)
   - ADAPTER-sente-2-xxx (main Black engine)
   - Possibly 2 more for recommendations
3. **Problem**: More than 4, or duplicates of same session

---

## Diagnosis Guide

### If You See: Multiple Controllers

**Root Cause**: React Strict Mode double-initialization, gameInitializedRef not working

**Solution**: Implement one of:
1. React Context (recommended)
2. Singleton pattern
3. Fix gameInitializedRef timing (already attempted)

### If You See: State Reverting

**Root Cause**: Multiple controllers with separate game records

**Solution**: Ensure only one controller instance exists

### If You See: Wrong Engine Session

**Root Cause**: isCurrentPlayerAI() logic error or player type mismatch

**Solution**: Review player type settings and turn detection

### If You See: Duplicate Sequences

**Root Cause**: Two initializations both calling newGame()

**Solution**: Verify gameInitializedRef is set BEFORE async call

---

## Next Steps After Testing

1. **Collect complete log** from console
2. **Count instances**: Controllers, Adapters, Components
3. **Trace sequence**: Follow one move from SEQ-1 to SEQ-7
4. **Identify issue**: Use diagnosis guide above
5. **Implement fix**: Based on findings
6. **Re-test**: Verify fix works

---

## Development Server

Server should be running at: `http://localhost:5173/`

Navigate to a handicap game setup to trigger all the logging.

**Remember**: Each player (sente/gote) should have their own engine instance as you noted - this is working correctly. We expect 2 main engines + potentially 2 recommendation engines = 4 total adapters.

---

## Files Modified

- `/Users/fgantt/projects/vibe/shogi-game/worktrees/usi/src/usi/controller.ts`
- `/Users/fgantt/projects/vibe/shogi-game/worktrees/usi/src/usi/engine.ts`
- `/Users/fgantt/projects/vibe/shogi-game/worktrees/usi/src/components/GamePage.tsx`

All changes are additive (logging only) - no behavior changes.

