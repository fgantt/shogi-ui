
// src/usi/engine.test.ts
import { WasmEngineAdapter } from './engine';
import { vi } from 'vitest';

// We assume the mock worker is correctly set up to be loaded by the test environment.
// In Vite/Jest, this might require some configuration (e.g., vite.config.ts plugins).
const MOCK_WORKER_PATH = './mock.worker.ts';

describe('WasmEngineAdapter', () => {
  let engine: WasmEngineAdapter;

  beforeEach(() => {
    engine = new WasmEngineAdapter(MOCK_WORKER_PATH);
  });

  afterEach(() => {
    engine.quit();
  });

  it('should initialize and receive usiok', async () => {
    const usiOkPromise = new Promise<void>(resolve => {
        engine.on('usiok', resolve);
    });
    engine.init();
    await expect(usiOkPromise).resolves.toBeUndefined();
  });

  it('should set position and get a bestmove response', async () => {
    await engine.init();
    await engine.isReady();
    
    const bestMovePromise = new Promise<string>(resolve => {
      engine.on('bestmove', ({ move }) => resolve(move));
    });

    await engine.setPosition('lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1', []);
    await engine.go({ infinite: true });

    const bestMove = await bestMovePromise;
    expect(typeof bestMove).toBe('string');
    expect(bestMove.length).toBeGreaterThan(3);
  });
});
