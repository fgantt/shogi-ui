import { Record, InitialPositionSFEN, Move, ImmutablePosition, Square, PieceType as TsshogiPieceType } from 'tsshogi';
import { EngineAdapter } from './engine';
import { EventEmitter } from '../utils/events';



export class ShogiController extends EventEmitter {
  private record: Record;
  private engine: EngineAdapter;
  private initialized = false;

  constructor(engine: EngineAdapter) {
    super();
    this.engine = engine;
    const recordResult = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to create initial record: ${recordResult.message}`);
    }
    this.record = recordResult;

    this.engine.on('bestmove', ({ move: usiMove }) => {
      if (usiMove && usiMove !== 'resign') {
        this.applyMove(usiMove);
      }
      this.emitStateChanged();
    });
  }

  async initialize(): Promise<void> {
    console.log('ShogiController: Initializing engine...');
    await this.engine.init();
    console.log('ShogiController: Engine initialized. Checking readiness...');
    await this.engine.isReady();
    console.log('ShogiController: Engine ready. Starting new game...');
    await this.engine.newGame();
    console.log('ShogiController: New game started. Emitting stateChanged...');
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
    if (isAttacked) {
      console.log(`Square ${square.usi} (${targetPiece.color}) is attacked by ${opposingAttackers.length} opposing pieces:`, opposingAttackers.map(a => a.usi));
    }
    return isAttacked;
  }

  public handleUserMove(usiMove: string): boolean {
    const moveResult = this.applyMove(usiMove);
    if (moveResult) {
      this.emitStateChanged();
      this.requestEngineMove();
      return true;
    }
    return false;
  }

  private applyMove(usiMove: string): Move | null {
    const move = this.record.position.createMoveByUSI(usiMove);
    if (move && this.record.append(move)) {
      return move;
    }
    return null;
  }

  private requestEngineMove(): void {
    const sfen = this.record.position.sfen;
    
    this.engine.setPosition(sfen, []);
    this.engine.go({ btime: 30000, wtime: 30000, byoyomi: 1000 });
  }
  
  public newGame(): void {
      const recordResult = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);
      if (recordResult instanceof Error) {
        throw new Error(`Failed to create new game record: ${recordResult.message}`);
      }
      this.record = recordResult;
      this.engine.newGame();
      this.emitStateChanged();
  }

  public loadSfen(sfen: string): void {
    const recordResult = Record.newByUSI(`sfen ${sfen}`);
    if (recordResult instanceof Error) {
      throw new Error(`Failed to load SFEN: ${recordResult.message}`);
    }
    this.record = recordResult;
    this.emitStateChanged();
  }

  public quit(): void {
    this.engine.quit();
  }

  private emitStateChanged(): void {
    // Force a new reference to ensure React re-renders
    this.emit('stateChanged', this.record.position);
  }
}