# Systematic Investigation Plan

## Objective
Identify the root cause of state desynchronization between React UI and WASM engines.

---

## Phase 1: Instance Tracking

### 1.1 Add Instance ID to ShogiController

**File**: `src/usi/controller.ts`

```typescript
export class ShogiController extends EventEmitter {
  private instanceId: string;
  
  constructor() {
    super();
    this.instanceId = `CTRL-${Math.random().toString(36).substr(2, 9)}`;
    console.log(`[${this.instanceId}] Controller created`);
  }
  
  // Add instanceId to ALL console.log statements:
  // console.log(`[${this.instanceId}] message here`)
}
```

**Expected Result**: Should see only ONE controller ID in logs. If multiple IDs appear, we have multiple controllers.

### 1.2 Add Instance ID to WASM Engine

**File**: `src/lib.rs`

```rust
#[wasm_bindgen]
pub struct ShogiEngine {
    instance_id: String,
    // ... existing fields
}

impl ShogiEngine {
    pub fn new() -> Self {
        let instance_id = format!("ENG-{}", random_id());
        crate::debug_utils::debug_log(&format!("[{}] Engine created", instance_id));
        
        Self {
            instance_id,
            // ... existing initialization
        }
    }
}

// Add instance_id to all debug logs:
// format!("[{}] message", self.instance_id)
```

**Expected Result**: Should see exactly 4 engine IDs (2 main + 2 recommendation). Track which ID is used for which call.

### 1.3 Add Session Tracking to Engine Adapter

**File**: `src/usi/engine.ts`

```typescript
export class WasmEngineAdapter implements EngineAdapter {
  private instanceId: string;
  
  constructor(sessionId: string) {
    this.sessionId = sessionId;
    this.instanceId = `ADAPTER-${sessionId}-${Math.random().toString(36).substr(2, 9)}`;
    console.log(`[${this.instanceId}] Adapter created for session: ${sessionId}`);
  }
  
  // Add instanceId to all console.log statements
}
```

**Expected Result**: Track which adapter instance handles which command.

---

## Phase 2: State Snapshot Logging

### 2.1 Log Record State After Every Change

**File**: `src/usi/controller.ts`

```typescript
private logRecordState(context: string): void {
  const sfen = this.record.position.sfen;
  const turn = sfen.includes(' b ') ? 'Black' : 'White';
  const moveCount = this.record.moves.length;
  
  console.log(`[${this.instanceId}] [${context}] RECORD STATE:`);
  console.log(`  SFEN: ${sfen}`);
  console.log(`  Turn: ${turn}`);
  console.log(`  Moves: ${moveCount}`);
  console.log(`  Player1Type: ${this.player1Type}, Player2Type: ${this.player2Type}`);
  console.log(`  isCurrentPlayerAI: ${this.isCurrentPlayerAI()}`);
}

// Call after every state change:
private applyMove(usiMove: string): Move | null {
  this.logRecordState('BEFORE applyMove');
  // ... existing logic
  const result = this.record.append(move);
  this.logRecordState('AFTER applyMove');
  return move;
}

public requestEngineMove(): void {
  this.logRecordState('BEFORE requestEngineMove');
  // ... existing logic
}
```

### 2.2 Log Engine State at Key Points

**File**: `src/lib.rs`

```rust
pub fn handle_position(&mut self, parts: &[&str]) -> Vec<String> {
    self.log_engine_state("START handle_position");
    // ... parse and set state
    self.log_engine_state("END handle_position");
}

pub fn get_best_move(&mut self, ...) -> Option<Move> {
    self.log_engine_state("START get_best_move");
    // ... search logic
}

fn log_engine_state(&self, context: &str) {
    let fen = self.board.to_fen(self.current_player, &self.captured_pieces);
    crate::debug_utils::debug_log(&format!(
        "[{}] [{}] ENGINE STATE: player={:?}, fen={}",
        self.instance_id, context, self.current_player, fen
    ));
}
```

---

## Phase 3: Call Sequence Tracing

### 3.1 Trace Single AI Move End-to-End

Add numbered sequence logging:

```typescript
// Controller
requestEngineMove() {
  console.log(`[${this.instanceId}] [SEQ-1] requestEngineMove START`);
  this.logRecordState('SEQ-1');
  // ...
  console.log(`[${this.instanceId}] [SEQ-2] Calling engine.setPosition`);
  engine.setPosition(currentSfen, []);
  console.log(`[${this.instanceId}] [SEQ-3] Calling engine.go`);
  engine.go({...});
}

// bestmove handler
engine.on('bestmove', ({ move: usiMove }) => {
  console.log(`[${this.instanceId}] [SEQ-4] bestmove received: ${usiMove}`);
  this.logRecordState('SEQ-4');
  
  const moveResult = this.applyMove(usiMove);
  console.log(`[${this.instanceId}] [SEQ-5] applyMove result: ${moveResult ? 'SUCCESS' : 'FAILED'}`);
  this.logRecordState('SEQ-5');
  
  console.log(`[${this.instanceId}] [SEQ-6] Checking if next player is AI`);
  if (this.isCurrentPlayerAI()) {
    console.log(`[${this.instanceId}] [SEQ-7] Next player IS AI, requesting move`);
    this.requestEngineMove();
  } else {
    console.log(`[${this.instanceId}] [SEQ-7] Next player is HUMAN, waiting`);
  }
});
```

---

## Phase 4: React Lifecycle Verification

### 4.1 Track Component Mount/Unmount

**File**: `src/components/GamePage.tsx`

```typescript
const componentId = useRef(`COMPONENT-${Math.random().toString(36).substr(2, 9)}`);

useEffect(() => {
  console.log(`[${componentId.current}] GamePage MOUNTED`);
  console.log(`[${componentId.current}] gameInitializedRef.current: ${gameInitializedRef.current}`);
  
  return () => {
    console.log(`[${componentId.current}] GamePage UNMOUNTING`);
  };
}, []);

useEffect(() => {
  console.log(`[${componentId.current}] Navigation effect triggered`);
  console.log(`[${componentId.current}] gameInitializedRef.current: ${gameInitializedRef.current}`);
  
  if (gameInitializedRef.current) {
    console.log(`[${componentId.current}] SKIPPING initialization - already initialized`);
    return;
  }
  
  console.log(`[${componentId.current}] PROCEEDING with initialization`);
  // ... initialization logic
}, [location.state, controller]);
```

---

## Phase 5: Verification Checklist

Run a single handicap game and verify:

### Expected Log Sequence for First AI Move

```
[COMPONENT-xxx] GamePage MOUNTED
[COMPONENT-xxx] Navigation effect triggered
[CTRL-xxx] Controller created
[ADAPTER-gote-xxx] Adapter created for session: gote
[ADAPTER-sente-xxx] Adapter created for session: sente
[ENG-xxx] Engine created (gote)
[ENG-yyy] Engine created (sente)

[CTRL-xxx] [SEQ-1] requestEngineMove START
[CTRL-xxx] [SEQ-1] RECORD STATE: SFEN=4k4/.../w - 1, Turn=White
[CTRL-xxx] [SEQ-2] Calling engine.setPosition
[ADAPTER-gote-xxx] setPosition: SFEN=4k4/.../w - 1
[ENG-xxx] [START handle_position] ENGINE STATE: player=White, fen=4k4/.../w -
[ENG-xxx] [END handle_position] ENGINE STATE: player=White, fen=4k4/.../w -
[CTRL-xxx] [SEQ-3] Calling engine.go
[ENG-xxx] [START get_best_move] ENGINE STATE: player=White, fen=4k4/.../w -
[ENG-xxx] Search completed: bestmove 5c5d

[CTRL-xxx] [SEQ-4] bestmove received: 5c5d
[CTRL-xxx] [SEQ-4] RECORD STATE: SFEN=4k4/.../w - 1, Turn=White
[CTRL-xxx] [BEFORE applyMove] RECORD STATE: SFEN=4k4/.../w - 1, Turn=White
[CTRL-xxx] [AFTER applyMove] RECORD STATE: SFEN=4k4/.../b - 1, Turn=Black
[CTRL-xxx] [SEQ-5] applyMove result: SUCCESS
[CTRL-xxx] [SEQ-6] Checking if next player is AI
[CTRL-xxx] [SEQ-7] Next player is HUMAN, waiting
```

### Red Flags to Watch For

1. **Multiple controller IDs** → Controller being recreated
2. **More than 4 engine IDs** → Engines being recreated
3. **Engine FEN doesn't match SFEN sent** → Synchronization failure
4. **Two SEQ-1 calls before SEQ-4** → Double initialization
5. **SEQ-7 says "IS AI" when should be HUMAN** → Player type logic error

---

## Phase 6: Hypothesis Testing

Based on logs, test specific hypotheses:

### Hypothesis 1: React Strict Mode Double Initialization
**Test**: Check if we see two complete SEQ-1 to SEQ-3 sequences before any SEQ-4
**Fix**: Ensure gameInitializedRef is set BEFORE async call (already attempted)

### Hypothesis 2: Multiple Controllers
**Test**: Check if we see multiple CTRL-xxx IDs
**Fix**: Use React Context or singleton pattern

### Hypothesis 3: Engine State Not Persisting
**Test**: Check if handle_position END state matches get_best_move START state
**Fix**: Investigate WASM memory/state issues

### Hypothesis 4: Wrong Engine Being Called
**Test**: Check if sessionId matches expected player
**Fix**: Review isCurrentPlayerAI() logic

### Hypothesis 5: Race Condition
**Test**: Check if SEQ-4 from first search arrives after SEQ-3 of second search
**Fix**: Add search session IDs and ignore stale responses

---

## Implementation Order

1. ✅ **Document Architecture** (this file + STATE_MANAGEMENT_ARCHITECTURE.md)
2. ⬜ **Add Instance IDs** to all components (1 hour)
3. ⬜ **Add State Logging** to all state changes (1 hour)
4. ⬜ **Add Sequence Logging** to AI move flow (30 min)
5. ⬜ **Test and Collect Logs** from single game (30 min)
6. ⬜ **Analyze Logs** to identify issue (1-2 hours)
7. ⬜ **Implement Fix** based on findings (varies)
8. ⬜ **Verify Fix** with comprehensive testing (1 hour)

**Total Estimated Time**: 5-7 hours of focused work

---

## Success Criteria

Fix is successful when:
- ✅ Only ONE controller instance ID in logs
- ✅ Exactly 4 engine instance IDs (2 main + 2 recommendation)
- ✅ Engine FEN always matches controller SFEN
- ✅ No duplicate move requests
- ✅ AI moves work correctly in all game modes
- ✅ No premature checkmate modals
- ✅ Handicap games work correctly

