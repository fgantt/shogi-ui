import { ShogiController } from './controller';
import { EngineAdapter } from './engine';
import { EventEmitter } from 'events';
import { vi, describe, it, expect, beforeEach } from 'vitest';

const mockMoveInstance = (move: string) => ({ toUSI: () => move, toSFEN: () => move });

const mockPosition = {
  createMoveByUSI: vi.fn((move: string) => {
    if (move === '1a1b') return null;
    return mockMoveInstance(move);
  }),
  toSFEN: () => 'sfen_string',
  lastMove: null as { toUSI: () => string; toSFEN: () => string } | null,
};

const mockRecord = {
  position: mockPosition,
  moves: [],
  root: { position: { toSFEN: () => 'sfen_string' } },
  append: vi.fn((move) => {
      mockPosition.lastMove = move;
      return true;
  }),
};

vi.mock('tsshogi', async () => {
  const actualTsshogi = await vi.importActual('tsshogi') as any;
  return {
    ...actualTsshogi,
    InitialPositionSFEN: 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1',
    Record: {
      ...actualTsshogi.Record,
      newBySFEN: vi.fn(() => mockRecord),
    },
  };
});

// A mock EngineAdapter for testing the controller
class MockEngineAdapter extends EventEmitter implements EngineAdapter {
  init = vi.fn().mockResolvedValue(undefined);
  isReady = vi.fn().mockResolvedValue(undefined);
  setOptions = vi.fn().mockResolvedValue(undefined);
  newGame = vi.fn().mockResolvedValue(undefined);
  setPosition = vi.fn().mockResolvedValue(undefined);
  go = vi.fn().mockResolvedValue(undefined);
  stop = vi.fn().mockResolvedValue(undefined);
  quit = vi.fn().mockResolvedValue(undefined);

  // Helper to simulate an engine's response
  simulateBestMove(usiMove: string) {
    this.emit('bestmove', { move: usiMove });
  }
}

describe('ShogiController', () => {
  let controller: ShogiController;
  let mockEngine: MockEngineAdapter;

  beforeEach(async () => {
    vi.clearAllMocks();
    mockPosition.lastMove = null;
    mockEngine = new MockEngineAdapter();
    controller = new ShogiController(mockEngine);
    await controller.initialize();
  });

  it('should initialize the engine on startup', () => {
    expect(mockEngine.init).toHaveBeenCalled();
    expect(mockEngine.isReady).toHaveBeenCalled();
    expect(mockEngine.newGame).toHaveBeenCalled();
  });

  it('should handle a valid user move and request an engine move', () => {
    const usiMove = '7g7f'; // A legal opening move
    const result = controller.handleUserMove(usiMove);

    expect(result).toBe(true);
    // Check that the controller's internal state was updated
    expect(controller.getPosition().toSFEN()).not.toContain('startpos');
    // Check that the controller asked the engine to move
    expect(mockEngine.setPosition).toHaveBeenCalledWith(expect.any(String), []);
    expect(mockEngine.go).toHaveBeenCalled();
  });

  it('should reject an illegal user move', () => {
    const usiMove = '1a1b'; // An illegal opening move
    const result = controller.handleUserMove(usiMove);
    expect(result).toBe(false);
    expect(mockEngine.go).not.toHaveBeenCalled();
  });

  it('should apply an engine move when received', () => {
    // User makes a move
    controller.handleUserMove('7g7f');
    
    // Engine responds
    const engineMove = '3c3d';
    mockEngine.simulateBestMove(engineMove);

    // Check that the engine's move was applied to the board
    const lastMove = controller.getPosition().lastMove;
    expect(lastMove?.toUSI()).toBe(engineMove);
  });

  it('should emit a stateChanged event after a move', () => {
    const listener = vi.fn();
    controller.on('stateChanged', listener);

    controller.handleUserMove('7g7f');
    expect(listener).toHaveBeenCalledTimes(1);

    mockEngine.simulateBestMove('3c3d');
    expect(listener).toHaveBeenCalledTimes(2);
  });
});