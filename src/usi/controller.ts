import { Record, InitialPositionSFEN, Move, ImmutablePosition, Square, PieceType as TsshogiPieceType } from 'tsshogi';
import { EngineAdapter, WasmEngineAdapter } from './engine';
import { EventEmitter } from '../utils/events';



export class ShogiController extends EventEmitter {
  private static instanceCount = 0;
  private instanceId: string;
  private instanceNumber: number;
  private record: Record;
  private sessions: Map<string, EngineAdapter> = new Map();
  private initialized = false;
  private player1Type: 'human' | 'ai' = 'human';
  private player2Type: 'human' | 'ai' = 'human';
  private player1Level: number = 5;
  private player2Level: number = 5;
  private btime: number = 30 * 60 * 1000;
  private wtime: number = 30 * 60 * 1000;
  private byoyomi: number = 10 * 1000;
  private recommendationsEnabled = false;
  private currentRecommendation: { from: Square | null; to: Square | null; isDrop?: boolean; pieceType?: string; isPromotion?: boolean } | null = null;
  private recommendationTimeout: NodeJS.Timeout | null = null;

  constructor() {
    super();
    this.instanceNumber = ++ShogiController.instanceCount;
    this.instanceId = `CTRL-${this.instanceNumber}-${Math.random().toString(36).substr(2, 9)}`;
    console.log(`========================================`);
    console.log(`[${this.instanceId}] Controller created`);
    console.log(`[${this.instanceId}] Total controllers: ${ShogiController.instanceCount}`);
    console.log(`========================================`);
    
    const recordResult = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to create initial record: ${recordResult.message}`);
    }
    this.record = recordResult;
  }

  private getEngine(sessionId: string): EngineAdapter {
    let engine = this.sessions.get(sessionId);
    if (!engine) {
      engine = new WasmEngineAdapter(sessionId);
      this.sessions.set(sessionId, engine);
      this.emit('sessionCreated', { sessionId, engine });
      
      // Initialize the engine asynchronously
      this.initializeEngine(engine);
      
      engine.on('bestmove', ({ move: usiMove, sessionId: bestmoveSessionId }) => {
        console.log(`[${this.instanceId}] [SEQ-4] bestmove received: ${usiMove}`);
        console.log(`[${this.instanceId}] Session: ${bestmoveSessionId}`);
        this.logRecordState('SEQ-4');
        
        // Handle AI resignation or no legal moves
        // Trim the move to handle cases like "resign " with trailing space
        const trimmedMove = usiMove?.trim();
        console.log(`[${this.instanceId}] After trim:`, { trimmedMove, isResign: trimmedMove === 'resign', isEmpty: !trimmedMove });
        
        if (trimmedMove === 'resign' || !trimmedMove) {
          console.log(`[${this.instanceId}] AI RESIGNED! usiMove: ${usiMove}, trimmed: ${trimmedMove}`);
          const isBlackTurn = this.record.position.sfen.includes(' b ');
          const winner = isBlackTurn ? 'player2' : 'player1';
          console.log(`[${this.instanceId}] EMITTING GAME OVER EVENT! winner: ${winner}`);
          this.emit('gameOver', { winner, position: this.record.position });
          this.emitStateChanged();
          return;
        }
        
        if (trimmedMove && trimmedMove !== 'resign') {
          if (bestmoveSessionId === 'sente' || bestmoveSessionId === 'gote') {
            if (this.recommendationsEnabled && this.hasHumanPlayer() && !this.isCurrentPlayerAI()) {
              this.parseRecommendation(trimmedMove);
            } else {
              console.log(`[${this.instanceId}] About to apply AI move: ${trimmedMove}`);
              console.log(`[${this.instanceId}] AI session: ${bestmoveSessionId}`);
              
              const moveResult = this.applyMove(trimmedMove);
              console.log(`[${this.instanceId}] [SEQ-5] applyMove result: ${moveResult ? 'SUCCESS' : 'FAILED'}`);
              this.logRecordState('SEQ-5');
              
              if (!moveResult) {
                // Move failed to apply - this should now properly indicate checkmate
                // since we've fixed the synchronization bug
                console.error(`[${this.instanceId}] AI MOVE REJECTED: ${trimmedMove}`);
                console.error(`[${this.instanceId}] Session: ${bestmoveSessionId}`);
                
                // The engine returned an invalid move - opponent wins
                const isBlackTurn = this.record.position.sfen.includes(' b ');
                const winner = isBlackTurn ? 'player2' : 'player1';
                console.log(`[${this.instanceId}] INVALID MOVE - GAME OVER! Winner: ${winner}`);
                this.emit('gameOver', { winner, position: this.record.position });
                this.emitStateChanged();
                return;
              }
              
              this.emit('aiMoveMade', { move: trimmedMove });
              this.emitStateChanged();
              
              console.log(`[${this.instanceId}] [SEQ-6] Checking if next player is AI`);
              if (this.isCurrentPlayerAI()) {
                console.log(`[${this.instanceId}] [SEQ-7] Next player IS AI, requesting move`);
                this.requestEngineMove();
              } else {
                console.log(`[${this.instanceId}] [SEQ-7] Next player is HUMAN, waiting`);
              }
            }
          }
        } else {
          this.emitStateChanged();
        }
      });
    }
    return engine;
  }

  private async initializeEngine(engine: EngineAdapter): Promise<void> {
    try {
      await engine.init();
      await engine.isReady();
      
      // Now set the engine to the current position
      engine.newGame();
      const currentSfen = this.record.position.sfen;
      // Pass empty moves array - the SFEN already has the complete position
      engine.setPosition(currentSfen, []);
    } catch (error) {
      console.error('Failed to initialize engine:', error);
    }
  }

  private async synchronizeAllEngines(currentSfen: string, moves: string[]): Promise<void> {
    // Synchronize all existing engines with the current position
    // NOTE: We ignore the moves parameter and pass empty array - SFEN already has complete position
    const syncPromises = Array.from(this.sessions.values()).map(async (engine) => {
      try {
        await engine.init();
        await engine.isReady();
        engine.newGame();
        // Pass empty moves array - the SFEN already has the complete position
        engine.setPosition(currentSfen, []);
      } catch (error) {
        console.error('Failed to synchronize engine:', error);
      }
    });
    
    await Promise.all(syncPromises);
  }

  async initialize(): Promise<void> {
    console.log('ShogiController: Initializing engines...');
    const initializers = Array.from(this.sessions.values()).map(engine => engine.init());
    await Promise.all(initializers);
    console.log('ShogiController: Engines initialized. Checking readiness...');
    const readinessCheckers = Array.from(this.sessions.values()).map(engine => engine.isReady());
    await Promise.all(readinessCheckers);
    console.log('ShogiController: Engines ready. Starting new game...');
    const newGameStarters = Array.from(this.sessions.values()).map(engine => engine.newGame());
    await Promise.all(newGameStarters);
    console.log('ShogiController: Emitting stateChanged...');
    this.emitStateChanged();
    console.log('ShogiController: State changed emitted.');
    this.initialized = true;
  }

  public getPosition(): ImmutablePosition {
    return this.record.position;
  }

  public getRecord(): Record {
    return this.record;
  }

  public getLastMove(): { from: Square | null; to: Square | null } | null {
    const moves = this.record.moves;
    if (moves.length === 0) return null;
    
    const lastMove = moves[moves.length - 1];
    if (!lastMove || !('from' in lastMove.move) || !('to' in lastMove.move)) {
      return null;
    }
    
    // Handle drops (where from is a PieceType) by setting from to null
    const fromSquare = typeof lastMove.move.from === 'object' && 'x' in lastMove.move.from 
      ? lastMove.move.from as Square 
      : null;
    
    return {
      from: fromSquare,
      to: lastMove.move.to as Square
    };
  }

  public isInitialized(): boolean {
    return this.initialized;
  }

  public setPlayerTypes(player1Type: 'human' | 'ai', player2Type: 'human' | 'ai'): void {
    this.player1Type = player1Type;
    this.player2Type = player2Type;
  }

  public setAILevels(player1Level: number, player2Level: number): void {
    this.player1Level = player1Level;
    this.player2Level = player2Level;
  }

  public setTimeControls(btime: number, wtime: number, byoyomi: number): void {
    this.btime = btime;
    this.wtime = wtime;
    this.byoyomi = byoyomi;
  }

  public updateCurrentTimes(blackTime: number, whiteTime: number): void {
    this.btime = blackTime;
    this.wtime = whiteTime;
  }

  public getPlayerTypes(): { player1Type: 'human' | 'ai'; player2Type: 'human' | 'ai' } {
    return { player1Type: this.player1Type, player2Type: this.player2Type };
  }

  public setRecommendationsEnabled(enabled: boolean): void {
    this.recommendationsEnabled = enabled;
    if (!enabled) {
      this.currentRecommendation = null;
      // Clear any pending recommendation timeout
      if (this.recommendationTimeout) {
        clearTimeout(this.recommendationTimeout);
        this.recommendationTimeout = null;
      }
    }
    this.emitStateChanged();
  }

  public areRecommendationsEnabled(): boolean {
    return this.recommendationsEnabled;
  }

  public   getCurrentRecommendation(): { from: Square | null; to: Square | null } | null {
    console.log('getCurrentRecommendation called, returning:', this.currentRecommendation);
    console.log('Current recommendation type:', typeof this.currentRecommendation);
    console.log('Current recommendation is null?', this.currentRecommendation === null);
    console.log('Current recommendation is undefined?', this.currentRecommendation === undefined);
    return this.currentRecommendation;
  }

  public clearRecommendation(): void {
    console.log('Clearing recommendation - was:', this.currentRecommendation);
    console.trace('Clear recommendation called from:');
    this.currentRecommendation = null;
    console.log('Recommendation cleared, now:', this.currentRecommendation);
    // Don't emit stateChanged here to avoid circular calls
    // The state will be updated when the next stateChanged event occurs
  }

  public hasHumanPlayer(): boolean {
    return this.player1Type === 'human' || this.player2Type === 'human';
  }

  public isCurrentPlayerAI(): boolean {
    const isPlayer1Turn = this.record.position.sfen.includes(' b ');
    const currentPlayerType = isPlayer1Turn ? this.player1Type : this.player2Type;
    return currentPlayerType !== 'human';
  }

  public getLegalMovesForSquare(square: Square): Square[] {
    // Since ImmutablePosition doesn't have generateMoves, we'll generate legal moves manually
    // by checking all possible destination squares and validating each move
    const legalMoves: Square[] = [];
    
    // TODO: Review this implementation for efficiency - checking all 81 squares may be overkill.
    // Consider implementing piece-specific move generation or using tsshogi's internal move generation
    // if available in future versions. This brute force approach works but could be optimized.
    
    // Check all 81 squares as potential destinations
    for (let row = 0; row < 9; row++) {
      for (let col = 0; col < 9; col++) {
        const destSquare = Square.newByXY(col, row);
        if (!destSquare) continue;
        
        // Create a move from the selected square to this destination
        const move = this.record.position.createMove(square, destSquare);
        if (move && this.record.position.isValidMove(move)) {
          legalMoves.push(destSquare);
        }
      }
    }
    
    return legalMoves;
  }

  public getValidDropSquares(pieceType: TsshogiPieceType): Square[] {
    const validSquares: Square[] = [];
    
    // First check if the player has this piece in their hand
    if (!this.hasPieceInHand(pieceType)) {
      return validSquares; // No pieces of this type in hand
    }
    
    // Check all 81 squares as potential drop destinations
    for (let row = 0; row < 9; row++) {
      for (let col = 0; col < 9; col++) {
        const destSquare = Square.newByXY(col, row);
        if (!destSquare) continue;
        
        // Check if square is empty
        if (this.record.position.board.at(destSquare)) {
          continue; // Square is occupied
        }
        
        // Create a drop move and check if it's valid
        const dropMove = this.createDropMove(pieceType, destSquare);
        if (dropMove && this.record.position.isValidMove(dropMove)) {
          validSquares.push(destSquare);
        }
      }
    }
    
    return validSquares;
  }

  private hasPieceInHand(pieceType: TsshogiPieceType): boolean {
    const currentPlayer = this.record.position.sfen.includes(' b ') ? 'black' : 'white';
    const hand = currentPlayer === 'black' ? this.record.position.blackHand : this.record.position.whiteHand;
    
    // Check if the hand has any pieces of this type
    return hand.counts.some(({ type, count }) => type === pieceType && count > 0);
  }

  private createDropMove(pieceType: TsshogiPieceType, toSquare: Square): Move | null {
    // Convert tsshogi piece type to USI piece character
    const pieceChar = this.pieceTypeToUsiChar(pieceType);
    if (!pieceChar) return null;
    
    // Create USI drop move string (e.g., "P*5d")
    const usiMove = `${pieceChar}*${toSquare.usi}`;
    
    // Create move using tsshogi's createMoveByUSI
    return this.record.position.createMoveByUSI(usiMove);
  }

  public pieceTypeToUsiChar(pieceType: TsshogiPieceType): string | null {
    // Map tsshogi piece types to USI piece characters
    switch (pieceType) {
      case TsshogiPieceType.PAWN:
        return 'P';
      case TsshogiPieceType.LANCE:
        return 'L';
      case TsshogiPieceType.KNIGHT:
        return 'N';
      case TsshogiPieceType.SILVER:
        return 'S';
      case TsshogiPieceType.GOLD:
        return 'G';
      case TsshogiPieceType.BISHOP:
        return 'B';
      case TsshogiPieceType.ROOK:
        return 'R';
      default:
        return null; // Invalid piece type for drops
    }
  }

  public isSquareAttacked(square: Square): boolean {
    // Use tsshogi's listAttackers method to check if the square is under attack
    const attackers = this.record.position.listAttackers(square);
    
    // Get the piece on the target square to determine which player it belongs to
    const targetPiece = this.record.position.board.at(square);
    if (!targetPiece) {
      return false; // No piece on the square, so it can't be attacked
    }
    
    // Filter attackers to only include pieces from the opposing player
    const opposingAttackers = attackers.filter(attackerSquare => {
      const attackerPiece = this.record.position.board.at(attackerSquare);
      return attackerPiece && attackerPiece.color !== targetPiece.color;
    });
    
    const isAttacked = opposingAttackers.length > 0;
    // if (isAttacked) {
    //   console.log(`Square ${square.usi} (${targetPiece.color}) is attacked by ${opposingAttackers.length} opposing pieces:`, opposingAttackers.map(a => a.usi));
    // }
    return isAttacked;
  }

  public handleUserMove(usiMove: string): boolean {
    console.log('handleUserMove called with:', usiMove);
    const moveResult = this.applyMove(usiMove);
    if (moveResult) {
      // Clear current recommendation when user makes a move
      console.log('Clearing recommendation due to user move');
      this.currentRecommendation = null;
      this.emitStateChanged();
      // Only request AI move if the next player is AI
      if (this.isCurrentPlayerAI()) {
        this.requestEngineMove();
      }
      return true;
    }
    return false;
  }

  private logRecordState(context: string): void {
    const sfen = this.record.position.sfen;
    const turn = sfen.includes(' b ') ? 'Black' : 'White';
    const moveCount = this.record.moves.length;
    
    console.log(`[${this.instanceId}] [${context}] RECORD STATE:`);
    console.log(`  SFEN: ${sfen}`);
    console.log(`  Turn: ${turn}`);
    console.log(`  Moves: ${moveCount}`);
    console.log(`  Player1Type: ${this.player1Type}, Player2Type: ${this.player2Type}`);
    console.log(`  isCurrentPlayerAI: ${this.isCurrentPlayerAI()}`);
  }

  private applyMove(usiMove: string): Move | null {
    console.log(`[${this.instanceId}] ========================================`);
    console.log(`[${this.instanceId}] applyMove called`);
    console.log(`[${this.instanceId}]   Move: ${usiMove}`);
    this.logRecordState('BEFORE applyMove');
    
    const move = this.record.position.createMoveByUSI(usiMove);
    console.log(`[${this.instanceId}]   createMoveByUSI result:`, move);
    
    if (move) {
      const appendResult = this.record.append(move);
      console.log(`[${this.instanceId}]   record.append result: ${appendResult}`);
      if (appendResult) {
        console.log(`[${this.instanceId}]   ✓ Move applied successfully`);
        this.logRecordState('AFTER applyMove');
        console.log(`[${this.instanceId}] ========================================`);
        return move;
      } else {
        console.error(`[${this.instanceId}]   ✗ record.append returned false - move was rejected`);
        console.error(`[${this.instanceId}] ========================================`);
      }
    } else {
      console.error(`[${this.instanceId}]   ✗ createMoveByUSI returned null`);
      console.error(`[${this.instanceId}]   This means the move is invalid for the current position`);
      console.error(`[${this.instanceId}]   Possible reasons:`);
      console.error(`[${this.instanceId}]     - Wrong player trying to move`);
      console.error(`[${this.instanceId}]     - Illegal move (blocked, out of bounds, etc)`);
      console.error(`[${this.instanceId}]     - Malformed USI string`);
      console.error(`[${this.instanceId}] ========================================`);
    }
    return null;
  }

  public requestEngineMove(currentBlackTime?: number, currentWhiteTime?: number): void {
    console.log(`[${this.instanceId}] [SEQ-1] requestEngineMove START`);
    this.logRecordState('SEQ-1');
    
    const isPlayer1Turn = this.record.position.sfen.includes(' b ');
    const sessionId = isPlayer1Turn ? 'sente' : 'gote';
    const engine = this.getEngine(sessionId);
    const level = isPlayer1Turn ? this.player1Level : this.player2Level;
    
    // CRITICAL FIX: Pass the current position SFEN with NO moves
    // The SFEN already represents the position after all moves have been applied
    // If we pass moves here, the engine will double-apply them, causing desynchronization
    const currentSfen = this.record.position.sfen;
    console.log(`[${this.instanceId}] Setting engine position: ${currentSfen}`);
    console.log(`[${this.instanceId}] Total moves in record: ${this.record.moves.length}`);
    console.log(`[${this.instanceId}] Session ID: ${sessionId}, Level: ${level}`);
    
    // Use current clock times if provided, otherwise fall back to stored values
    const btime = currentBlackTime !== undefined ? currentBlackTime : this.btime;
    const wtime = currentWhiteTime !== undefined ? currentWhiteTime : this.wtime;
    
    console.log(`[${this.instanceId}] [SEQ-2] Calling engine.setPosition`);
    engine.setSearchDepth(level);
    // Pass empty moves array - the SFEN already has the complete position
    engine.setPosition(currentSfen, []);
    
    console.log(`[${this.instanceId}] [SEQ-3] Calling engine.go`);
    engine.go({ 
      btime: btime, 
      wtime: wtime, 
      byoyomi: this.byoyomi 
    });
  }

  public async requestRecommendation(): Promise<void> {
    if (!this.recommendationsEnabled || !this.hasHumanPlayer() || this.isCurrentPlayerAI()) {
      return;
    }

    // Clear any existing timeout
    if (this.recommendationTimeout) {
      clearTimeout(this.recommendationTimeout);
    }

    const isPlayer1Turn = this.record.position.sfen.includes(' b ');
    const sessionId = isPlayer1Turn ? 'sente' : 'gote';
    const engine = this.getEngine(sessionId);
    
    // Get the current position SFEN and all moves from the record
    const currentSfen = this.record.position.sfen;
    const moves = this.record.moves.map(move => {
      if ('move' in move && typeof move.move === 'object' && 'toUSI' in move.move) {
        return (move.move as any).toUSI();
      }
      return '';
    }).filter(move => move !== '');
    
    console.log('RequestRecommendation Debug:');
    console.log('- Session ID:', sessionId);
    console.log('- Is Player 1 Turn:', isPlayer1Turn);
    console.log('- Current SFEN:', currentSfen);
    console.log('- SFEN indicates turn:', currentSfen.includes(' b ') ? 'Player 1 (Black)' : 'Player 2 (White)');
    console.log('- Move History:', moves);
    console.log('- Total Moves:', this.record.moves.length);
    console.log('- Record position turn:', this.record.position.sfen.includes(' b ') ? 'Player 1 (Black)' : 'Player 2 (White)');
    
    // Ensure both engines are synchronized with the current position
    await this.synchronizeAllEngines(currentSfen, moves);
    
    // Request a quick recommendation with shorter time
    engine.go({ btime: 1000, wtime: 1000, byoyomi: 500 });
    
    // Set a timeout to clear the recommendation request if it takes too long
    this.recommendationTimeout = setTimeout(() => {
      this.emit('recommendationTimeout');
    }, 5000); // 5 second timeout
  }
  
  public async newGame(customSfen?: string): Promise<void> {
      const sfenToUse = customSfen || InitialPositionSFEN.STANDARD;
      const recordResult = Record.newByUSI(`sfen ${sfenToUse}`);
      if (recordResult instanceof Error) {
        throw new Error(`Failed to create new game record: ${recordResult.message}`);
      }
      this.record = recordResult;
      
      // Reset all engines and set them to the starting position
      const engineUpdates = Array.from(this.sessions.values()).map(async (engine) => {
        await engine.init();
        await engine.isReady();
        engine.newGame();
        // Set the engine to the starting position
        engine.setPosition(this.record.position.sfen, []);
      });
      
      await Promise.all(engineUpdates);
      this.emitStateChanged();
      
      // Check if the first player is AI and request move
      if (this.isCurrentPlayerAI()) {
        this.requestEngineMove();
      }
  }

  public async loadSfen(sfen: string): Promise<void> {
    const recordResult = Record.newByUSI(`sfen ${sfen}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to load SFEN: ${recordResult.message}`);
    }
    this.record = recordResult;
    
    // Reset all engines and set them to the loaded position
    const engineUpdates = Array.from(this.sessions.values()).map(async (engine) => {
      await engine.init();
      await engine.isReady();
      engine.newGame();
      // Set the engine to the loaded position
      const currentSfen = this.record.position.sfen;
      // Pass empty moves array - the SFEN already has the complete position
      engine.setPosition(currentSfen, []);
    });
    
    await Promise.all(engineUpdates);
    this.emitStateChanged();
  }

  public quit(): void {
    for (const engine of this.sessions.values()) {
      engine.quit();
    }
  }

  private parseRecommendation(usiMove: string): void {
    try {
      console.log('Parsing recommendation move:', usiMove);
      console.log('Current position SFEN:', this.record.position.sfen);
      console.log('Current player turn:', this.record.position.sfen.includes(' b ') ? 'Player 1 (Black)' : 'Player 2 (White)');
      
      // Clear timeout since we got a response
      if (this.recommendationTimeout) {
        clearTimeout(this.recommendationTimeout);
        this.recommendationTimeout = null;
      }

      // Parse USI move to get from and to squares
      const move = this.record.position.createMoveByUSI(usiMove);
      console.log('Parsed move:', move);
      
      if (move && 'to' in move) {
        const fromSquare = typeof move.from === 'object' && 'x' in move.from 
          ? move.from as Square 
          : null;
        const toSquare = move.to as Square;
        
        // Check if this is a drop move (from is null or not a Square)
        const isDrop = fromSquare === null;
        let pieceType = '';
        
        if (isDrop) {
          // Extract piece type from USI move string (e.g., "P*5d" -> "P")
          const match = usiMove.match(/^([A-Z])\*/);
          pieceType = match ? match[1] : '';
        }
        
        // Check if this is a promotion move (USI move ends with '+')
        const isPromotion = usiMove.endsWith('+');
        
        console.log('Recommendation squares:', { from: fromSquare, to: toSquare, isDrop, pieceType, isPromotion });
        
        this.currentRecommendation = {
          from: fromSquare,
          to: toSquare,
          isDrop,
          pieceType,
          isPromotion
        };
        console.log('Set current recommendation:', this.currentRecommendation);
        console.log('Recommendation type after setting:', typeof this.currentRecommendation);
        console.log('Recommendation is null after setting?', this.currentRecommendation === null);
        console.log('Recommendation is undefined after setting?', this.currentRecommendation === undefined);
        this.emitStateChanged();
        this.emit('recommendationReceived');
      } else {
        console.log('Move parsing failed - invalid move structure');
        console.log('Move object:', move);
        console.log('Move has "to" property:', move && 'to' in move);
      }
    } catch (error) {
      console.error('Error parsing recommendation move:', error);
    }
  }

  private emitStateChanged(): void {
    console.log('Emitting stateChanged event, current recommendation:', this.currentRecommendation);
    // Force a new reference to ensure React re-renders
    this.emit('stateChanged', this.record.position);
  }
}