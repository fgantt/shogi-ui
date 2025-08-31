import { WasmEngineAdapter } from './engine';
import { vi, describe, it, expect, beforeEach, afterEach } from 'vitest';

const postMessage = vi.fn();
const terminate = vi.fn();
let onmessage: (e: any) => void = () => {};

const MockWorker = vi.fn(() => ({
  postMessage,
  terminate,
  addEventListener: vi.fn(),
  removeEventListener: vi.fn(),
  set onmessage(handler: (e: any) => void) {
    onmessage = handler;
  },
  get onmessage() {
    return onmessage;
  }
}));

vi.stubGlobal('Worker', MockWorker);

describe('WasmEngineAdapter', () => {
  let engine: WasmEngineAdapter;

  beforeEach(() => {
    vi.clearAllMocks();
    engine = new WasmEngineAdapter('./mock.worker.ts');
  });

  afterEach(() => {
    engine.quit();
  });

  it('should initialize and receive usiok', async () => {
    const p = new Promise<void>(res => engine.on('usiok', res));
    engine.init();
    expect(postMessage).toHaveBeenCalledWith({ command: 'usi' });
    // simulate worker response
    onmessage({ data: { command: 'usiok' } });
    await p;
  });

  it('should set position and get a bestmove response', async () => {
    const p = new Promise<any>(res => engine.on('bestmove', res));
    const sfen = 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1';
    await engine.setPosition(sfen, []);
    await engine.go({ btime: 1000, wtime: 1000, byoyomi: 500 });
    expect(postMessage).toHaveBeenCalledWith({ command: 'position', position: `sfen ${sfen} ` });
    expect(postMessage).toHaveBeenCalledWith({ command: 'go', btime: 1000, wtime: 1000, byoyomi: 500 });
    // simulate worker response
    onmessage({ data: { command: 'bestmove', move: '7g7f' } });
    const { move } = await p;
    expect(move).toBe('7g7f');
  });
});