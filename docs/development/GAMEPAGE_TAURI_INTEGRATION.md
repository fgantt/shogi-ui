# GamePage Tauri Integration Guide

This document provides step-by-step instructions for integrating the Tauri engine system with the existing GamePage component (Tasks 4.8-4.9).

## Overview

The GamePage currently uses a WASM-based worker system for engine communication. This needs to be replaced with Tauri-based process management using the new backend infrastructure.

## Reference Implementation

See `src/components/TauriGameDemo.tsx` for a complete working example of all patterns described below.

## Task 4.8: Add Engine Selector to GamePage

### Step 1: Import EngineSelector Component

```typescript
import { EngineSelector } from './EngineSelector';
import type { EngineConfig } from '../types/engine';
```

### Step 2: Add State for Engine Selection

```typescript
const [selectedEngineId, setSelectedEngineId] = useState<string | null>(null);
const [activeEngineId, setActiveEngineId] = useState<string | null>(null);
const [engineConfig, setEngineConfig] = useState<EngineConfig | null>(null);
```

### Step 3: Add EngineSelector to UI

Add this to the game setup section (before game starts):

```typescript
<EngineSelector
  selectedEngineId={selectedEngineId}
  onEngineSelect={setSelectedEngineId}
  label="AI Engine:"
  includeNone={true}  // Allow human-only games
/>
```

### Step 4: Load Engine Configuration

```typescript
useEffect(() => {
  if (!selectedEngineId) return;

  const loadEngine = async () => {
    const response = await invoke<CommandResponse<EngineConfig[]>>('get_engines');
    if (response.success && response.data) {
      const engine = response.data.find(e => e.id === selectedEngineId);
      setEngineConfig(engine || null);
    }
  };

  loadEngine();
}, [selectedEngineId]);
```

## Task 4.9: Update Game Logic for Tauri Communication

### Step 1: Import Tauri Utilities

```typescript
import { invoke } from '@tauri-apps/api/core';
import { useTauriEvents } from '../hooks/useTauriEvents';
import {
  sendUsiCommand,
  requestEngineMove,
  parseBestMove,
  parseEngineInfo,
  stopEngineThinking,
} from '../utils/tauriEngine';
```

### Step 2: Replace Worker with Tauri Engine

**Old pattern (WASM worker):**
```typescript
const worker = new Worker('./shogi.worker.ts');
worker.postMessage({ command: 'go', btime: 30000 });
worker.onmessage = (e) => {
  if (e.data.type === 'bestmove') {
    // Handle move
  }
};
```

**New pattern (Tauri engine):**
```typescript
// Spawn engine when game starts
const spawnResult = await invoke<CommandResponse>('spawn_engine', {
  engineId: selectedEngineId,
  name: engineConfig.name,
  path: engineConfig.path,
});

// Request move
await requestEngineMove(
  selectedEngineId,
  'startpos',
  moveHistory,
  { btime: 30000, wtime: 30000, byoyomi: 5000 }
);
```

### Step 3: Listen for Engine Responses

```typescript
useTauriEvents(activeEngineId, {
  onUsiMessage: (engineId, message) => {
    if (message.startsWith('bestmove')) {
      const { move } = parseBestMove(message);
      if (move && move !== 'resign') {
        applyEngineMove(move);
      }
    } else if (message.startsWith('info')) {
      const info = parseEngineInfo(message);
      updateSearchInfo(info);
    }
  },
  onUsiError: (engineId, error) => {
    console.error('Engine error:', error);
    handleEngineError(error);
  },
});
```

### Step 4: Cleanup on Unmount

```typescript
useEffect(() => {
  // ... initialization code ...

  return () => {
    if (activeEngineId) {
      invoke('stop_engine', { engineId: activeEngineId });
    }
  };
}, [activeEngineId]);
```

### Step 5: Replace TauriUsiMonitor

**Old:**
```typescript
<UsiMonitor
  lastSentCommand={lastSentCommand}
  lastReceivedCommand={lastReceivedCommand}
  communicationHistory={communicationHistory}
  sessions={sessions}
  isVisible={isUsiMonitorVisible}
  onToggle={onToggleUsiMonitor}
/>
```

**New:**
```typescript
<TauriUsiMonitor
  engineIds={activeEngineId ? [activeEngineId] : []}
  isVisible={isUsiMonitorVisible}
  onToggle={() => setIsUsiMonitorVisible(!isUsiMonitorVisible)}
  onSendCommand={(engineId, command) => sendUsiCommand(engineId, command)}
/>
```

## Key Differences: WASM vs Tauri

| Aspect | WASM (Old) | Tauri (New) |
|--------|-----------|-------------|
| Engine Location | In-browser worker | Separate process |
| Communication | `postMessage` / `onmessage` | `invoke()` / Tauri events |
| Performance | Limited by browser | Native performance |
| External Engines | Not possible | Fully supported |
| Debugging | Browser console only | stdout/stderr capture |
| Lifecycle | Worker thread | Process management |

## Testing Checklist

- [ ] Engine selector displays available engines
- [ ] Built-in engine can be selected and initialized
- [ ] Engine spawns successfully when game starts
- [ ] Position updates are sent correctly
- [ ] Engine returns valid moves
- [ ] Moves are applied to the game board
- [ ] Search info is displayed during thinking
- [ ] Engine can be stopped mid-search
- [ ] Engine is cleaned up when game ends
- [ ] USI monitor shows all communication
- [ ] Error handling works for engine failures
- [ ] Multiple games can be played in sequence

## Utility Functions Reference

### Available Functions (from `utils/tauriEngine.ts`)

- `spawnEngine(id, name, path)` - Start an engine process
- `sendUsiCommand(id, command)` - Send any USI command
- `stopEngine(id)` - Terminate an engine
- `getBuiltinEnginePath()` - Get path to bundled engine
- `initializeEngineSession(id, path, name)` - Full initialization
- `requestEngineMove(id, position, moves, timeControl)` - Request a move
- `stopEngineThinking(id)` - Stop search in progress
- `parseBestMove(message)` - Parse bestmove response
- `parseEngineInfo(message)` - Parse info messages

## Example: Complete Game Flow

```typescript
// 1. User selects engine
<EngineSelector onEngineSelect={setSelectedEngineId} />

// 2. Game starts - spawn engine
const engine = await getEngineConfig(selectedEngineId);
await invoke('spawn_engine', {
  engineId: selectedEngineId,
  name: engine.name,
  path: engine.path,
});

// 3. Initialize for new game
await sendUsiCommand(selectedEngineId, 'usinewgame');
await sendUsiCommand(selectedEngineId, 'isready');

// 4. Player makes a move
const newMoveHistory = [...moveHistory, userMove];
setMoveHistory(newMoveHistory);

// 5. Request engine response
await requestEngineMove(
  selectedEngineId,
  'startpos',
  newMoveHistory,
  { btime: playerTime, wtime: engineTime }
);

// 6. Engine responds via event
useTauriEvents(selectedEngineId, {
  onUsiMessage: (_, message) => {
    if (message.startsWith('bestmove')) {
      const { move } = parseBestMove(message);
      applyEngineMove(move);
    }
  }
});

// 7. Game ends - cleanup
await invoke('stop_engine', { engineId: selectedEngineId });
```

## Migration Strategy

1. **Phase 1**: Add EngineSelector alongside existing system
2. **Phase 2**: Create parallel Tauri code path (if/else based on flag)
3. **Phase 3**: Test thoroughly with both systems
4. **Phase 4**: Remove WASM worker code
5. **Phase 5**: Clean up unused imports and dependencies

## Notes

- The `TauriGameDemo` component at `/demo` shows a complete working implementation
- All existing game logic (board state, move validation, UI) can remain unchanged
- Only the engine communication layer needs to be replaced
- The Tauri system is more powerful and allows external engines

