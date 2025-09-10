import { WasmUsiHandler } from '../../pkg-bundler/shogi_engine.js';

let wasmModule: typeof import('../../pkg-bundler/shogi_engine.js') | null = null;
let isInitialized = false;

/**
 * Initialize the WebAssembly engine
 */
export async function initializeWasmEngine(): Promise<void> {
    if (isInitialized) {
        return;
    }
    
    try {
        wasmModule = await import('../../pkg-bundler/shogi_engine.js');
        isInitialized = true;
    } catch (error) {
        console.error('Failed to initialize WebAssembly engine:', error);
        throw error;
    }
}

/**
 * Check if WebAssembly engine is available
 */
export function isWasmEngineAvailable(): boolean {
    return isInitialized && wasmModule !== null;
}

/**
 * Create a new WasmUsiHandler instance
 */
export function createWasmUsiHandler(): WasmUsiHandler {
    if (!wasmModule) {
        throw new Error('WebAssembly module not initialized. Call initializeWasmEngine() first.');
    }
    return new wasmModule.WasmUsiHandler();
}

/**
 * Reset the engine state
 */
export function resetEngine(): void {
    isInitialized = false;
    wasmModule = null;
}