# Current Problem: Detailed Analysis

## The Bug

AI makes first move successfully, then immediately tries to make THE SAME MOVE again with stale position, causing invalid move detection and false checkmate modal.

---

## Evidence from Logs

### First Move (✓ SUCCESS)

```
[ENGINE gote] Setting position:
  SFEN: 4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1
  [HANDLE_POSITION] STATE SET: player=White, fen=4k4/.../w -
  [GET_BEST_MOVE] ENGINE STATE: player=White, fen=4k4/.../w -
  
bestmove 5c5d

[CONTROLLER] About to apply AI move. Current position: 4k4/.../w - 1
[CONTROLLER] Current turn: White
[CONTROLLER] AI session: gote Move: 5c5d
[CONTROLLER] ✓ Move applied successfully
[CONTROLLER] New position SFEN: 4k4/9/pppp1pppp/4p4/9/9/... b - 1
[CONTROLLER] New turn: Black
```

**Analysis**: ✅ Everything correct
- Engine receives correct SFEN (White to move)
- Engine state is correct (White, handicap position)
- Move 5c5d is generated
- Move applies successfully
- Position updates to Black's turn

### Second Move (✗ FAILURE)

```
[ENGINE gote] Setting position:
  SFEN: 4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1  ← OLD SFEN!
  [HANDLE_POSITION] STATE SET: player=White, fen=4k4/.../w -
  [GET_BEST_MOVE] ENGINE STATE: player=White, fen=4k4/.../w -
  
bestmove 5c5d  ← SAME MOVE!

[CONTROLLER] About to apply AI move. Current position: 4k4/.../b - 1  ← BLACK TO MOVE!
[CONTROLLER] Current turn: Black
[CONTROLLER] AI session: gote Move: 5c5d
[CONTROLLER] ✗ createMoveByUSI returned null
[CONTROLLER] INVALID MOVE - GAME OVER! Winner: player2
```

**Analysis**: ❌ Multiple problems
- Engine receives OLD SFEN (still showing White to move, pawn at 5c)
- Engine state is stale
- Engine generates same move (5c5d)
- Controller's record shows BLACK to move (correct)
- Move fails because White can't move on Black's turn

---

## The Critical Question

**Why is `engine.setPosition()` being called with the OLD SFEN?**

The controller code shows:
```typescript
public requestEngineMove(): void {
  const currentSfen = this.record.position.sfen;  // Should be current!
  engine.setPosition(currentSfen, []);
}
```

**Hypothesis A: requestEngineMove() is being called before record updates**
- Unlikely - logs show "New position" before second engine call

**Hypothesis B: Multiple controller instances**
- Second controller still has old record
- React Strict Mode creates two controllers
- Both try to manage the game

**Hypothesis C: Async race condition**
- First bestmove handler still executing
- Second requestEngineMove() called before record.append() completes
- Reads stale record.position

**Hypothesis D: Wrong engine session**
- gote engine is being called when should call sente
- gote engine hasn't been synchronized since game start

---

## Code Flow Analysis

### What SHOULD Happen After First Move

```typescript
// In bestmove handler
1. AI move 5c5d arrives
2. record.append(move)
   → record.position.sfen = "...b - 1" (Black to move)
3. emit('stateChanged')
4. isCurrentPlayerAI() checks turn
   → record.position.sfen.includes(' b ') = true → Player 1 turn
   → this.player1Type = 'human' → return false
5. Does NOT call requestEngineMove()
```

### What IS Happening

```typescript
1. AI move 5c5d arrives
2. record.append(move)
   → record.position.sfen = "...b - 1" (Black to move)
3. emit('stateChanged')
4. isCurrentPlayerAI() checks turn (SOMEHOW RETURNS TRUE??)
5. requestEngineMove() IS called
   → BUT: reading OLD record.position.sfen somehow?
   → OR: called on DIFFERENT controller instance?
```

---

## Key Clues

### Clue 1: Double Initialization
```
GamePage.tsx:262 Initializing game from navigation state: {...}
GamePage.tsx:374 [GAMEPAGE] Cleaning up gameOver event listener  ← cleanup
GamePage.tsx:262 Initializing game from navigation state: {...}  ← SECOND INIT!
```

This proves React Strict Mode is mounting twice AND both are executing.

### Clue 2: Engine Session is Correct
```
[CONTROLLER] AI session: gote
```

Both calls use `gote`, which is correct for White/Player2. So it's not wrong engine.

### Clue 3: Controller Has Correct State
```
[CONTROLLER] Current position: 4k4/9/pppp1pppp/4p4/...b - 1
[CONTROLLER] Current turn: Black
```

The controller that's trying to apply the move has the CORRECT state. But the engine was sent OLD state.

### Clue 4: Two Separate Searches
The logs show two complete engine initializations:
```
[0ms] handle_position called with 5 parts  ← First search
[0ms] handle_position called with 5 parts  ← Second search
```

Both searches complete and return `bestmove 5c5d`.

---

## Most Likely Cause: Multiple Controllers

### Evidence Supporting This Theory

1. **Two complete game initializations**
   - React Strict Mode mounts component twice
   - Each mount creates its own controller instance
   - Each controller calls newGame()

2. **gameInitializedRef not working**
   - Ref is set AFTER async newGame() completes
   - Both mounts execute newGame() before either completes
   - Result: Two controllers, two games

3. **Second controller has stale state**
   - Controller A: Has current state after first move
   - Controller B: Still has initial state
   - When Controller B's engine returns, it tries to apply to its own stale record

4. **Both controllers share same engines???**
   - If engines are singletons or stored globally
   - Both controllers might be calling same engine instances
   - Engine gets mixed commands from two controllers

---

## Test to Confirm Multiple Controllers

Add to ShogiController constructor:
```typescript
private static instanceCount = 0;
private instanceNumber: number;

constructor() {
  this.instanceNumber = ++ShogiController.instanceCount;
  console.log(`[CTRL-${this.instanceNumber}] Controller created`);
  console.log(`[CTRL-${this.instanceNumber}] Total controllers: ${ShogiController.instanceCount}`);
}
```

**Expected if theory is correct**: 
```
[CTRL-1] Controller created
[CTRL-1] Total controllers: 1
[CTRL-2] Controller created
[CTRL-2] Total controllers: 2
```

---

## Solution Path

### If Multiple Controllers is Confirmed:

**Option 1: Fix React Strict Mode Guard** (attempted but failed)
- Set gameInitializedRef BEFORE async call ✓ (already done)
- Should prevent second initialization

**Option 2: Use Singleton Controller**
```typescript
// controller-singleton.ts
let instance: ShogiController | null = null;

export function getController(): ShogiController {
  if (!instance) {
    instance = new ShogiController();
  }
  return instance;
}
```

**Option 3: Disable React Strict Mode** (not recommended for production)
```typescript
// main.tsx
<React.StrictMode>  ← Remove this wrapper
  <App />
</React.StrictMode>
```

**Option 4: Use React Context**
```typescript
const ControllerContext = createContext<ShogiController | null>(null);

function App() {
  const controller = useMemo(() => new ShogiController(), []); // Only once
  return (
    <ControllerContext.Provider value={controller}>
      {/* ... */}
    </ControllerContext.Provider>
  );
}
```

---

## Immediate Next Steps

1. **Add controller instance tracking** to confirm theory
2. **If confirmed**: Implement Option 4 (React Context) or Option 2 (Singleton)
3. **Test thoroughly** with handicap games
4. **Remove all the extensive debug logging** once working

---

## Why This is Hard to Debug

1. **Async operations** - race conditions are hard to see
2. **React lifecycle** - Strict Mode behavior is unexpected
3. **Multiple components** - UI, Controller, Engine, Worker
4. **No TypeScript errors** - Everything type-checks fine
5. **Intermittent timing** - May work differently at different speeds

This is exactly the kind of bug that requires **systematic investigation** rather than quick fixes. 🎯

