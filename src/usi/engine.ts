import { EventEmitter } from '../utils/events';
import { createWasmUsiHandler, initializeWasmEngine } from '../ai/wasmEngine';

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

// A new adapter that uses WasmUsiHandler directly (no worker)
export class WasmUsiEngineAdapter extends EventEmitter implements EngineAdapter {
  private handler: any; // WasmUsiHandler
  private isSearching = false;
  private searchTimeout: NodeJS.Timeout | null = null;

  constructor() {
    super();
    // Don't create handler here - wait for init()
  }

  async init(): Promise<void> {
    console.log('WasmUsiEngineAdapter: Initializing WASM module...');
    await initializeWasmEngine();
    console.log('WasmUsiEngineAdapter: WASM module initialized. Creating handler...');
    this.handler = createWasmUsiHandler();
    
    console.log('WasmUsiEngineAdapter: Sending usi command...');
    const command = 'usi';
    this.emit('usiCommandSent', { command });
    const output = this.handler.process_command(command);
    console.log('WasmUsiEngineAdapter: USI response:', output);
    
    // Process any pending output
    this.processPendingOutput();
  }

  async isReady(): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    console.log('WasmUsiEngineAdapter: Sending isready command...');
    const command = 'isready';
    this.emit('usiCommandSent', { command });
    const output = this.handler.process_command(command);
    console.log('WasmUsiEngineAdapter: isready response:', output);
    
    // Process any pending output
    this.processPendingOutput();
  }

  async setOptions(options: { [key: string]: string | number | boolean }): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    console.log('WasmUsiEngineAdapter: Setting options...', options);
    for (const [name, value] of Object.entries(options)) {
      const command = `setoption name ${name} value ${value}`;
      this.emit('usiCommandSent', { command });
      this.handler.process_command(command);
    }
    this.processPendingOutput();
  }

  async newGame(): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    console.log('WasmUsiEngineAdapter: Sending usinewgame command...');
    const command = 'usinewgame';
    this.emit('usiCommandSent', { command });
    this.handler.process_command(command);
    this.processPendingOutput();
  }

  async setPosition(sfen: string, moves: string[]): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    const movesString = moves.length > 0 ? `moves ${moves.join(' ')}` : '';
    const command = `position sfen ${sfen} ${movesString}`;
    console.log('WasmUsiEngineAdapter: Setting position:', command);
    this.emit('usiCommandSent', { command });
    this.handler.process_command(command);
    this.processPendingOutput();
  }

  async setDifficulty(difficulty: 'easy' | 'medium' | 'hard'): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    // Convert difficulty string to number
    const difficultyMap = {
      'easy': 3,
      'medium': 5, 
      'hard': 8
    };
    
    const difficultyNumber = difficultyMap[difficulty];
    this.handler.set_difficulty(difficultyNumber);
    
    console.log('WasmUsiEngineAdapter: Set difficulty to', difficulty, '(', difficultyNumber, ')');
  }

  async go(options: { btime?: number; wtime?: number; byoyomi?: number; infinite?: boolean }): Promise<void> {
    if (!this.handler) {
      throw new Error('Handler not initialized. Call init() first.');
    }
    
    if (this.isSearching) {
      console.log('WasmUsiEngineAdapter: Already searching, ignoring go command');
      return;
    }

    this.isSearching = true;
    const goCommand = this.buildGoCommand(options);
    console.log('WasmUsiEngineAdapter: Starting search with command:', goCommand);
    this.emit('usiCommandSent', { command: goCommand });
    
    // Since the search is now synchronous in WASM, we can process it directly
    // Use setTimeout to avoid blocking the main thread
    this.searchTimeout = setTimeout(() => {
      try {
        this.handler.process_command(goCommand);
        // Process output after successful command
        this.processPendingOutput();
        // Process output again after a small delay to catch any remaining output
        setTimeout(() => {
          this.processPendingOutput();
        }, 10);
      } catch (error) {
        console.error('WasmUsiEngineAdapter: Error in process_command:', error);
        // Don't process output on error to avoid circular reference
      } finally {
        this.isSearching = false;
      }
    }, 0);
  }

  async stop(): Promise<void> {
    if (this.searchTimeout) {
      clearTimeout(this.searchTimeout);
      this.searchTimeout = null;
    }
    
    if (this.isSearching && this.handler) {
      console.log('WasmUsiEngineAdapter: Stopping search...');
      const command = 'stop';
      this.emit('usiCommandSent', { command });
      this.handler.process_command(command);
      this.isSearching = false;
      this.processPendingOutput();
    }
  }

  async quit(): Promise<void> {
    console.log('WasmUsiEngineAdapter: quit() called.');
    await this.stop();
    if (this.handler) {
      const command = 'quit';
      this.emit('usiCommandSent', { command });
      this.handler.process_command(command);
    }
  }

  private buildGoCommand(options: { btime?: number; wtime?: number; byoyomi?: number; infinite?: boolean }): string {
    const parts: string[] = ['go'];
    
    if (options.btime !== undefined) parts.push(`btime ${options.btime}`);
    if (options.wtime !== undefined) parts.push(`wtime ${options.wtime}`);
    if (options.byoyomi !== undefined) parts.push(`byoyomi ${options.byoyomi}`);
    if (options.infinite) parts.push('infinite');
    
    return parts.join(' ');
  }

  private processPendingOutput(): void {
    if (!this.handler) {
      return;
    }
    
    const output = this.handler.get_pending_output();
    if (output && output.length > 0) {
      for (const line of output) {
        this.processOutputLine(line);
      }
    }
  }

  private processOutputLine(line: string): void {
    console.log('WasmUsiEngineAdapter: Processing output:', line);
    
    // Emit event for received command
    this.emit('usiCommandReceived', { command: line });
    
    if (line.startsWith('bestmove ')) {
      const move = line.substring(9).trim();
      this.isSearching = false;
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
