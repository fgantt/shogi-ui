import { Record, InitialPositionSFEN, Move, Position } from 'tsshogi';
import { EngineAdapter } from './engine';
import { EventEmitter } from '../utils/events';

export class ShogiController extends EventEmitter {
  private record: Record;
  private engine: EngineAdapter;

  constructor(engine: EngineAdapter) {
    super();
    this.engine = engine;
    this.record = Record.newByUSI(`sfen ${InitialPositionSFEN}`);

    this.engine.on('bestmove', ({ move: usiMove }) => {
      if (usiMove && usiMove !== 'resign') {
        this.applyMove(usiMove);
      }
      this.emit('stateChanged', this.record.position);
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
    this.emit('stateChanged', this.record.position);
    console.log('ShogiController: State changed emitted.');
  }

  public getPosition(): Position {
    return this.record.position;
  }

  public getRecord(): Record {
    return this.record;
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
    const sfen = this.record.position.toSFEN();
    
    this.engine.setPosition(sfen, []);
    this.engine.go({ btime: 30000, wtime: 30000, byoyomi: 1000 });
  }
  
  public newGame(): void {
      this.record = Record.newByUSI(`sfen ${InitialPositionSFEN}`);
      this.engine.newGame();
      this.emit('stateChanged', this.record.position);
  }

  public loadSfen(sfen: string): void {
    this.record = Record.newByUSI(`sfen ${sfen}`);
    this.emit('stateChanged', this.record.position);
  }
}
