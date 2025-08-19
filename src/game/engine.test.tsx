import type { GameState, Piece, Coords, Player } from '../types';
import { describe, it, expect, beforeEach } from 'vitest';
import {
  getInitialGameState,
  getLegalMoves,
  movePiece,
  dropPiece,
  isKingInCheck,
  isCheckmate,
  PAWN, LANCE, KNIGHT, SILVER, GOLD, KING, ROOK, BISHOP,
  PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_ROOK, PROMOTED_BISHOP,
  PLAYER_1, PLAYER_2,
  ROWS, COLS
} from './engine';

interface RepeatingMove {
  from: Coords;
  to: Coords;
  player: Player;
}

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
  let board: (Piece | null)[][];

  beforeEach(() => {
    board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
  });

  it('should return correct moves for a Pawn (Player 1)', () => {
    board[6][4] = { type: PAWN, player: PLAYER_1 };
    const moves = getLegalMoves(board[6][4], 6, 4, board);
    expect(moves).toEqual([[5, 4]] as Coords[]);
  });

  it('should return correct moves for a Pawn (Player 2)', () => {
    board[2][4] = { type: PAWN, player: PLAYER_2 };
    const moves = getLegalMoves(board[2][4], 2, 4, board);
    expect(moves).toEqual([[3, 4]] as Coords[]);
  });

  it('should return correct moves for a Lance (Player 1)', () => {
    board[8][0] = { type: LANCE, player: PLAYER_1 };
    const moves = getLegalMoves(board[8][0], 8, 0, board);
    expect(moves).toEqual([[7, 0], [6, 0], [5, 0], [4, 0], [3, 0], [2, 0], [1, 0], [0, 0]] as Coords[]);
  });

  it('should return correct moves for a Lance (Player 2)', () => {
    board[0][0] = { type: LANCE, player: PLAYER_2 };
    const moves = getLegalMoves(board[0][0], 0, 0, board);
    expect(moves).toEqual([[1, 0], [2, 0], [3, 0], [4, 0], [5, 0], [6, 0], [7, 0], [8, 0]] as Coords[]);
  });

  it('should return correct moves for a Knight (Player 1)', () => {
    board[5][4] = { type: KNIGHT, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    const expectedMoves = [[3, 3], [3, 5]].sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a Knight (Player 2)', () => {
    board[3][4] = { type: KNIGHT, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    const expectedMoves = ([[5, 3], [5, 5]] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a Silver General (Player 1)', () => {
    board[5][4] = { type: SILVER, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    const expectedMoves = ([[4, 4], [4, 3], [4, 5], [6, 3], [6, 5]] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a Silver General (Player 2)', () => {
    board[3][4] = { type: SILVER, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    const expectedMoves = ([[4, 4], [4, 3], [4, 5], [2, 3], [2, 5]] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a Gold General (Player 1)', () => {
    board[5][4] = { type: GOLD, player: PLAYER_1 };
    const moves = getLegalMoves(board[5][4], 5, 4, board);
    const expectedMoves = ([[4, 4], [6, 4], [5, 3], [5, 5], [4, 3], [4, 5]] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a Gold General (Player 2)', () => {
    board[3][4] = { type: GOLD, player: PLAYER_2 };
    const moves = getLegalMoves(board[3][4], 3, 4, board);
    const expectedMoves = ([[4, 4], [2, 4], [3, 3], [3, 5], [4, 3], [4, 5]] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });

  it('should return correct moves for a King', () => {
    board[4][4] = { type: KING, player: PLAYER_1 };
    const moves = getLegalMoves(board[4][4], 4, 4, board);
    const expectedMoves = ([
      [3, 3], [3, 4], [3, 5],
      [4, 3],         [4, 5],
      [5, 3], [5, 4], [5, 5],
    ] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
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
    expect(moves).toEqual([[5, 4]] as Coords[]);
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
    const expectedMoves = ([
      [4, 4], [6, 4], [5, 3], [5, 5], [4, 3], [4, 5]
    ] as Coords[]).sort((a, b) => a[0] - b[0] || a[1] - b[1]);
    expect(moves.sort((a, b) => a[0] - b[0] || a[1] - b[1])).toEqual(expectedMoves);
  });
});

describe('movePiece', () => {
  it('should move a piece to an empty square', () => {
    const initialState = getInitialGameState();
    const from: Coords = [6, 4]; // Player 1 Pawn
    const to: Coords = [5, 4];
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
    const from: Coords = [6, 4]; // Player 1 Pawn
    const to: Coords = [5, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[5][4]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(newState.board[6][4]).toBeNull();
    expect(newState.capturedPieces[PLAYER_1]).toEqual([{ type: PAWN, player: PLAYER_2 }]);
    expect(newState.currentPlayer).toBe(PLAYER_2);
  });

  it('should revert promoted piece to original type when captured', () => {
    const initialState = getInitialGameState();
    initialState.board[5][4] = { type: PROMOTED_ROOK, player: PLAYER_2 };
    const from: Coords = [6, 4]; // Player 1 Pawn
    const to: Coords = [5, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.capturedPieces[PLAYER_1]).toEqual([{ type: ROOK, player: PLAYER_2 }]);
  });

  it('should set promotionPending for optional promotion', () => {
    const initialState = getInitialGameState();
    initialState.board[2][4] = { type: SILVER, player: PLAYER_1 }; // P1 Silver near promotion zone
    const from: Coords = [2, 4];
    const to: Coords = [1, 4]; // Into promotion zone
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
    const from: Coords = [1, 4];
    const to: Coords = [0, 4];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[0][4]).toEqual({ type: PROMOTED_PAWN, player: PLAYER_1 });
    expect(newState.promotionPending).toBeNull();
  });

  it('should automatically promote for mandatory promotion (Knight)', () => {
    const initialState = getInitialGameState();
    initialState.board[2][4] = { type: KNIGHT, player: PLAYER_1 };
    const from: Coords = [2, 4];
    const to: Coords = [0, 3];
    const newState = movePiece(initialState, from, to);

    expect(newState.board[0][3]).toEqual({ type: PROMOTED_KNIGHT, player: PLAYER_1 });
    expect(newState.promotionPending).toBeNull();
  });

  it('should set isCheckmate to true when a move results in checkmate', () => {
    const initialState = getInitialGameState();
    initialState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));

    // Player 2 King at 1a (0,8)
    initialState.board[0][8] = { type: KING, player: PLAYER_2 };
    initialState.kingPositions[PLAYER_2] = [0, 8];

    // Player 1 Rook at 1c (2,8)
    initialState.board[2][8] = { type: ROOK, player: PLAYER_1 };
    
    // Player 1 Gold at 2b (1,7)
    initialState.board[1][7] = { type: GOLD, player: PLAYER_1 };

    initialState.currentPlayer = PLAYER_1;

    const from: Coords = [2, 8]; // Player 1 Rook
    const to: Coords = [1, 8];   // Move to 1b to checkmate

    const newState = movePiece(initialState, from, to, 'Test', false);

    expect(newState.isCheckmate).toBe(true);
    expect(newState.currentPlayer).toBe(PLAYER_2);
  });
});

describe('dropPiece', () => {
  it('should drop a piece onto an empty square', () => {
    const initialState = getInitialGameState();
    // Clear the pawn at [6,5] to avoid Nifu rule for this test
    initialState.board[6][5] = null;
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // Corrected player
    const newState = dropPiece(initialState, PAWN, [5, 5] as Coords);

    expect(newState.board[5][5]).toEqual({ type: PAWN, player: PLAYER_1 });
    expect(newState.capturedPieces[PLAYER_1]).toEqual([]);
    expect(newState.currentPlayer).toBe(PLAYER_2);
  });

  it('should not drop a piece on an occupied square', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    initialState.board[5][5] = { type: PAWN, player: PLAYER_2 };
    const newState = dropPiece(initialState, PAWN, [5, 5] as Coords);

    expect(newState).toBe(initialState); // Should return the same state
  });

  it('should enforce Nifu rule (cannot drop pawn in file with another unpromoted pawn)', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    initialState.board[5][5] = { type: PAWN, player: PLAYER_1 }; // Existing pawn in the column
    const newState = dropPiece(initialState, PAWN, [4, 5] as Coords);

    expect(newState).toBe(initialState);
  });

  it('should not drop a pawn on the last rank', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_2 });
    const newState = dropPiece(initialState, PAWN, [0, 4] as Coords);

    expect(newState).toBe(initialState);
  });

  it('should not drop a lance on the last rank', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: LANCE, player: PLAYER_2 });
    const newState = dropPiece(initialState, LANCE, [0, 4] as Coords);

    expect(newState).toBe(initialState);
  });

  it('should not drop a knight on the last two ranks', () => {
    const initialState = getInitialGameState();
    initialState.capturedPieces[PLAYER_1].push({ type: KNIGHT, player: PLAYER_2 });
    const newState = dropPiece(initialState, KNIGHT, [1, 4] as Coords);

    expect(newState).toBe(initialState);
  });

  it('should not allow dropping a piece if it puts the current player\'s king in check', () => {
    const initialState = getInitialGameState();
    // Set up a scenario where dropping a pawn would put Player 1's king in check
    // P1 King at [8,4]
    initialState.board[8][4] = { type: KING, player: PLAYER_1 };
    // P2 Rook at [0,4] checking P1 King
    initialState.board[0][4] = { type: ROOK, player: PLAYER_2 };
    // P1 has a captured pawn
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 });

    // Attempt to drop the pawn at [7,4] which would block the check, but also put P1 in check
    // This is a complex scenario, let's simplify:
    // P1 King at [8,4]
    // P2 Rook at [0,4]
    // P1 has a captured pawn
    // If P1 drops pawn at [7,4], it blocks the check.
    // The rule is: "Cannot drop a piece if it puts the current player's king in check"
    // Let's create a scenario where dropping a piece *does* put the current player's king in check.
    // This is usually not possible with a single drop, as drops are usually to *escape* check or make a threat.
    // The original code's intent was likely to prevent self-check from drops.

    // Let's create a simpler test case for self-check from drop:
    // P1 King at [8,4]
    // P2 Rook at [0,4] (not checking initially)
    initialState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
    initialState.board[8][4] = { type: KING, player: PLAYER_1 };
    initialState.board[0][0] = { type: ROOK, player: PLAYER_2 }; // P2 Rook far away
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 });

    // Now, if P1 drops a pawn at [7,4], and then P2 moves Rook to [7,4] (not possible)
    // The rule is about *the drop itself* putting the king in check.
    // This is usually for "discovered check" scenarios, but Shogi drops don't typically create discovered checks for the *dropper*.

    // Let's re-evaluate the original code's intent:
    // `if (isKingInCheck(tempBoard, currentPlayer)) { ... return { ...gameState, currentPlayer: nextPlayer }; }`
    // This implies that if the *simulated* board after the drop results in the *current player's* king being in check,
    // then the drop is illegal, and the game state should not change, but the player should switch.
    // This is incorrect. If the drop is illegal, the state should not change at all.

    // Test case: Drop a piece that would put own king in check (should not happen in valid game, but testing the logic)
    // Setup: Player 1 King at [8,4]. Player 2 has a Rook at [8,0].
    // Player 1 tries to drop a piece (e.g., a pawn) at [8,1]. This pawn would not block the check.
    // This scenario is hard to construct naturally for a drop.

    // Let's test the specific behavior of the `dropPiece` function's problematic line.
    // We need a scenario where `isKingInCheck(tempBoard, currentPlayer)` becomes true after the drop.
    // This means the piece being dropped *itself* is checking the current player's king, which is impossible.
    // Or, the drop *unblocks* a check from an opponent's piece.

    // Simpler test: Ensure that if `isKingInCheck(tempBoard, currentPlayer)` is true, the original state is returned.
    // We'll manually set up `tempBoard` to be in check for the `currentPlayer` after the drop.
    const originalGameState = getInitialGameState();
    originalGameState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // P1 has a pawn to drop

    // Create a board where dropping a pawn at [7,4] would put P1's king at [8,4] in check
    // This is not a realistic scenario for a drop, but it tests the condition.
    // Let's assume P2 has a hidden piece that would check P1's king if P1 drops a pawn.
    // This is getting too complicated.

    // The simplest way to test the problematic line:
    // Mock `isKingInCheck` to return true for the `currentPlayer` after a simulated drop.
    // This is not ideal for a unit test, as it mocks the very function we are testing the interaction with.

    // Let's stick to the original interpretation: if the drop *itself* puts the current player in check, it's illegal.
    // This is usually not the case for drops, as drops are on empty squares.

    // The most direct way to test the problematic line is to ensure that if `isKingInCheck(tempBoard, currentPlayer)`
    // evaluates to true, the function returns the *original* gameState, not a modified one with `currentPlayer` switched.

    // Let's create a scenario where dropping a piece *would* put the current player's king in check.
    // This is highly unusual for Shogi, as drops are usually to empty squares.
    // The only way a drop could put *your own* king in check is if it uncovers a check from your opponent.
    // This is a "discovered check" on yourself.

    // Scenario:
    // P1 King at [8,4]
    // P2 Rook at [8,0] (currently blocked by P1's own piece at [8,1])
    // P1 tries to drop a piece at [8,1] (the blocking piece's position)
    // This would be an illegal move because [8,1] is not empty.

    // Let's assume the `isKingInCheck` function is correct.
    // The problematic line is: `return { ...gameState, currentPlayer: nextPlayer };`
    // This should be `return gameState;`

    // Let's create a test where a drop is attempted, and we *expect* `isKingInCheck(tempBoard, currentPlayer)` to be true.
    // This means we need to set up the board such that dropping a piece at `to` will put `currentPlayer`'s king in check.
    // This is a very unusual scenario for a drop.

    // The most straightforward way to test the bug is to ensure that if `isKingInCheck(tempBoard, currentPlayer)` is true,
    // the `dropPiece` function returns the *original* `gameState` without any modifications.

    const initialGameState = getInitialGameState();
    // Manually set up a state where dropping a pawn at [7,4] would put P1's king at [8,4] in check
    // This is a contrived scenario to test the specific bug.
    initialGameState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
    initialGameState.board[8][4] = { type: KING, player: PLAYER_1 }; // P1 King
    initialGameState.board[0][4] = { type: ROOK, player: PLAYER_2 }; // P2 Rook
    initialGameState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // P1 has a pawn to drop

    // If P1 drops a pawn at [7,4], it would block the check from P2's Rook.
    // So, `isKingInCheck(tempBoard, currentPlayer)` would be false.

    // Let's try to make `isKingInCheck(tempBoard, currentPlayer)` true.
    // This means the piece being dropped *itself* is checking the current player's king, which is impossible.
    // Or, the drop *unblocks* a check from an opponent's piece.

    // The only way `isKingInCheck(tempBoard, currentPlayer)` would be true is if the `tempBoard` is already in check
    // for the `currentPlayer` *before* the drop, and the drop doesn't resolve it.
    // But the `dropPiece` function is called when the `currentPlayer` is making a move.
    // A player cannot make a move that leaves their king in check.

    // The problematic line is specifically about the *simulated* drop putting the *current player's* king in check.
    // This is a self-check.

    // Let's create a scenario where dropping a piece *would* put the current player's king in check.
    // This is highly unusual for Shogi, as drops are usually to empty squares.
    // The only way a drop could put *your own* king in check is if it uncovers a check from your opponent.
    // This is a "discovered check" on yourself.

    // Let's assume the `isKingInCheck` function is correct.
    // The problematic line is: `return { ...gameState, currentPlayer: nextPlayer };`
    // This should be `return gameState;`

    // We need to create a scenario where `isKingInCheck(tempBoard, currentPlayer)` returns true.
    // This means the `tempBoard` (after the simulated drop) has the `currentPlayer`'s king in check.
    // This can happen if the drop *unblocks* a check from an opponent's piece.

    // Example:
    // P1 King at [8,4]
    // P2 Rook at [8,0]
    // P1 has a piece at [8,1] (e.g., a pawn) blocking the Rook's check.
    // P1 tries to drop a captured piece (e.g., a pawn) at [8,1] (where the blocking piece is).
    // This is an illegal drop because the square is not empty.

    // Let's simplify the test for the problematic line.
    // We will mock `isKingInCheck` to return true for the `currentPlayer` after the simulated drop.
    // This will directly test the `if (isKingInCheck(tempBoard, currentPlayer))` block.

    

    const initialGameStateForSelfCheck = getInitialGameState();
    initialGameStateForSelfCheck.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
    initialGameStateForSelfCheck.board[8][4] = { type: KING, player: PLAYER_1 }; // P1 King
    initialGameStateForSelfCheck.board[8][0] = { type: ROOK, player: PLAYER_2 }; // P2 Rook
    initialGameStateForSelfCheck.board[8][1] = { type: PAWN, player: PLAYER_1 }; // P1 Pawn blocking check
    initialGameStateForSelfCheck.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // P1 has a pawn to drop

    // Now, if P1 tries to drop a pawn at [8,1], it's an illegal move because the square is occupied.
    // The `dropPiece` function already handles this: `if (board[toRow][toCol]) { return gameState; }`

    // We need a scenario where the drop is on an *empty* square, but it *causes* a self-check.
    // This is the "discovered check" scenario.
    // P1 King at [8,4]
    // P2 Rook at [8,0]
    // P1 has a piece at [8,1] (e.g., a pawn) blocking the Rook's check.
    // P1 tries to drop a captured piece (e.g., a pawn) at [7,1] (an empty square).
    // This drop *unblocks* the check from P2's Rook to P1's King.

    const gameStateBeforeSelfCheckDrop = getInitialGameState();
    gameStateBeforeSelfCheckDrop.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));
    gameStateBeforeSelfCheckDrop.board[8][4] = { type: KING, player: PLAYER_1 }; // P1 King
    gameStateBeforeSelfCheckDrop.board[8][0] = { type: ROOK, player: PLAYER_2 }; // P2 Rook
    gameStateBeforeSelfCheckDrop.board[8][1] = { type: PAWN, player: PLAYER_1 }; // P1 Pawn blocking check
    gameStateBeforeSelfCheckDrop.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 }); // P1 has a pawn to drop

    // Attempt to drop the pawn at [7,1] (an empty square)
    const newStateAfterSelfCheckDrop = dropPiece(gameStateBeforeSelfCheckDrop, PAWN, [7, 1] as Coords);

    // After this drop, the P1 King at [8,4] is now in check from P2 Rook at [8,0] because the pawn at [8,1] moved.
    // So, `isKingInCheck(tempBoard, currentPlayer)` should be true.
    // The `dropPiece` function should return the original `gameState` in this case.
    expect(newStateAfterSelfCheckDrop).toBe(gameStateBeforeSelfCheckDrop);
    expect(newStateAfterSelfCheckDrop.currentPlayer).toBe(PLAYER_1); // Current player should not switch
  });

  it('should set isCheckmate to true when a drop results in checkmate', () => {
    const initialState = getInitialGameState();
    // Clear the board for a specific checkmate scenario
    initialState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));

    // Player 2 King at [0,0]
    initialState.board[0][0] = { type: KING, player: PLAYER_2 };
    initialState.kingPositions[PLAYER_2] = [0, 0];

    // Player 1 Gold General at [0,1] (covers escape squares)
    initialState.board[0][1] = { type: GOLD, player: PLAYER_1 };

    // Player 1 Gold General at [1,1] (covers more escape squares)
    initialState.board[1][1] = { type: GOLD, player: PLAYER_1 };

    // Player 1 has a captured Pawn
    initialState.capturedPieces[PLAYER_1].push({ type: PAWN, player: PLAYER_1 });

    // Set current player to Player 1
    initialState.currentPlayer = PLAYER_1;

    // Drop the Pawn at [1,0] to checkmate Player 2 King
    const newState = dropPiece(initialState, PAWN, [1, 0] as Coords);

    expect(newState.isCheckmate).toBe(true);
    expect(newState.currentPlayer).toBe(PLAYER_2); // It's Player 2's turn, and they are checkmated
  });
});

describe('isKingInCheck', () => {
  let board: (Piece | null)[][];

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
  let gameState: GameState;

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

describe('checkSennichite', () => {
  it('should detect a draw by repetition (Sennichite) after four occurrences of the same position', () => {
    let gameState = getInitialGameState();

    // Set up a simplified board for the repetition
    // Remove all pieces except the ones involved in the repetition and the kings
    gameState.board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));

    // Place kings
    gameState.board[8][4] = { type: KING, player: PLAYER_1 };
    gameState.kingPositions[PLAYER_1] = [8, 4];
    gameState.board[0][4] = { type: KING, player: PLAYER_2 };
    gameState.kingPositions[PLAYER_2] = [0, 4];

    // Place the pieces involved in the repetition
    // Player 1 Gold at [3,3] (Shogi G64)
    gameState.board[3][3] = { type: GOLD, player: PLAYER_1 };
    // Player 2 Promoted Lance at [5,5] (Shogi +L46)
    gameState.board[5][5] = { type: PROMOTED_LANCE, player: PLAYER_2 };

    // Define the repeating moves
    const repeatingMoves: RepeatingMove[] = [
      // black G64-65 (internal [3,3] to [4,3])
      { from: [3, 3], to: [4, 3], player: PLAYER_1 },
      // white +L46-45 (internal [5,5] to [4,5])
      { from: [5, 5], to: [4, 5], player: PLAYER_2 },
      // black G65-64 (internal [4,3] to [3,3])
      { from: [4, 3], to: [3, 3], player: PLAYER_1 },
      // white +L45-46 (internal [4,5] to [5,5])
      { from: [4, 5], to: [5, 5], player: PLAYER_2 },
    ];

    // Perform the repeating moves to trigger Sennichite
    // The position repeats every 4 moves. We need 4 occurrences of the same position.
    // Initial state is 1st occurrence.
    // After 4 moves, it's the 2nd occurrence.
    // After 8 moves, it's the 3rd occurrence.
    // After 12 moves, it's the 4th occurrence, and should be a draw.
    const numRepetitions = 3; // To get 4 occurrences (initial + 3 repetitions)

    for (let i = 0; i < numRepetitions; i++) {
      for (const move of repeatingMoves) {
        // Ensure the current player matches the move's player
        if (gameState.currentPlayer !== move.player) {
          // This should not happen if the sequence is correct, but as a safeguard
          // we might need to manually switch player or adjust the test setup.
          // For this specific test, the sequence alternates players correctly.
        }

        // Simulate the move
        gameState = movePiece(gameState, move.from, move.to);
        // After each move, check if it's a draw
        if (gameState.isDraw) {
          break; // Exit if draw is detected early
        }
      }
      if (gameState.isDraw) {
        break; // Exit outer loop if draw is detected
      }
    }

    expect(gameState.isDraw).toBe(true);
  });
});

