import { Record, InitialPositionSFEN, Move, Position } from 'tsshogi';
import { EngineAdapter } from './engine';
import { EventEmitter } from 'events';

export class ShogiController extends EventEmitter {
  private record: Record;
  private engine: EngineAdapter;

  constructor(engine: EngineAdapter) {
    super();
    this.engine = engine;
    this.record = Record.newBySFEN(InitialPositionSFEN);

    this.engine.on('bestmove', ({ move: usiMove }) => {
      if (usiMove && usiMove !== 'resign') {
        this.applyMove(usiMove);
      }
      this.emit('stateChanged', this.record.position);
    });
  }

  async initialize(): Promise<void> {
    await this.engine.init();
    await this.engine.isReady();
    await this.engine.newGame();
    this.emit('stateChanged', this.record.position);
  }

  public getPosition(): Position {
    return this.record.position;
  }

  public handleUserMove(usiMove: string): boolean {
    const moveResult = this.applyMove(usiMove);
    if (moveResult) {
      this.emit('stateChanged', this.record.position);
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
    const moves = this.record.moves.map(m => m.toUSI());
    // The SFEN of the initial position
    const sfen = this.record.root.position.toSFEN(); 
    
    this.engine.setPosition(sfen, moves);
    this.engine.go({ btime: 30000, wtime: 30000, byoyomi: 1000 });
  }
  
  public newGame(): void {
      this.record = Record.newBySFEN(InitialPositionSFEN);
      this.engine.newGame();
      this.emit('stateChanged', this.record.position);
  }
}
