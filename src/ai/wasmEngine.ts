import { ShogiEngine, PieceType as WasmPieceType } from '../../pkg-bundler/shogi_engine.js';

let isInitialized = false;

/**
 * Initialize the WebAssembly engine
 */
export async function initializeWasmEngine(): Promise<void> {
    if (isInitialized) {
        return;
    }
    
    try {
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
    return isInitialized;
}

/**
 * Reset the engine state
 */
export function resetEngine(): void {
    isInitialized = false;
}