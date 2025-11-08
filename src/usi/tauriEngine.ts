/**
 * Tauri-based engine adapter
 * Provides a bridge between the existing USI controller and Tauri backend
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { CommandResponse } from '../types/engine';
import { EventEmitter } from '../utils/events';

export interface EngineAdapter {
  sessionId: string;
  initialize(): Promise<void>;
  sendCommand(command: string): Promise<void>;
  destroy(): Promise<void>;
  setSearchDepth(depth: number): void;
  on(event: string, callback: (...args: any[]) => void): void;
  off(event: string, callback: (...args: any[]) => void): void;
  events: EventEmitter;
}

export class TauriEngineAdapter implements EngineAdapter {
  public sessionId: string;
  public events: EventEmitter;
  
  private engineId: string;
  private enginePath: string;
  private engineName: string;
  private unlistenFunctions: UnlistenFn[] = [];
  private initialized: boolean = false;
  private searchDepth: number = 5;

  constructor(sessionId: string, enginePath: string, engineName: string = 'Engine') {
    this.sessionId = sessionId;
    this.engineId = sessionId; // Use sessionId as engineId for consistency
    this.enginePath = enginePath;
    this.engineName = engineName;
    this.events = new EventEmitter();
  }

  async initialize(): Promise<void> {
    if (this.initialized) {
      return;
    }

    // Spawn the engine process
    const spawnResponse = await invoke<CommandResponse>('spawn_engine', {
      engineId: this.engineId,
      name: this.engineName,
      path: this.enginePath,
    });

    if (!spawnResponse.success) {
      throw new Error(spawnResponse.message || 'Failed to spawn engine');
    }

    // Set up event listeners for USI messages
    await this.setupEventListeners();

    // Wait a bit for initialization
    await new Promise(resolve => setTimeout(resolve, 200));

    // Send usinewgame
    await this.sendCommand('usinewgame');
    
    // Send isready
    await this.sendCommand('isready');

    this.initialized = true;
  }

  private async setupEventListeners(): Promise<void> {
    // Listen for USI messages
    const messageEvent = `usi-message::${this.engineId}`;
    const messageUnlisten = await listen<string>(messageEvent, (event) => {
      const message = event.payload;
      
      // Emit the raw message
      this.events.emit('usiCommandReceived', {
        command: message,
        sessionId: this.sessionId,
      });

      // Also emit for compatibility with old code
      this.events.emit('message', message);
    });
    this.unlistenFunctions.push(messageUnlisten);

    // Listen for errors
    const errorEvent = `usi-error::${this.engineId}`;
    const errorUnlisten = await listen<string>(errorEvent, (event) => {
      const error = event.payload;
      
      this.events.emit('error', error);
      this.events.emit('usiCommandReceived', {
        command: `ERROR: ${error}`,
        sessionId: this.sessionId,
      });
    });
    this.unlistenFunctions.push(errorUnlisten);
  }

  async sendCommand(command: string): Promise<void> {
    // Emit sent event for monitoring
    this.events.emit('usiCommandSent', {
      command,
      sessionId: this.sessionId,
    });

    // Send via Tauri backend
    const response = await invoke<CommandResponse>('send_usi_command', {
      engineId: this.engineId,
      command,
    });

    if (!response.success) {
      throw new Error(response.message || 'Failed to send command');
    }
  }

  setSearchDepth(depth: number): void {
    this.searchDepth = depth;
    // Send setoption command
    this.sendCommand(`setoption name depth value ${depth}`).catch(err => {
      console.error('Failed to set search depth:', err);
    });
  }

  async destroy(): Promise<void> {
    // Stop the engine
    try {
      await invoke<CommandResponse>('stop_engine', {
        engineId: this.engineId,
      });
    } catch (error) {
      console.error('Error stopping engine:', error);
    }

    // Unlisten from all events
    for (const unlisten of this.unlistenFunctions) {
      unlisten();
    }
    this.unlistenFunctions = [];

    // Clear all event listeners
    this.events.removeAllListeners();

    this.initialized = false;
  }

  on(event: string, callback: (...args: any[]) => void): void {
    this.events.on(event, callback);
  }

  off(event: string, callback: (...args: any[]) => void): void {
    this.events.off(event, callback);
  }
}

/**
 * Factory function to create an engine adapter
 * This can be used to create Tauri adapters based on environment
 */
export async function createEngineAdapter(
  sessionId: string,
  enginePath?: string,
  engineName?: string
): Promise<EngineAdapter> {
  // For now, we'll use Tauri if we're in a Tauri environment
  // In the future, this could detect the environment and choose appropriately
  
  if (!enginePath) {
    // Get built-in engine path
    const pathResponse = await invoke<CommandResponse<{ path: string }>>('get_builtin_engine_path');
    
    if (!pathResponse.success || !pathResponse.data) {
      throw new Error('Failed to get built-in engine path');
    }
    
    enginePath = pathResponse.data.path;
    engineName = 'Built-in Engine';
  }

  const adapter = new TauriEngineAdapter(sessionId, enginePath, engineName);
  await adapter.initialize();
  
  return adapter;
}

