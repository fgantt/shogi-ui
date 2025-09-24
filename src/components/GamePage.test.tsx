import { render, screen, fireEvent } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ShogiControllerProvider } from '../context/ShogiControllerContext';
import { vi, describe, it, expect, beforeEach } from 'vitest';
import { ShogiController } from '../usi/controller';
import { Position, Record, Square } from 'tsshogi';
import GamePage from './GamePage';

// Mock tsshogi since it cannot be used in vitest
vi.mock('tsshogi', async () => {
  const original = await vi.importActual('tsshogi');
  const Position = vi.fn();
  Position.prototype.get = vi.fn();
  Position.prototype.moves = vi.fn();
  Position.prototype.toSFEN = vi.fn();
  Position.prototype.clone = vi.fn().mockReturnThis();

  const Square = {
    fromRowCol: vi.fn((r, c) => ({ row: r, col: c, toUSI: () => `${9-c}${String.fromCharCode('a'.charCodeAt(0) + r)}`})),
  };

  const Record = vi.fn(() => ({
    moves: [],
  }));

  return {
    ...original,
    Position,
    Square,
    Record,
    PieceType: original.PieceType,
  };
});

describe('GamePage', () => {
  let mockController: ShogiController;
  let mockPosition: Position;

  beforeEach(() => {
    mockPosition = new Position();
    // Create a mutable board
    mockPosition.board = Array(9).fill(null).map(() => Array(9).fill(null));
    mockPosition.board[6][8] = { kind: 'PAWN', color: 'black' }; // Black pawn at 1g (row 6, col 8)
    mockPosition.turn = 'black';
    mockPosition.hand = { black: [], white: [] };

    const mockRecord = new Record();

    mockController = {
      on: vi.fn(),
      off: vi.fn(),
      getPosition: () => mockPosition,
      getRecord: () => mockRecord,
      handleUserMove: vi.fn(),
      newGame: vi.fn(),
    } as unknown as ShogiController;
  });

  it('renders the game page and handles a valid move', () => {
    render(
      <BrowserRouter>
        <ShogiControllerProvider controller={mockController}>
          <GamePage />
        </ShogiControllerProvider>
      </BrowserRouter>
    );

    // 1g is row 6, col 8.
    // 1f is row 5, col 8.
    const pawnSquare = screen.getByTestId('square-6-8');
    const destinationSquare = screen.getByTestId('square-5-8');

    fireEvent.click(pawnSquare);
    fireEvent.click(destinationSquare);

    expect(mockController.handleUserMove).toHaveBeenCalledWith('1g1f');
  });
});