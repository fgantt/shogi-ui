import { WasmEngineAdapter } from '../usi/engine';

let engineAdapter: WasmEngineAdapter | null = null;

export async function initializeWasm(): Promise<void> {
  if (!engineAdapter) {
    try {
      engineAdapter = new WasmEngineAdapter();
      await engineAdapter.init();
      await engineAdapter.isReady();
      console.log('WasmEngineAdapter initialized successfully');
    } catch (error) {
      console.error("Failed to initialize wasm engine on startup", error);
      throw error;
    }
  }
}

export function getEngineAdapter(): WasmEngineAdapter {
  if (!engineAdapter) {
    throw new Error('Engine adapter not initialized. Call initializeWasm() first.');
  }
  return engineAdapter;
}