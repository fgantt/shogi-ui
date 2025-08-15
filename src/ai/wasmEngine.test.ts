import type { GameState } from '../types';
import { describe, it, expect } from 'vitest';
import {
  getWasmAiMove,
  getPerformanceMetrics
} from './wasmEngine.js';

// Mock game state for testing
const mockGameState: GameState = {
  board: [
    // Row 0 (top - player2)
    [
      { type: 'L', player: 'player2' }, { type: 'N', player: 'player2' }, { type: 'S', player: 'player2' }, { type: 'G', player: 'player2' }, { type: 'K', player: 'player2' }, { type: 'G', player: 'player2' }, { type: 'S', player: 'player2' }, { type: 'N', player: 'player2' }, { type: 'L', player: 'player2' }
    ],
    // Row 1
    [
      null, { type: 'R', player: 'player2' }, null, null, null, null, null, { type: 'B', player: 'player2' }, null
    ],
    // Row 2
    Array(9).fill({ type: 'P', player: 'player2' }),
    // Rows 3-5 (empty)
    Array(9).fill(null),
    Array(9).fill(null),
    Array(9).fill(null),
    // Row 6
    Array(9).fill({ type: 'P', player: 'player1' }),
    // Row 7
    [
      null, { type: 'B', player: 'player1' }, null, null, null, null, null, { type: 'R', player: 'player1' }, null
    ],
    // Row 8 (bottom - player1)
    [
      { type: 'L', player: 'player1' }, { type: 'N', player: 'player1' }, { type: 'S', player: 'player1' }, { type: 'G', player: 'player1' }, { type: 'K', player: 'player1' }, { type: 'G', player: 'player1' }, { type: 'S', player: 'player1' }, { type: 'N', player: 'player1' }, { type: 'L', player: 'player1' }
    ]
  ],
  currentPlayer: 'player1',
  capturedPieces: {
    player1: [],
    player2: []
  },
  moveHistory: [],
  isCheck: false,
  isCheckmate: false,
  isDraw: false,
  kingPositions: {
    player1: [8, 4],
    player2: [0, 4]
  },
  pastStates: [],
};

describe('WebAssembly Engine Tests', () => {
  it('should generate a valid move', async () => {
    const move = await getWasmAiMove(mockGameState, 1);
    expect(move).toBeDefined();
    expect(move).not.toBeNull();
    expect(move.from === 'drop' || (Array.isArray(move.from) && Array.isArray(move.to))).toBe(true);
  });

  it('should return performance metrics', async () => {
    const metrics = await getPerformanceMetrics(mockGameState, 1);
    expect(metrics).toBeDefined();
    expect(metrics.executionTime).toBeGreaterThan(0);
  });
});