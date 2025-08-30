import { isWasmEngineAvailable, initializeWasmEngine } from './wasmEngine';

export async function initializeWasm(): Promise<void> {
  if (!isWasmEngineAvailable()) {
    try {
      await initializeWasmEngine();
    } catch (error) {
      console.error("Failed to initialize wasm engine on startup", error);
    }
  }
}