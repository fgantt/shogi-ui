# State Management Architecture Analysis

## Date
October 9, 2025

## Purpose
Document the complete call sequence, data flow, and source of truth for game state management between the React UI and WASM AI engines.

---

## Source of Truth

**PRIMARY SOURCE OF TRUTH: `controller.record` (tsshogi `Record` object)**

The `ShogiController` maintains a `tsshogi.Record` object which contains:
- Current position (`record.position`) with SFEN string
- Full move history (`record.moves`)
- Legal move validation
- Turn management

**All game state must flow through this single source of truth.**

---

## Architecture Components

### 1. React UI Layer (`GamePage.tsx`)
- **Role**: Display and user interaction
- **State**: React state for UI rendering only
- **Does NOT**: Make game logic decisions

### 2. Controller Layer (`ShogiController`)
- **Role**: Game state management and orchestration
- **State**: 
  - `record: Record` - THE source of truth
  - `player1Type`, `player2Type` - Player configuration
  - Engine references (sente, gote)
- **Responsibilities**:
  - Validate and apply moves via `record.append()`
  - Synchronize engines with current position
  - Emit state changes to UI
  - Determine game over conditions

### 3. WASM Engine Layer (`WasmEngineAdapter`)
- **Role**: AI move generation ONLY
- **State**: Internal board representation for search
- **Does NOT**: Maintain authoritative game state
- **Must**: Be synchronized before each search

---

## Call Sequence: AI Move Generation

### Ideal Flow

```
1. UI triggers AI move request
   └─> controller.requestEngineMove()

2. Controller prepares engine
   ├─> Get current SFEN from record.position.sfen
   ├─> Determine which engine (sente/gote) based on turn
   └─> engine.setPosition(currentSfen, [])  // Empty moves!

3. Engine internal processing
   ├─> handle_position() parses SFEN
   ├─> Updates internal board state
   └─> get_best_move() searches for best move

4. Engine returns move
   └─> Emits 'bestmove' event with USI string

5. Controller receives bestmove
   ├─> Validates move: record.position.createMoveByUSI()
   ├─> Applies move: record.append(move)
   ├─> Updates record.position (turn switches automatically)
   └─> Emits 'stateChanged' to UI

6. UI receives stateChanged
   └─> Re-renders with new position

7. If next player is AI, goto step 1
```

### Critical Rule
**WASM engines receive ONLY:**
- Current SFEN (complete position)
- Empty moves array `[]`

**Why?** The SFEN already represents the complete board state after all moves. Passing moves causes double-application.

---

## Current Issues Observed

### Issue 1: Double Initialization
**Symptom**: AI makes same move twice with old position

**Root Cause**: React Strict Mode mounts components twice
- First mount: calls `controller.newGame()`
- Cleanup runs
- Second mount: calls `controller.newGame()` again
- Result: Two game instances, two engines searching

**Status**: Fix attempted in GamePage.tsx with `gameInitializedRef`

### Issue 2: Engine State Desynchronization
**Symptom**: Engine reports wrong position in GET_BEST_MOVE

**Evidence from logs**:
```
[HANDLE_POSITION] Verification FEN: 4k4/9/pppp1pppp/4p4/9/9/...
[GET_BEST_MOVE] Position FEN: 4k4/9/ppppppppp/9/9/9/...  ← OLD!
```

**Possible Causes**:
1. Multiple engine instances (one per mount)
2. Engine state not persisting between handle_position and get_best_move
3. Async timing issues
4. Worker thread state isolation

---

## Data Flow Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                      React UI (GamePage)                     │
│  - Renders board                                             │
│  - Handles user clicks                                       │
│  - Displays modals                                           │
└────────────────────┬────────────────────────────────────────┘
                     │
                     │ Events: onMoveMade, onStateChanged
                     │ Commands: makeMove(), requestEngineMove()
                     ▼
┌─────────────────────────────────────────────────────────────┐
│              ShogiController (Source of Truth)               │
│  ┌──────────────────────────────────────────────┐           │
│  │  record: Record (tsshogi)                    │           │
│  │  - position: Position (SFEN, turn, etc)      │           │
│  │  - moves: Move[]                             │           │
│  └──────────────────────────────────────────────┘           │
│                                                               │
│  Methods:                                                    │
│  - applyMove(usiMove) → validates & updates record          │
│  - requestEngineMove() → syncs engine & requests search     │
│  - isCurrentPlayerAI() → determines if AI should move       │
└──────────┬────────────────────────┬─────────────────────────┘
           │                        │
           │ setPosition(sfen, [])  │ setPosition(sfen, [])
           ▼                        ▼
┌──────────────────────┐  ┌──────────────────────┐
│  Sente Engine        │  │  Gote Engine         │
│  (Player 1 / Black)  │  │  (Player 2 / White)  │
│                      │  │                      │
│  - Receives SFEN     │  │  - Receives SFEN     │
│  - Runs search       │  │  - Runs search       │
│  - Returns bestmove  │  │  - Returns bestmove  │
└──────────────────────┘  └──────────────────────┘
           │                        │
           └────────────┬───────────┘
                        │
                        │ bestmove event
                        ▼
              Back to Controller
```

---

## State Synchronization Points

### Point 1: Game Initialization
```typescript
// GamePage.tsx
controller.newGame(initialSfen)
  └─> ShogiController.newGame()
      ├─> record = createRecord(initialSfen)
      └─> synchronizeAllEngines(record.position.sfen, [])
          ├─> senteEngine.setPosition(sfen, [])
          └─> goteEngine.setPosition(sfen, [])
```

### Point 2: Human Move
```typescript
// GamePage.tsx
controller.makeMove(fromSquare, toSquare)
  └─> ShogiController.makeMove()
      ├─> move = record.position.createMove(from, to)
      ├─> record.append(move)  // SOURCE OF TRUTH UPDATED
      ├─> emit('stateChanged')
      └─> if (isCurrentPlayerAI()) requestEngineMove()
```

### Point 3: AI Move Request
```typescript
// ShogiController.requestEngineMove()
├─> isPlayer1Turn = record.position.sfen.includes(' b ')
├─> sessionId = isPlayer1Turn ? 'sente' : 'gote'
├─> engine = getEngine(sessionId)
├─> currentSfen = record.position.sfen  // GET FROM SOURCE OF TRUTH
└─> engine.setPosition(currentSfen, [])  // SYNC ENGINE
    └─> engine.go()
```

### Point 4: AI Move Response
```typescript
// ShogiController bestmove handler
engine.on('bestmove', ({ move: usiMove }) => {
  ├─> move = record.position.createMoveByUSI(usiMove)  // VALIDATE
  ├─> if (!move) → GAME OVER (invalid move)
  ├─> record.append(move)  // UPDATE SOURCE OF TRUTH
  ├─> emit('stateChanged')
  └─> if (isCurrentPlayerAI()) requestEngineMove()  // CHAIN IF AI vs AI
})
```

---

## Critical Bugs to Investigate

### Bug 1: Why is GET_BEST_MOVE seeing old position?

**Question**: Is `handle_position` and `get_best_move` executing on the same engine instance?

**Test**: Add instance ID to WASM engine
```rust
pub struct ShogiEngine {
    instance_id: String,  // Generate UUID on new()
    // Log this in both handle_position and get_best_move
}
```

### Bug 2: Why is engine being called twice?

**Question**: Is React Strict Mode creating two controllers or two engines?

**Test**: Add instance ID to ShogiController
```typescript
class ShogiController {
  private instanceId = Math.random().toString(36);
  // Log this in all methods
}
```

### Bug 3: Are we calling requestEngineMove() at the wrong time?

**Question**: When does `isCurrentPlayerAI()` check get called?

**Test**: Add comprehensive logging to bestmove handler
```typescript
console.log('Before applyMove:', record.position.sfen);
const moveResult = applyMove(usiMove);
console.log('After applyMove:', record.position.sfen);
console.log('isCurrentPlayerAI():', isCurrentPlayerAI());
```

---

## Proposed Investigation Steps

### Step 1: Add Instance Tracking
Add unique IDs to:
- [ ] ShogiController instances
- [ ] WASM Engine instances  
- [ ] Web Worker instances

Log instance IDs in every method to track if we're dealing with multiple instances.

### Step 2: Add State Snapshots
Before and after every state change, log:
- [ ] Controller's `record.position.sfen`
- [ ] Controller's `record.moves.length`
- [ ] Which engine is being called (sessionId)
- [ ] Current turn (black/white)

### Step 3: Trace Complete Flow
Pick ONE move and trace it completely:
- [ ] Log entry to every method
- [ ] Log all state before/after
- [ ] Log all engine communications
- [ ] Verify SFEN at each step

### Step 4: Verify Worker Isolation
Check if Web Workers maintain separate state:
- [ ] Does each worker have its own engine instance?
- [ ] Are workers being recreated?
- [ ] Is state leaking between workers?

### Step 5: Check React Lifecycle
- [ ] Verify gameInitializedRef is working
- [ ] Check if controller is being recreated
- [ ] Verify only one newGame() call executes

---

## Questions to Answer

1. **How many ShogiController instances exist?**
   - Should be: 1 per game session
   - Check: Add instance counter

2. **How many WASM engine instances exist?**
   - Should be: 2 (sente + gote) + 2 (recommendation engines) = 4 total
   - Check: Add instance tracking

3. **When does each engine get synchronized?**
   - Should be: Before every `go()` command
   - Check: Log all setPosition calls

4. **What is the state of record.position after each move?**
   - Should be: Updated immediately after append()
   - Check: Log SFEN after every append()

5. **Why is the second bestmove using old SFEN?**
   - Hypothesis: Second engine hasn't been synchronized
   - Check: Verify setPosition was called with correct SFEN

---

## Next Actions

1. **Immediate**: Add instance IDs to all components
2. **Short-term**: Implement comprehensive state snapshot logging
3. **Medium-term**: Create integration test for AI vs AI game
4. **Long-term**: Consider refactoring to Redux or similar for clearer state management

---

## Design Principles (for fixing)

1. **Single Source of Truth**: `controller.record` is authoritative
2. **Synchronize Before Search**: Always call `setPosition()` before `go()`
3. **Empty Moves Array**: Never pass moves to `setPosition()`, only SFEN
4. **Validate Before Apply**: Always validate moves from engine before applying
5. **One Controller**: Prevent multiple controller instances
6. **Engine State is Ephemeral**: Engines should be stateless search machines

