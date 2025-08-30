import { EventEmitter } from 'events';

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
  private isReadyPromise: Promise<void>;

  constructor(workerPath: string) {
    super();
    this.worker = new Worker(new URL(workerPath, import.meta.url), { type: 'module' });
    
    this.isReadyPromise = new Promise(resolve => {
        const readyHandler = (e: MessageEvent) => {
            if (e.data.command === 'readyok') {
                this.worker.removeEventListener('message', readyHandler);
                resolve();
            }
        };
        this.worker.addEventListener('message', readyHandler);
    });

    this.worker.onmessage = (e: MessageEvent) => {
      const { command, ...args } = e.data;
      this.emit(command, args);
    };
  }

  private postCommand(command: string, args: Record<string, unknown> = {}): void {
    this.worker.postMessage({ command, ...args });
  }

  async init(): Promise<void> {
    this.postCommand('usi');
    return new Promise(resolve => {
        const handler = (e: MessageEvent) => {
            if (e.data.command === 'usiok') {
                this.worker.removeEventListener('message', handler);
                resolve();
            }
        };
        this.worker.addEventListener('message', handler);
    });
  }
  
  async isReady(): Promise<void> {
      this.postCommand('isready');
      return this.isReadyPromise;
  }

  async setOptions(options: { [key: string]: string | number | boolean }): Promise<void> {
      for (const [name, value] of Object.entries(options)) {
          this.postCommand('setoption', { name, value });
      }
  }

  async newGame(): Promise<void> {
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
    this.postCommand('quit');
    this.worker.terminate();
  }
}
