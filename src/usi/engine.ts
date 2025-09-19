import { EventEmitter } from '../utils/events';

// An interface for any USI-compliant engine adapter
export interface EngineAdapter extends EventEmitter {
  init(): Promise<void>;
  isReady(): Promise<void>;
  setOptions(options: { [key: string]: string | number | boolean }): Promise<void>;
  setDifficulty?(difficulty: 'easy' | 'medium' | 'hard'): Promise<void>;
  newGame(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  go(options: { btime?: number; wtime?: number; byoyomi?: number; infinite?: boolean }): Promise<void>;
  stop(): Promise<void>;
  quit(): Promise<void>;
}

// An adapter for a WASM engine running in a Web Worker
export class WasmEngineAdapter extends EventEmitter implements EngineAdapter {
  private worker: Worker;

  constructor() {
    super();
    this.worker = new Worker(new URL('./shogi.worker.ts', import.meta.url), { type: 'module' });

    this.worker.onmessage = (e: MessageEvent) => {
      const message = e.data;
      if (typeof message === 'string') {
        this.processOutputLine(message);
      } else if (Array.isArray(message)) {
        message.forEach(line => this.processOutputLine(line));
      } else {
        // For now, log non-string messages
        console.log("Received non-string message from worker:", message);
      }
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
          this.postCommand(`setoption name ${name} value ${value}`);
      }
  }

  async setDifficulty(difficulty: 'easy' | 'medium' | 'hard'): Promise<void> {
    console.log('WasmEngineAdapter: Setting difficulty...', difficulty);
    const difficultyMap = {
      'easy': 3,
      'medium': 5, 
      'hard': 8
    };
    this.postCommand(`setoption name difficulty value ${difficultyMap[difficulty]}`);
  }

  async newGame(): Promise<void> {
    console.log('WasmEngineAdapter: Sending usinewgame command...');
    this.postCommand('usinewgame');
  }

  async setPosition(sfen: string, moves: string[]): Promise<void> {
    const movesString = moves.length > 0 ? `moves ${moves.join(' ')}` : '';
    this.postCommand(`position sfen ${sfen} ${movesString}`);
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

  private processOutputLine(line: string): void {
    console.log('WasmEngineAdapter: Processing output:', line);

    // Emit event for received command
    this.emit('usiCommandReceived', { command: line });

    if (line.startsWith('bestmove ')) {
      const move = line.substring(9).trim();
      this.emit('bestmove', { move });
    } else if (line.startsWith('info ')) {
      // Parse info line and emit appropriate events
      const infoContent = line.substring(5);
      this.emit('info', { content: infoContent });
    } else if (line === 'readyok') {
      this.emit('readyok');
    } else if (line === 'usiok') {
      this.emit('usiok');
    }
  }
}