/**
 * Tauri-aware Shogi Controller
 * Extends the base controller to support Tauri-based engine management
 */

import { Record, InitialPositionSFEN, ImmutablePosition, Square } from 'tsshogi';
import { EventEmitter } from '../utils/events';
import { TauriEngineAdapter, EngineAdapter } from './tauriEngine';
import { invoke } from '@tauri-apps/api/core';
import type { CommandResponse, EngineConfig } from '../types/engine';

export class TauriShogiController extends EventEmitter {
  private static instanceCount = 0;
  private instanceId: string;
  private instanceNumber: number;
  private record: Record;
  private sessions: Map<string, EngineAdapter> = new Map();
  private initialized = false;
  private player1Type: 'human' | 'ai' = 'human';
  private player2Type: 'human' | 'ai' = 'human';
  private player1EngineId: string | null = null;
  private player2EngineId: string | null = null;
  private player1Level: number = 5;
  private player2Level: number = 5;
  private btime: number = 30 * 60 * 1000;
  private wtime: number = 30 * 60 * 1000;
  private byoyomi: number = 10 * 1000;
  private recommendationsEnabled = false;
  private currentRecommendation: { from: Square | null; to: Square | null; isDrop?: boolean; pieceType?: string; isPromotion?: boolean } | null = null;
  private positionHistory: Map<string, number> = new Map();
  private gameOver = false;

  constructor() {
    super();
    this.instanceNumber = ++TauriShogiController.instanceCount;
    this.instanceId = `TAURI-CTRL-${this.instanceNumber}-${Math.random().toString(36).substr(2, 9)}`;
    console.log(`[${this.instanceId}] Tauri Controller created`);
    
    const recordResult = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to create initial record: ${recordResult.message}`);
    }
    this.record = recordResult;
  }

  async initialize(): Promise<void> {
    if (this.initialized) return;
    
    console.log(`[${this.instanceId}] Initializing Tauri controller`);
    
    // Try to register built-in engine
    try {
      await invoke<CommandResponse>('register_builtin_engine');
    } catch (error) {
      console.warn('Failed to register built-in engine:', error);
    }
    
    this.initialized = true;
  }

  isInitialized(): boolean {
    return this.initialized;
  }

  /**
   * Set engine IDs for players
   */
  setPlayerEngines(player1EngineId: string | null, player2EngineId: string | null): void {
    this.player1EngineId = player1EngineId;
    this.player2EngineId = player2EngineId;
    console.log(`[${this.instanceId}] Set engines - P1: ${player1EngineId}, P2: ${player2EngineId}`);
  }

  /**
   * Get engine for current player
   */
  private async getCurrentEngine(): Promise<EngineAdapter | null> {
    const isBlackTurn = this.record.position.sfen.includes(' b ');
    const engineId = isBlackTurn ? this.player1EngineId : this.player2EngineId;
    
    if (!engineId) return null;

    // Get or create engine session
    const sessionId = isBlackTurn ? 'sente' : 'gote';
    let engine = this.sessions.get(sessionId);
    
    if (!engine) {
      // Load engine config
      const response = await invoke<CommandResponse<EngineConfig[]>>('get_engines');
      if (!response.success || !response.data) {
        throw new Error('Failed to load engines');
      }
      
      const engineConfig = response.data.find(e => e.id === engineId);
      if (!engineConfig) {
        throw new Error(`Engine not found: ${engineId}`);
      }

      // Create and initialize engine adapter
      engine = new TauriEngineAdapter(sessionId, engineConfig.path, engineConfig.name);
      await engine.initialize();
      
      this.sessions.set(sessionId, engine);
      this.emit('sessionCreated', { sessionId, engine });

      // Set up move response handler
      engine.on('message', (message: string) => {
        if (message.startsWith('bestmove')) {
          const parts = message.split(/\s+/);
          const move = parts[1];
          this.handleEngineBestMove(move, sessionId);
        }
      });
    }

    return engine;
  }

  private handleEngineBestMove(move: string, sessionId: string): void {
    console.log(`[${this.instanceId}] Best move received: ${move} from ${sessionId}`);

    if (move === 'resign' || !move) {
      const isBlackTurn = this.record.position.sfen.includes(' b ');
      const winner = isBlackTurn ? 'player2' : 'player1';
      this.gameOver = true;
      this.emit('gameOver', { winner, position: this.record.position, endgameType: 'resignation' });
      this.emitStateChanged();
      return;
    }

    // Handle recommendations vs actual moves
    if (sessionId === 'sente' || sessionId === 'gote') {
      if (this.recommendationsEnabled && this.hasHumanPlayer() && !this.isCurrentPlayerAI()) {
        this.parseRecommendation(move);
      } else {
        const moveResult = this.applyMove(move);
        
        if (!moveResult) {
          const isBlackTurn = this.record.position.sfen.includes(' b ');
          const winner = isBlackTurn ? 'player2' : 'player1';
          this.gameOver = true;
          this.emit('gameOver', { winner, position: this.record.position, endgameType: 'illegal' });
          this.emitStateChanged();
          return;
        }

        this.emit('aiMoveMade', { move });
        this.emitStateChanged();

        // Chain AI moves if next player is also AI
        if (!this.gameOver && this.isCurrentPlayerAI()) {
          setTimeout(() => this.requestEngineMove(), 500);
        }
      }
    }
  }

  private parseRecommendation(usiMove: string): void {
    // Parse the USI move to extract from/to squares
    // This is a simplified implementation
    console.log(`[${this.instanceId}] Parsing recommendation: ${usiMove}`);
    
    // Store the recommendation
    this.currentRecommendation = { from: null, to: null };
    this.emit('recommendationReady', { move: usiMove });
  }

  setPlayerTypes(player1: 'human' | 'ai', player2: 'human' | 'ai'): void {
    this.player1Type = player1;
    this.player2Type = player2;
  }

  setAILevels(level1: number, level2: number): void {
    this.player1Level = level1;
    this.player2Level = level2;
  }

  setTimeControls(btime: number, wtime: number, byoyomi: number): void {
    this.btime = btime;
    this.wtime = wtime;
    this.byoyomi = byoyomi;
  }

  updateCurrentTimes(btime: number, wtime: number): void {
    this.btime = btime;
    this.wtime = wtime;
  }

  async newGame(initialSfen?: string): Promise<void> {
    console.log(`[${this.instanceId}] Starting new game`);
    this.gameOver = false;
    this.positionHistory.clear();
    this.currentRecommendation = null;

    // Stop all existing engines
    await this.stopAllEngines();
    this.sessions.clear();

    // Create new record
    const sfen = initialSfen || InitialPositionSFEN.STANDARD;
    const recordResult = Record.newByUSI(`sfen ${sfen}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to create record: ${recordResult.message}`);
    }
    this.record = recordResult;

    this.emitStateChanged();

    // If first player is AI, request move
    if (this.isCurrentPlayerAI()) {
      setTimeout(() => this.requestEngineMove(), 500);
    }
  }

  async requestEngineMove(): Promise<void> {
    if (this.gameOver) {
      console.log(`[${this.instanceId}] Game over, not requesting move`);
      return;
    }

    try {
      const engine = await this.getCurrentEngine();
      if (!engine) {
        console.error('No engine available for current player');
        return;
      }

      // Set search depth based on level
      const isBlackTurn = this.record.position.sfen.includes(' b ');
      const level = isBlackTurn ? this.player1Level : this.player2Level;
      engine.setSearchDepth(level);

      // Send position
      const currentSfen = this.record.position.sfen;
      await engine.sendCommand(`position sfen ${currentSfen}`);

      // Send go command with time controls
      const goCmd = `go btime ${this.btime} wtime ${this.wtime} byoyomi ${this.byoyomi}`;
      await engine.sendCommand(goCmd);
      
      console.log(`[${this.instanceId}] Requested move from engine`);
    } catch (error) {
      console.error('Failed to request engine move:', error);
    }
  }

  async requestRecommendation(): Promise<void> {
    // For recommendations, we use a separate recommendation session
    // This is similar to requestEngineMove but for hints
    console.log(`[${this.instanceId}] Requesting recommendation`);
    // Implementation similar to requestEngineMove but with different session
  }

  async handleUserMove(usiMove: string): Promise<boolean> {
    console.log(`[${this.instanceId}] User move: ${usiMove}`);
    
    const result = this.applyMove(usiMove);
    if (!result) {
      return false;
    }

    this.emit('userMoveMade', { move: usiMove });
    this.emitStateChanged();

    // Clear recommendation
    this.currentRecommendation = null;

    // Request AI move if next player is AI
    if (!this.gameOver && this.isCurrentPlayerAI()) {
      setTimeout(() => this.requestEngineMove(), 500);
    }

    return true;
  }

  private applyMove(usiMove: string): boolean {
    try {
      const moveResult = this.record.unmake(usiMove);
      if (moveResult instanceof Error) {
        console.error(`[${this.instanceId}] Move failed:`, moveResult);
        return false;
      }

      // Track position for repetition
      const positionKey = this.record.position.sfen;
      const count = (this.positionHistory.get(positionKey) || 0) + 1;
      this.positionHistory.set(positionKey, count);

      if (count >= 4) {
        this.gameOver = true;
        this.emit('gameOver', { winner: 'draw', position: this.record.position, endgameType: 'repetition' });
      }

      return true;
    } catch (error) {
      console.error(`[${this.instanceId}] Error applying move:`, error);
      return false;
    }
  }

  getPosition(): ImmutablePosition {
    return this.record.position;
  }

  getRecord(): Record {
    return this.record;
  }

  getLastMove(): { from: Square | null; to: Square | null } | null {
    const moves = this.record.moves;
    if (moves.length === 0) return null;
    
    const lastMove = moves[moves.length - 1];
    return {
      from: lastMove.from || null,
      to: lastMove.to
    };
  }

  isCurrentPlayerAI(): boolean {
    const isBlackTurn = this.record.position.sfen.includes(' b ');
    return isBlackTurn ? this.player1Type === 'ai' : this.player2Type === 'ai';
  }

  hasHumanPlayer(): boolean {
    return this.player1Type === 'human' || this.player2Type === 'human';
  }

  areRecommendationsEnabled(): boolean {
    return this.recommendationsEnabled;
  }

  setRecommendationsEnabled(enabled: boolean): void {
    this.recommendationsEnabled = enabled;
  }

  getCurrentRecommendation() {
    return this.currentRecommendation;
  }

  clearRecommendation(): void {
    this.currentRecommendation = null;
  }

  private emitStateChanged(): void {
    this.emit('stateChanged', this.record.position);
  }

  async stopAllEngines(): Promise<void> {
    console.log(`[${this.instanceId}] Stopping all engines`);
    
    for (const [sessionId, engine] of this.sessions.entries()) {
      try {
        await engine.destroy();
      } catch (error) {
        console.error(`Failed to stop engine ${sessionId}:`, error);
      }
    }
  }

  async destroy(): Promise<void> {
    console.log(`[${this.instanceId}] Destroying controller`);
    await this.stopAllEngines();
    this.sessions.clear();
    this.removeAllListeners();
  }
}

