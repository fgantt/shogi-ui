# Quick Test Instructions

## What Was Done

Implemented comprehensive logging for steps 2-5 of the Investigation Plan:
- ✅ Instance tracking for all components (Controller, Adapters, React component)
- ✅ State snapshot logging at every decision point
- ✅ Sequence tracing for AI moves (SEQ-1 through SEQ-7)
- Ready for systematic testing

## How to Test Right Now

1. **Open browser** to `http://localhost:5173/`
2. **Open DevTools Console** (F12 or Cmd+Opt+I)
3. **Clear the console**
4. **Start a 10-piece handicap game**:
   - Player 1 (Black): Human
   - Player 2 (White): AI (Easy)
   - Select "10-piece handicap" from presets
   - Click Start Game

## What to Look For

### ✅ GOOD - Single Controller
```
[CTRL-1-xxxxx] Controller created
[CTRL-1-xxxxx] Total controllers: 1
```

### ❌ BAD - Multiple Controllers  
```
[CTRL-1-xxxxx] Controller created
[CTRL-1-xxxxx] Total controllers: 1
[CTRL-2-yyyyy] Controller created  ← PROBLEM!
[CTRL-2-yyyyy] Total controllers: 2
```

### ✅ GOOD - Consistent Sequence (Same Controller ID Throughout)
```
[CTRL-1-xxx] [SEQ-1] requestEngineMove START
[CTRL-1-xxx] [SEQ-2] Calling engine.setPosition
[CTRL-1-xxx] [SEQ-3] Calling engine.go
[CTRL-1-xxx] [SEQ-4] bestmove received: 5c5d
[CTRL-1-xxx] [SEQ-5] applyMove result: SUCCESS
[CTRL-1-xxx] [SEQ-6] Checking if next player is AI
[CTRL-1-xxx] [SEQ-7] Next player is HUMAN
```

### ❌ BAD - Inconsistent Controller IDs
```
[CTRL-1-xxx] [SEQ-1] requestEngineMove START
[CTRL-1-xxx] [SEQ-2] Calling engine.setPosition
[CTRL-1-xxx] [SEQ-3] Calling engine.go
[CTRL-2-yyy] [SEQ-4] bestmove received: 5c5d  ← Different controller!
```

### ❌ BAD - State Reverting
```
[CTRL-1-xxx] [AFTER applyMove] SFEN: .../b - 1, Turn: Black
[CTRL-1-xxx] [SEQ-1] requestEngineMove START
[CTRL-1-xxx] SFEN: .../w - 1, Turn: White  ← Reverted!
```

## Quick Diagnosis

**If you see MULTIPLE controllers** → React Strict Mode is creating two instances
  - Solution: Need to implement React Context or Singleton pattern

**If you see DIFFERENT controller IDs in one sequence** → Multiple controllers handling same game
  - Solution: Same as above

**If you see REVERTING state** → Controllers have separate game records
  - Solution: Same as above

**If you see CORRECT logs but STILL get checkmate modal** → Different issue, need deeper investigation

## Expected Normal Flow

For a successful first AI move in handicap game:
1. Component mounts ONCE
2. Controller created ONCE (CTRL-1)
3. Two adapters created (sente and gote)
4. SEQ-1: Request engine move
5. Engine receives correct SFEN (White to move)
6. Engine returns valid move
7. SEQ-5: Move applied successfully
8. State changes to Black's turn
9. SEQ-7: Next player is HUMAN (stops)
10. NO checkmate modal

## What to Share

Please share the console output from browser DevTools, particularly:
- The controller creation lines
- The complete SEQ-1 through SEQ-7 sequence
- Any error messages or unexpected behavior

This will help us pinpoint the exact issue! 🎯

