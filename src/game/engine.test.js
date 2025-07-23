import { describe, it, expect, beforeEach } from 'vitest';
import {
  getInitialGameState,
  getLegalMoves,
  movePiece,
  dropPiece,
  isKingInCheck,
  isCheckmate,
  PAWN, LANCE, KNIGHT, SILVER, GOLD, KING, ROOK, BISHOP,
  PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_ROOK, PROMOTED_BISHOP,
  PLAYER_1, PLAYER_2,
  ROWS, COLS
} from './engine';

describe('getInitialGameState', () => {
  it('should return the correct initial game state', () => {
    const gameState = getInitialGameState();
    expect(gameState.board.length).toBe(ROWS);
    expect(gameState.board[0].length).toBe(COLS);
    expect(gameState.currentPlayer).toBe(PLAYER_1);
    expect(gameState.capturedPieces[PLAYER_1]).toEqual([]);
    expect(gameState.capturedPieces[PLAYER_2]).toEqual([]);
    expect(gameState.moveHistory).toEqual([]);
    expect(gameState.isCheck).toBe(false);
    expect(gameState.isCheckmate).toBe(false);

    // Check a few specific pieces
    expect(gameState.board[8][4]).toEqual({ type: KING, player: PLAYER_1 });
    expect(gameState.board[0][4]).toEqual({ type: KING, player: PLAYER_2 });
    expect(gameState.board[6][0]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(gameState.board[2][8]).toEqual({ type: PAWN, player: PLAYER_2 });
    expect(gameState.board[7][1]).toEqual({ type: BISHOP, player: PLAYER_1 });
    expect(gameState.board[1][1]).toEqual({ type: ROOK, player: PLAYER_2 });
  });
});

describe('getLegalMoves', () => {
  let board;

  beforeEach(() => {
    board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
  });

  it('should return correct moves for a Pawn (Player 1)', () => {
    board[6][4] = { type: PAWN, player: PLAYER_1 };
    const moves = getLegalMoves(board[6][4], 6, 4, board);
    expect(moves).toEqual([[5, 4]]);
  });

  it('should return correct moves for a Pawn (Player 2)', () => {
    board[2][4] = { type: PAWN, player: PLAYER_2 };
    const moves = getLegalMoves(board[2][4], 2, 4, board);
    expect(moves).toEqual([[3, 4]]);
  });

  it('should return correct moves for a Lance (Player 1)', () => {
    board[8][0] = { type: LANCE, player: PLAYER_1 };
    const moves = getLegalMoves(board[8][0], 8, 0, board);
    expect(moves).toEqual([[7, 0], [6, 0], [5, 0], [4, 0], [3, 0], [2, 0], [1, 0], [0, 0]]);
  });

  it('should return correct moves for a Lance (Player 2)', () => {
    board[0][0] = { type: LANCE, player: PLAYER_2 };
    const moves = getLegalMoves(board[0][0], 0, 0, board);
    expect(moves).toEqual([[1, 0], [2, 0], [3, 0], [4, 0], [5, 0], [6, 0], [7, 0], [8, 0]]);
  });

  it('should return correct moves for a Knight (Player 1)', () => {
    board[5][4] = { type: KNIGHT, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[3, 3], [3, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Knight (Player 2)', () => {
    board[3][4] = { type: KNIGHT, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[5, 3], [5, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Silver General (Player 1)', () => {
    board[5][4] = { type: SILVER, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[4, 4], [4, 3], [4, 5], [6, 3], [6, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Silver General (Player 2)', () => {
    board[3][4] = { type: SILVER, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[4, 4], [4, 3], [4, 5], [2, 3], [2, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Gold General (Player 1)', () => {
    board[5][4] = { type: GOLD, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[4, 4], [6, 4], [5, 3], [5, 5], [4, 3], [4, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Gold General (Player 2)', () => {
    board[3][4] = { type: GOLD, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([[4, 4], [2, 4], [3, 3], [3, 5], [4, 3], [4, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a King', () => {
    board[4][4] = { type: KING, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([
      [3, 3], [3, 4], [3, 5],
      [4, 3],         [4, 5],
      [5, 3], [5, 4], [5, 5],
    ].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });

  it('should return correct moves for a Rook', () => {
    board[4][4] = { type: ROOK, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    expect(moves.length).toBe(16); // 8 horizontal + 8 vertical - 1 (current pos)
  });

  it('should return correct moves for a Bishop', () => {
    board[4][4] = { type: BISHOP, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    expect(moves.length).toBe(16); // 8 diagonal - 1 (current pos)
  });

  it('should not allow capturing own pieces', () => {
    board[6][4] = { type: PAWN, player: PLAYER_1 };
    board[5][4] = { type: PAWN, player: PLAYER_1 };
    const moves = getLegalMoves(board[6][4], 6, 4, board);
    expect(moves).toEqual([]);
  });

  it('should allow capturing opponent pieces', () => {
    board[6][4] = { type: PAWN, player: PLAYER_1 };
    board[5][4] = { type: PAWN, player: PLAYER_2 };
    const moves = getLegalMoves(board[6][4], 6, 4, board);
    expect(moves).toEqual([[5, 4]]);
  });

  it('should return correct moves for a Promoted Rook (Dragon King)', () => {
    board[4][4] = { type: PROMOTED_ROOK, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    expect(moves.length).toBe(20); // Rook moves (16) + King moves (8) - overlaps (4)
  });

  it('should return correct moves for a Promoted Bishop (Dragon Horse)', () => {
    board[4][4] = { type: PROMOTED_BISHOP, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    expect(moves.length).toBe(20); // Bishop moves (16) + King moves (8) - overlaps (4)
  });

  it('should return correct moves for a Promoted Pawn (Gold General)', () => {
    board[5][4] = { type: PROMOTED_PAWN, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual([
      [4, 4], [6, 4], [5, 3], [5, 5], [4, 3], [4, 5]
    ].sort((a, b) => a[0] - b[0] || a[1] - b[1]));
  });
});

describe('movePiece', () => {
  it('should move a piece to an empty square', () => {
    const initialState = getInitialGameState();
    const from = [6, 4]; // Player 1 Pawn
    const to = [5, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[5][4]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(newState.board[6][4]).toBeNull();
    expect(newState.currentPlayer).toBe(PLAYER_2);
    expect(newState.moveHistory.length).toBe(1);
  });

  it('should capture an opponent\'s piece', () => {
    const initialState = getInitialGameState();
    // Move P1 Pawn to capture P2 Pawn
    initialState.board[5][4] = { type: PAWN, player: PLAYER_2 }; // Place P2 Pawn for capture
    const from = [6, 4]; // Player 1 Pawn
    const to = [5, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[5][4]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(newState.board[6][4]).toBeNull();
    expect(newState.capturedPieces[PLAYER_1]).toEqual([{ type: PAWN, player: PLAYER_2 }]);
    expect(newState.currentPlayer).toBe(PLAYER_2);
  });

  it('should revert promoted piece to original type when captured', () => {
    const initialState = getInitialGameState();
    initialState.board[5][4] = { type: PROMOTED_ROOK, player: PLAYER_2 };
    const from = [6, 4]; // Player 1 Pawn
    const to = [5, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.capturedPieces[PLAYER_1]).toEqual([{ type: ROOK, player: PLAYER_2 }]);
  });

  it('should set promotionPending for optional promotion', () => {
    const initialState = getInitialGameState();
    initialState.board[2][4] = { type: SILVER, player: PLAYER_1 }; // P1 Silver near promotion zone
    const from = [2, 4];
    const to = [1, 4]; // Into promotion zone
    const newState = movePiece(initialState, from, to);

    expect(newState.promotionPending).toEqual({
      from: [2, 4],
      to: [1, 4],
      piece: { type: SILVER, player: PLAYER_1 },
    });
    expect(newState.board[1][4]).toBeNull(); // Board not updated yet
  });

  it('should automatically promote for mandatory promotion (Pawn)', () => {
    const initialState = getInitialGameState();
    initialState.board[1][4] = { type: PAWN, player: PLAYER_1 };
    const from = [1, 4];
    const to = [0, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[0][4]).toEqual({ type: PROMOTED_PAWN, player: PLAYER_1 });
    expect(newState.promotionPending).toBeNull();
  });

  it('should automatically promote for mandatory promotion (Knight)', () => {
    const initialState = getInitialGameState();
    initialState.board[2][4] = { type: KNIGHT, player: PLAYER_1 };
    const from = [2, 4];
    const to = [0, 3];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[0][3]).toEqual({ type: PROMOTED_KNIGHT, player: PLAYER_1 });
    expect(newState.promotionPending).toBeNull();
  });
});

describe('dropPiece', () => {
  it('should drop a piece onto an empty square', () => {
    const initialState = getInitialGameState();
    // Clear the pawn at [6,5] to avoid Nifu rule for this test
    initialState.board[6][5] = null;
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // Corrected player
    const newState = dropPiece(initialState, PAWN, [5, 5]);

    expect(newState.board[5][5]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(newState.capturedPieces[PLAYER_1]).toEqual([]);
    expect(newState.currentPlayer).toBe(PLAYER_2);
  });

  it('should not drop a piece on an occupied square', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    initialState.board[5][5] = { type: PAWN, player: PLAYER_2 };
    const newState = dropPiece(initialState, PAWN, [5, 5]);

    expect(newState).toBe(initialState); // Should return the same state
  });

  it('should enforce Nifu rule (cannot drop pawn in file with another unpromoted pawn)', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    initialState.board[5][5] = { type: PAWN, player: PLAYER_1 }; // Existing pawn in the column
    const newState = dropPiece(initialState, PAWN, [4, 5]);

    expect(newState).toBe(initialState);
  });

  it('should not drop a pawn on the last rank', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    const newState = dropPiece(initialState, PAWN, [0, 4]);

    expect(newState).toBe(initialState);
  });

  it('should not drop a lance on the last rank', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: LANCE, player: PLAYER_2 });
    const newState = dropPiece(initialState, LANCE, [0, 4]);

    expect(newState).toBe(initialState);
  });

  it('should not drop a knight on the last two ranks', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: KNIGHT, player: PLAYER_2 });
    const newState = dropPiece(initialState, KNIGHT, [1, 4]);

    expect(newState).toBe(initialState);
  });
});

describe('isKingInCheck', () => {
  let board;

  beforeEach(() => {
    board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
  });

  it('should return false if king is not in check', () => {
    board[8][4] = { type: KING, player: PLAYER_1 };
    expect(isKingInCheck(board, PLAYER_1)).toBe(false);
  });

  it('should return true if king is in check by a pawn', () => {
    board[8][4] = { type: KING, player: PLAYER_1 };
    board[7][4] = { type: PAWN, player: PLAYER_2 };
    expect(isKingInCheck(board, PLAYER_1)).toBe(true);
  });

  it('should return true if king is in check by a rook', () => {
    board[8][4] = { type: KING, player: PLAYER_1 };
    board[8][0] = { type: ROOK, player: PLAYER_2 };
    expect(isKingInCheck(board, PLAYER_1)).toBe(true);
  });

  it('should return true if king is in check by a bishop', () => {
    board[8][4] = { type: KING, player: PLAYER_1 };
    board[4][0] = { type: BISHOP, player: PLAYER_2 };
    expect(isKingInCheck(board, PLAYER_1)).toBe(true);
  });
});

describe('isCheckmate', () => {
  let gameState;

  beforeEach(() => {
    gameState = getInitialGameState();
    // Clear board for specific test scenarios
    gameState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
  });

  it('should return false if king is not in check', () => {
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    expect(isCheckmate(gameState)).toBe(false);
  });

  it('should return false if king can move out of check', () => {
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    gameState.board[7][4] = { type: ROOK, player: PLAYER_2 }; // Checking rook
    expect(isCheckmate(gameState)).toBe(false);
  });

  it('should return true for a simple checkmate', () => {
    // Player 1 King at [8,4]
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    // Player 2 Rooks at [8,3] and [8,5] (flanking checkmate)
    gameState.board[8][3] = { type: ROOK, player: PLAYER_2 };
    gameState.board[8][5] = { type: ROOK, player: PLAYER_2 };

    // Ensure no escape squares
    gameState.board[7][3] = { type: PAWN, player: PLAYER_1 };
    gameState.board[7][4] = { type: PAWN, player: PLAYER_1 };
    gameState.board[7][5] = { type: PAWN, player: PLAYER_1 };

    expect(isCheckmate(gameState)).toBe(true);
  });

  it('should return false if a piece can block the check', () => {
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    gameState.board[6][4] = { type: ROOK, player: PLAYER_2 }; // Checking rook
    gameState.board[7][4] = { type: PAWN, player: PLAYER_1 }; // Blocking pawn
    expect(isCheckmate(gameState)).toBe(false);
  });

  it('should return false if a piece can capture the checking piece', () => {
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    gameState.board[7][4] = { type: ROOK, player: PLAYER_2 }; // Checking rook
    gameState.board[7][3] = { type: GOLD, player: PLAYER_1 }; // Capturing gold
    expect(isCheckmate(gameState)).toBe(false);
  });

  it('should return false if a captured piece can be dropped to block check', () => {
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    gameState.board[6][4] = { type: ROOK, player: PLAYER_2 }; // Checking rook
    gameState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 }); // Captured pawn to drop
    expect(isCheckmate(gameState)).toBe(false);
  });
});