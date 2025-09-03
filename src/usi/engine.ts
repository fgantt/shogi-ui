import { EventEmitter } from '../utils/events';

// An interface for any USI-compliant engine adapter
export interface EngineAdapter extends EventEmitter {
  init(): Promise<void>;
  isReady(): Promise<void>;
  setOptions(options: { [key: string]: string | number | boolean }): Promise<void>;
  newGame(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  go(options: { btime?: number; wtime?: number; byoyomi?: number; infinite?: boolean }): Promise<void>;
  stop(): Promise<void>;
  quit(): Promise<void>;
}

// An adapter for a WASM engine running in a Web Worker
export class WasmEngineAdapter extends EventEmitter implements EngineAdapter {
  private worker: Worker;

  constructor(workerPath: string) {
    super();
    this.worker = new Worker(new URL(workerPath, import.meta.url), { type: 'module' });

    this.worker.onmessage = (e: MessageEvent) => {
      const { command, ...args } = e.data;
      this.emit(command, args);
    };

    this.worker.onerror = (e: ErrorEvent) => {
      console.error('AI Worker Error:', e.message, e.filename, e.lineno);
    };
  }

  private postCommand(command: string, args: Record<string, unknown> = {}): void {
    this.worker.postMessage({ command, ...args });
  }

  async init(): Promise<void> {
    console.log('WasmEngineAdapter: Sending usi command...');
    this.postCommand('usi');
    return new Promise(resolve => {
      this.once('usiok', () => {
        console.log('WasmEngineAdapter: Received usiok.');
        resolve();
      });
    });
  }

  async isReady(): Promise<void> {
    console.log('WasmEngineAdapter: Sending isready command...');
    this.postCommand('isready');
    return new Promise(resolve => {
      this.once('readyok', () => {
        console.log('WasmEngineAdapter: Received readyok.');
        resolve();
      });
    });
  }

  async setOptions(options: { [key: string]: string | number | boolean }): Promise<void> {
      console.log('WasmEngineAdapter: Setting options...', options);
      for (const [name, value] of Object.entries(options)) {
          this.postCommand('setoption', { name, value });
      }
  }

  async newGame(): Promise<void> {
    console.log('WasmEngineAdapter: Sending usinewgame command...');
    this.postCommand('usinewgame');
  }

  async setPosition(sfen: string, moves: string[]): Promise<void> {
    const movesString = moves.length > 0 ? `moves ${moves.join(' ')}` : '';
    this.postCommand('position', { position: `sfen ${sfen} ${movesString}` });
  }

  async go(options: { btime?: number; wtime?: number; byoyomi?: number; infinite?: boolean }): Promise<void> {
    this.postCommand('go', options);
  }

  async stop(): Promise<void> {
    this.postCommand('stop');
  }

  async quit(): Promise<void> {
    console.log('WasmEngineAdapter: quit() called. Terminating worker.');
    this.postCommand('quit');
    this.worker.terminate();
  }
}
