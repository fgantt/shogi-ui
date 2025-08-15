import { getWasmAiMove, isWasmEngineAvailable, getPerformanceMetrics, initializeWasmEngine } from './wasmEngine.js';
import type { GameState } from '../types';

// Define a type for the AI worker to make it more predictable
interface AiWorker extends Worker {
  resolve?: (value: any) => void;
  reject?: (reason?: any) => void;
}

let aiWorker: AiWorker;


export async function initializeWasm(): Promise<void> {
  if (!isWasmEngineAvailable()) {
    try {
      await initializeWasmEngine();
    } catch (error) {
      console.error("Failed to initialize wasm engine on startup", error);
    }
  }
}

function initializeWorker(): void {
  aiWorker = new Worker(new URL('./ai.worker.ts', import.meta.url), { type: 'module' });

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
export async function getAiMove(gameState: GameState, difficulty: number, engineType: 'ai-js' | 'ai-wasm'): Promise<any> {
  if (engineType === 'ai-wasm') {
    // Try WebAssembly engine
    if (isWasmEngineAvailable()) {
      try {
        console.log('Using WebAssembly engine...');
        const startTime = performance.now();
        
        const move = await getWasmAiMove(gameState, difficulty);
        
        const executionTime = performance.now() - startTime;
        console.log(`WebAssembly engine completed in ${executionTime.toFixed(2)}ms`);
        
        return move;
      } catch (error: any) {
        console.error('WebAssembly engine failed:', error.message);
        throw new Error('WASM engine failed'); // Re-throw to be caught by caller
      }
    } else {
      console.warn('WebAssembly engine not available, falling back to JavaScript.');
      return getJavaScriptAiMove(gameState, difficulty);
    }
  } else if (engineType === 'ai-js') {
    console.log('Using JavaScript engine...');
    return getJavaScriptAiMove(gameState, difficulty);
  } else {
    // Default to JavaScript if an unknown engineType is passed
    console.warn(`Unknown engine type: ${engineType}. Defaulting to JavaScript engine.`);
    return getJavaScriptAiMove(gameState, difficulty);
  }
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










