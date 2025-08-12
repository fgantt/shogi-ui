# WebAssembly Integration Guide

This guide explains how to integrate and use the WebAssembly engine in your Shogi application.

## Quick Start

### 1. Build the WebAssembly Module

```bash
# Make sure you have Rust and wasm-pack installed
cargo install wasm-pack

# Build the WebAssembly module
./build.sh
```

### 2. Import and Use

```javascript
import { getWasmAiMove } from './src/ai/wasmEngine.js';

// Get AI move
const move = await getWasmAiMove(gameState, 'medium');
```

## Integration Options

### Option 1: Direct WebAssembly Usage

Use the WebAssembly engine directly for maximum performance:

```javascript
import { 
  getWasmAiMove, 
  isWasmEngineAvailable,
  getEngineStats 
} from './src/ai/wasmEngine.js';

class ShogiGame {
  async getAiMove(difficulty) {
    if (isWasmEngineAvailable()) {
      try {
        return await getWasmAiMove(this.gameState, difficulty);
      } catch (error) {
        console.warn('WebAssembly failed, using fallback');
        return this.getFallbackMove(difficulty);
      }
    }
    return this.getFallbackMove(difficulty);
  }
}
```

### Option 2: Automatic Fallback (Recommended)

Use the integrated computer player that automatically handles fallback:

```javascript
import { getAiMove } from './src/ai/computerPlayer.js';

class ShogiGame {
  async getAiMove(difficulty) {
    // Automatically uses WebAssembly if available, falls back to JavaScript
    return await getAiMove(this.gameState, difficulty);
  }
}
```

### Option 3: Performance Comparison

Compare both engines to see the performance difference:

```javascript
import { compareEnginePerformance } from './src/ai/computerPlayer.js';

async function benchmarkEngines(gameState, difficulty) {
  const results = await compareEnginePerformance(gameState, difficulty);
  
  console.log('WebAssembly performance:', results.wasm);
  console.log('JavaScript performance:', results.javascript);
  
  if (results.wasm && results.javascript) {
    const improvement = results.javascript.executionTime / results.wasm.executionTime;
    console.log(`WebAssembly is ${improvement.toFixed(1)}x faster`);
  }
}
```

## API Reference

### Core Functions

#### `getWasmAiMove(gameState, difficulty)`
- **Parameters:**
  - `gameState`: Current game state object
  - `difficulty`: 'easy', 'medium', or 'hard'
- **Returns:** Promise resolving to move object
- **Throws:** Error if WebAssembly engine fails

#### `isWasmEngineAvailable()`
- **Returns:** Boolean indicating if WebAssembly is ready
- **Use:** Check availability before calling WebAssembly functions

#### `getEngineStats()`
- **Returns:** Object with engine status and features
- **Use:** Get information about the current engine state

### Engine Control

#### `forceJavaScriptEngine()`
- **Use:** Force use of JavaScript engine (for testing/fallback)

#### `enableWasmEngine()`
- **Use:** Re-enable WebAssembly engine

#### `getEngineStatus()`
- **Returns:** Current engine configuration and status

### Performance Monitoring

#### `getPerformanceMetrics(gameState, difficulty)`
- **Returns:** Detailed performance data including execution time and memory usage

#### `benchmarkEngines(gameState, difficulty)`
- **Returns:** Performance comparison between engines

## Game State Format

The WebAssembly engine expects the game state in this format:

```javascript
const gameState = {
  board: [
    // 9x9 array representing the board
    // Each cell contains: { type: 'P', player: 'player1' } or null
  ],
  currentPlayer: 'player1', // or 'player2'
  capturedPieces: {
    player1: [], // Array of captured pieces
    player2: []
  },
  moveHistory: [], // Array of previous moves
  isCheck: false,
  isCheckmate: false
};
```

### Piece Types

- `'K'` - King
- `'R'` - Rook
- `'B'` - Bishop
- `'G'` - Gold
- `'S'` - Silver
- `'N'` - Knight
- `'L'` - Lance
- `'P'` - Pawn
- `'+P'` - Promoted Pawn
- `'+L'` - Promoted Lance
- `'+N'` - Promoted Knight
- `'+S'` - Promoted Silver
- `'+B'` - Promoted Bishop
- `'+R'` - Promoted Rook

## Difficulty Levels

### Easy (1 second time limit)
- Search depth: 3-4 ply
- Good for beginners
- Fast response time

### Medium (3 seconds time limit)
- Search depth: 5-6 ply
- Balanced performance
- Recommended for most games

### Hard (9 seconds time limit)
- Search depth: 7-8 ply
- Strong play
- Best for experienced players

## Error Handling

### WebAssembly Initialization Failures

```javascript
try {
  const move = await getWasmAiMove(gameState, 'medium');
} catch (error) {
  if (error.message.includes('WebAssembly')) {
    // WebAssembly failed, use JavaScript fallback
    const fallbackMove = await getJavaScriptMove(gameState, 'medium');
    return fallbackMove;
  } else {
    // Other error, handle appropriately
    throw error;
  }
}
```

### Automatic Fallback

The integrated computer player automatically handles fallback:

```javascript
import { getAiMove } from './src/ai/computerPlayer.js';

// This will automatically use JavaScript if WebAssembly fails
const move = await getAiMove(gameState, 'medium');
```

## Performance Optimization

### 1. Engine Reuse

For multiple moves in the same game, the engine maintains state:

```javascript
// The engine automatically reuses the same instance
const move1 = await getWasmAiMove(gameState, 'medium');
// ... make move ...
const move2 = await getWasmAiMove(newGameState, 'medium');
```

### 2. Memory Management

The WebAssembly engine is optimized for memory usage:

```javascript
// Monitor memory usage
const metrics = await getPerformanceMetrics(gameState, 'medium');
console.log('Memory used:', metrics.memoryUsed);
```

### 3. Search Depth Optimization

Choose appropriate difficulty for your use case:

```javascript
// For real-time games, use easy/medium
const move = await getWasmAiMove(gameState, 'easy');

// For analysis, use hard
const analysisMove = await getWasmAiMove(gameState, 'hard');
```

## Testing and Debugging

### Enable Debug Logging

```javascript
// Set debug flag
localStorage.setItem('shogi_debug', 'true');

// Check console for detailed logs
```

### Test Engine Status

```javascript
import { getEngineStatus } from './src/ai/computerPlayer.js';

const status = getEngineStatus();
console.log('Current engine:', status.currentEngine);
console.log('WebAssembly available:', status.wasmAvailable);
```

### Performance Testing

```javascript
import { runAllTests } from './src/ai/wasmEngine.test.js';

// Run comprehensive tests
const allTestsPassed = await runAllTests();
console.log('All tests passed:', allTestsPassed);
```

## Browser Compatibility

### Supported Browsers

- **Chrome**: 57+ (full support)
- **Firefox**: 52+ (full support)
- **Safari**: 11+ (full support)
- **Edge**: 79+ (full support)

### Feature Detection

```javascript
if (typeof WebAssembly === 'undefined') {
  console.warn('WebAssembly not supported, using JavaScript engine');
  // Force JavaScript engine
  forceJavaScriptEngine();
}
```

## Troubleshooting

### Common Issues

1. **WebAssembly not loading**
   - Check if `wasm-pack build` completed successfully
   - Verify file paths in import statements
   - Check browser console for errors

2. **Performance not improving**
   - Ensure WebAssembly is actually being used
   - Check if JavaScript fallback is triggered
   - Verify build optimization flags

3. **Memory issues**
   - Monitor memory usage in browser dev tools
   - Check for memory leaks in move generation
   - Verify bitboard operations are efficient

### Debug Commands

```javascript
// Check engine capabilities
const capabilities = getEngineCapabilities();
console.log('Engine capabilities:', capabilities);

// Reset engine state
import { resetEngine } from './src/ai/wasmEngine.js';
resetEngine();

// Force JavaScript mode
import { forceJavaScriptEngine } from './src/ai/computerPlayer.js';
forceJavaScriptEngine();
```

## Advanced Usage

### Custom Time Limits

```javascript
// Override default time limits
const customTimeLimit = 5000; // 5 seconds
const engine = ShogiEngine.new();
engine.set_time_limit(customTimeLimit);
```

### Engine Configuration

```javascript
// Configure engine parameters
const engine = ShogiEngine.new();
engine.set_search_depth(10);
engine.enable_transposition_table(true);
engine.enable_killer_moves(true);
```

### Batch Processing

```javascript
// Process multiple positions
const positions = [gameState1, gameState2, gameState3];
const moves = await Promise.all(
  positions.map(state => getWasmAiMove(state, 'medium'))
);
```

## Integration Examples

### React Component

```jsx
import React, { useState, useEffect } from 'react';
import { getWasmAiMove } from './ai/wasmEngine.js';

function ShogiGame() {
  const [gameState, setGameState] = useState(initialState);
  const [isThinking, setIsThinking] = useState(false);

  const getAiMove = async (difficulty) => {
    setIsThinking(true);
    try {
      const move = await getWasmAiMove(gameState, difficulty);
      // Apply move to game state
      setGameState(applyMove(gameState, move));
    } catch (error) {
      console.error('AI move failed:', error);
    } finally {
      setIsThinking(false);
    }
  };

  return (
    <div>
      <button onClick={() => getAiMove('medium')} disabled={isThinking}>
        {isThinking ? 'Thinking...' : 'AI Move'}
      </button>
      {/* Game board and controls */}
    </div>
  );
}
```

### Vue Component

```vue
<template>
  <div>
    <button @click="getAiMove('medium')" :disabled="isThinking">
      {{ isThinking ? 'Thinking...' : 'AI Move' }}
    </button>
  </div>
</template>

<script>
import { getWasmAiMove } from './ai/wasmEngine.js';

export default {
  data() {
    return {
      gameState: initialState,
      isThinking: false
    };
  },
  methods: {
    async getAiMove(difficulty) {
      this.isThinking = true;
      try {
        const move = await getWasmAiMove(this.gameState, difficulty);
        // Apply move to game state
        this.gameState = this.applyMove(this.gameState, move);
      } catch (error) {
        console.error('AI move failed:', error);
      } finally {
        this.isThinking = false;
      }
    }
  }
};
</script>
```

### Vanilla JavaScript

```javascript
class ShogiGame {
  constructor() {
    this.gameState = getInitialGameState();
    this.isThinking = false;
  }

  async getAiMove(difficulty) {
    if (this.isThinking) return;
    
    this.isThinking = true;
    try {
      const move = await getWasmAiMove(this.gameState, difficulty);
      this.applyMove(move);
    } catch (error) {
      console.error('AI move failed:', error);
    } finally {
      this.isThinking = false;
    }
  }

  applyMove(move) {
    // Apply the move to the game state
    // Update UI, etc.
  }
}
```

## Performance Benchmarks

### Expected Performance

| Difficulty | JavaScript | WebAssembly | Improvement |
|------------|------------|-------------|-------------|
| Easy       | 100-200ms  | 20-40ms     | 5x faster   |
| Medium     | 500-1000ms | 100-200ms   | 5x faster   |
| Hard       | 2000-5000ms| 400-1000ms  | 5x faster   |

### Memory Usage

- **JavaScript**: 50-100MB for complex positions
- **WebAssembly**: 10-20MB for same positions
- **Improvement**: 3-5x reduction in memory usage

## Conclusion

The WebAssembly integration provides significant performance improvements while maintaining compatibility with existing code. The automatic fallback ensures your application continues to work even if WebAssembly fails.

For best results:
1. Use the integrated computer player for automatic fallback
2. Monitor performance metrics to ensure WebAssembly is being used
3. Handle errors gracefully with fallback options
4. Test thoroughly in your target browsers

The WebAssembly engine represents a major upgrade to the AI system and should provide noticeably stronger play and faster response times.
