import { Record, InitialPositionSFEN, Move, Position } from 'tsshogi';
import { EngineAdapter } from './engine';
import { EventEmitter } from '../utils/events';



export class ShogiController extends EventEmitter {
  private record: Record;
  private engine: EngineAdapter;
  private initialized = false;

  constructor(engine: EngineAdapter) {
    super();
    this.engine = engine;
    this.record = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);

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

  public getPosition(): Position {
    return this.record.position;
  }

  public getRecord(): Record {
    return this.record;
  }

  public isInitialized(): boolean {
    return this.initialized;
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
      this.record = Record.newByUSI(`sfen ${InitialPositionSFEN.STANDARD}`);
      this.engine.newGame();
      this.emitStateChanged();
  }

  public loadSfen(sfen: string): void {
    this.record = Record.newByUSI(`sfen ${sfen}`);
    this.emitStateChanged();
  }

  public quit(): void {
    this.engine.quit();
  }

  private emitStateChanged(): void {
    // Create a new Position object to ensure React detects the change.
    const newPosition = Position.newBySFEN(this.record.position.sfen);
    this.emit('stateChanged', newPosition);
  }
}