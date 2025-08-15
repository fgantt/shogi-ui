import { getWasmAiMove, isWasmEngineAvailable, getPerformanceMetrics, initializeWasmEngine } from './wasmEngine.js';
import type { GameState } from '../types';

// Define a type for the AI worker to make it more predictable
interface AiWorker extends Worker {
  resolve?: (value: any) => void;
  reject?: (reason?: any) => void;
}

let aiWorker: AiWorker;
let useWasmEngine = true;

export async function initializeWasm(): Promise<void> {
  if (useWasmEngine && !isWasmEngineAvailable()) {
    try {
      await initializeWasmEngine();
    } catch (error) {
      console.error("Failed to initialize wasm engine on startup", error);
      useWasmEngine = false;
    }
  }
}

function initializeWorker(): void {
  aiWorker = new Worker(new URL('./ai.worker.js', import.meta.url), { type: 'module' });

  aiWorker.onmessage = (event: MessageEvent) => {
    // The worker will send back the best move, which we will then resolve.
    if (aiWorker.resolve) {
      aiWorker.resolve(event.data);
    }
  };

  aiWorker.onerror = (error: ErrorEvent) => {
    console.error("AI Worker Error:", error);
    if (aiWorker.reject) {
      aiWorker.reject(error);
    }
  };
}

/**
 * Get AI move with WebAssembly engine fallback to JavaScript
 */
export async function getAiMove(gameState: GameState, difficulty: number): Promise<any> {
  // Try WebAssembly engine first
  if (useWasmEngine && isWasmEngineAvailable()) {
    try {
      console.log('Using WebAssembly engine...');
      const startTime = performance.now();
      
      const move = await getWasmAiMove(gameState, difficulty);
      
      const executionTime = performance.now() - startTime;
      console.log(`WebAssembly engine completed in ${executionTime.toFixed(2)}ms`);
      
      return move;
    } catch (error: any) {
      console.warn('WebAssembly engine failed, falling back to JavaScript:', error.message);
      useWasmEngine = false; // Disable WebAssembly for this session
    }
  }

  // Fallback to JavaScript engine
  console.log('Using JavaScript engine...');
  return getJavaScriptAiMove(gameState, difficulty);
}

/**
 * Get AI move using the JavaScript engine (original implementation)
 */
function getJavaScriptAiMove(gameState: GameState, difficulty: number): Promise<any> {
  return new Promise((resolve, reject) => {
    if (!aiWorker) {
      initializeWorker();
    }

    aiWorker.resolve = resolve;
    aiWorker.reject = reject;

    aiWorker.postMessage({ gameState, difficulty });
  });
}

/**
 * Force use of JavaScript engine (for testing/fallback)
 */
export function forceJavaScriptEngine(): void {
  useWasmEngine = false;
  console.log('Forced JavaScript engine mode');
}

/**
 * Re-enable WebAssembly engine
 */
export function enableWasmEngine(): void {
  useWasmEngine = true;
  console.log('WebAssembly engine re-enabled');
}

/**
 * Get current engine status
 */
export function getEngineStatus(): { wasmAvailable: boolean; wasmEnabled: boolean; currentEngine: string } {
  return {
    wasmAvailable: isWasmEngineAvailable(),
    wasmEnabled: useWasmEngine,
    currentEngine: useWasmEngine && isWasmEngineAvailable() ? 'WebAssembly' : 'JavaScript'
  };
}

/**
 * Get performance comparison between engines
 */
export async function compareEnginePerformance(gameState: GameState, difficulty: number): Promise<any> {
  const results: any = {};
  
  // Test WebAssembly engine if available
  if (isWasmEngineAvailable()) {
    try {
      const wasmMetrics = await getPerformanceMetrics(gameState, difficulty);
      results.wasm = wasmMetrics;
    } catch (error: any) {
      results.wasm = { error: error.message };
    }
  }
  
  // Test JavaScript engine
  try {
    const jsStart = performance.now();
    const jsStartMemory = (performance as any).memory?.usedJSHeapSize || 0;
    
    const jsMove = await getJavaScriptAiMove(gameState, difficulty);
    
    const jsTime = performance.now() - jsStart;
    const jsMemory = (performance as any).memory?.usedJSHeapSize || 0;
    
    results.javascript = {
      move: jsMove,
      executionTime: jsTime,
      memoryUsed: jsMemory - jsStartMemory,
      engineType: 'JavaScript',
      difficulty
    };
  } catch (error: any) {
    results.javascript = { error: error.message };
  }
  
  return results;
}

/**
 * Get engine recommendations for different difficulties
 */
export function getEngineRecommendations(): any {
  return {
    easy: {
      recommended: 'WebAssembly',
      reason: 'Fast move generation and evaluation'
    },
    medium: {
      recommended: 'WebAssembly', 
      reason: 'Better search depth in same time'
    },
    hard: {
      recommended: 'WebAssembly',
      reason: 'Significant performance improvement for deep search'
    }
  };
}
