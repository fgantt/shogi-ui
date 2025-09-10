import { WasmUsiEngineAdapter } from '../usi/engine';

let engineAdapter: WasmUsiEngineAdapter | null = null;

export async function initializeWasm(): Promise<void> {
  if (!engineAdapter) {
    try {
      engineAdapter = new WasmUsiEngineAdapter();
      await engineAdapter.init();
      await engineAdapter.isReady();
      console.log('WasmUsiEngineAdapter initialized successfully');
    } catch (error) {
      console.error("Failed to initialize wasm engine on startup", error);
      throw error;
    }
  }
}

export function getEngineAdapter(): WasmUsiEngineAdapter {
  if (!engineAdapter) {
    throw new Error('Engine adapter not initialized. Call initializeWasm() first.');
  }
  return engineAdapter;
}