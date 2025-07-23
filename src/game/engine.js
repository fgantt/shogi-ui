export const ROWS = 9;
export const COLS = 9;

// Piece Types
export const KING = 'K';
export const ROOK = 'R';
export const BISHOP = 'B';
export const GOLD = 'G';
export const SILVER = 'S';
export const KNIGHT = 'N';
export const LANCE = 'L';
export const PAWN = 'P';

// Promoted Piece Types
export const PROMOTED_ROOK = '+R';
export const PROMOTED_BISHOP = '+B';
export const PROMOTED_SILVER = '+S';
export const PROMOTED_KNIGHT = '+N';
export const PROMOTED_LANCE = '+L';
export const PROMOTED_PAWN = '+P';

// Players
export const PLAYER_1 = 'player1';
export const PLAYER_2 = 'player2';

/**
 * Creates the initial game state.
 * @returns {object} The initial game state.
 */
export function getInitialGameState() {
  const board = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));

  // Helper to create a piece
  const piece = (type, player) => ({ type, player });

  // Player 1 (bottom)
  board[8] = [
    piece(LANCE, PLAYER_1), piece(KNIGHT, PLAYER_1), piece(SILVER, PLAYER_1), piece(GOLD, PLAYER_1), piece(KING, PLAYER_1), piece(GOLD, PLAYER_1), piece(SILVER, PLAYER_1), piece(KNIGHT, PLAYER_1), piece(LANCE, PLAYER_1)
  ];
  board[7] = [
    null, piece(BISHOP, PLAYER_1), null, null, null, null, null, piece(ROOK, PLAYER_1), null
  ];
  for (let i = 0; i < COLS; i++) {
    board[6][i] = piece(PAWN, PLAYER_1);
  }

  // Player 2 (top)
  board[0] = [
    piece(LANCE, PLAYER_2), piece(KNIGHT, PLAYER_2), piece(SILVER, PLAYER_2), piece(GOLD, PLAYER_2), piece(KING, PLAYER_2), piece(GOLD, PLAYER_2), piece(SILVER, PLAYER_2), piece(KNIGHT, PLAYER_2), piece(LANCE, PLAYER_2)
  ];
  board[1] = [
    null, piece(ROOK, PLAYER_2), null, null, null, null, null, piece(BISHOP, PLAYER_2), null
  ];
  for (let i = 0; i < COLS; i++) {
    board[2][i] = piece(PAWN, PLAYER_2);
  }


  return {
    board,
    currentPlayer: PLAYER_1,
    capturedPieces: {
      [PLAYER_1]: [],
      [PLAYER_2]: []
    },
    moveHistory: [],
    isCheck: false,
    isCheckmate: false,
    pastStates: [], // Add pastStates to store previous game states
    kingPositions: {
      [PLAYER_1]: [8, 4],
      [PLAYER_2]: [0, 4],
    },
  };
}

/**
 * Gets all legal moves for a piece at a given position.
 * @param {object} piece The piece to get moves for.
 * @param {number} row The row of the piece.
 * @param {number} col The column of the piece.
 * @param {Array<Array<object>>} board The current board state.
 * @returns {Array<[number, number]>} An array of [row, col] pairs representing legal moves.
 */
export function getLegalMoves(piece, row, col, board) {
  const moves = [];
  if (!piece) {
    return moves;
  }

  const { type, player } = piece;
  const player_mult = player === PLAYER_1 ? -1 : 1;

  const canMove = (r, c) => {
    if (r < 0 || r >= ROWS || c < 0 || c >= COLS) return false; // Off-board
    const targetPiece = board[r][c];
    if (targetPiece && targetPiece.player === player) return false; // Cannot capture own piece
    return true;
  };

  const addMove = (r, c) => {
    if (canMove(r, c)) {
      moves.push([r, c]);
    }
  };

  // Helper for sliding pieces (Rook, Bishop, Lance)
  const addSlidingMoves = (directions) => {
    for (const [dr, dc] of directions) {
      let r = row + dr;
      let c = col + dc;
      while (r >= 0 && r < ROWS && c >= 0 && c < COLS) {
        const targetPiece = board[r][c];
        if (targetPiece) {
          if (targetPiece.player !== player) {
            moves.push([r, c]); // Can capture
          }
          break; // Blocked by a piece
        }
        moves.push([r, c]); // Empty square
        r += dr;
        c += dc;
      }
    }
  };

  const getGoldMoves = () => {
      const goldMoves = [
        [row + player_mult, col],
        [row - player_mult, col],
        [row, col + 1],
        [row, col - 1],
        [row + player_mult, col + 1],
        [row + player_mult, col - 1],
      ];
      goldMoves.forEach(([r, c]) => addMove(r, c));
  }

  switch (type) {
    case PAWN:
      addMove(row + player_mult, col);
      break;

    case LANCE:
      addSlidingMoves([[player_mult, 0]]);
      break;

    case KNIGHT:
      const knightMoves = [
        [row + player_mult * 2, col + 1],
        [row + player_mult * 2, col - 1],
      ];
      knightMoves.forEach(([r, c]) => addMove(r, c));
      break;

    case SILVER:
      const silverMoves = [
        [row + player_mult, col], // Forward
        [row + player_mult, col + 1], // Forward-right
        [row + player_mult, col - 1], // Forward-left
        [row - player_mult, col + 1], // Backward-right
        [row - player_mult, col - 1], // Backward-left
      ];
      silverMoves.forEach(([r, c]) => addMove(r, c));
      break;

    case GOLD:
    case PROMOTED_PAWN:
    case PROMOTED_LANCE:
    case PROMOTED_KNIGHT:
    case PROMOTED_SILVER:
        getGoldMoves();
        break;

    case BISHOP:
      addSlidingMoves([[-1, -1], [-1, 1], [1, -1], [1, 1]]);
      break;

    case ROOK:
      addSlidingMoves([[-1, 0], [1, 0], [0, -1], [0, 1]]);
      break;

    case KING:
      const kingMoves = [
        [row - 1, col - 1], [row - 1, col], [row - 1, col + 1],
        [row,     col - 1],                 [row,     col + 1],
        [row + 1, col - 1], [row + 1, col], [row + 1, col + 1],
      ];
      kingMoves.forEach(([r, c]) => addMove(r, c));
      break;

    case PROMOTED_BISHOP:
        addSlidingMoves([[-1, -1], [-1, 1], [1, -1], [1, 1]]);
        addMove(row - 1, col);
        addMove(row + 1, col);
        addMove(row, col - 1);
        addMove(row, col + 1);
        break;

    case PROMOTED_ROOK:
        addSlidingMoves([[-1, 0], [1, 0], [0, -1], [0, 1]]);
        addMove(row - 1, col - 1);
        addMove(row - 1, col + 1);
        addMove(row + 1, col - 1);
        addMove(row + 1, col + 1);
        break;
  }

  return moves;
}

/**
 * Moves a piece and updates the game state.
 * This function now handles promotion logic.
 * @param {object} gameState The current game state.
 * @param {[number, number]} from The starting [row, col] of the piece.
 * @param {[number, number]} to The destination [row, col] of the piece.
 * @returns {object} The new game state, possibly with a `promotionPending` flag.
 */
export function movePiece(gameState, from, to) {
  const { board, currentPlayer } = gameState;
  const [fromRow, fromCol] = from;
  const [toRow, toCol] = to;

  const piece = board[fromRow][fromCol];
  if (!piece) return gameState;

  // --- Promotion Logic ---
  const promotionZoneStart = currentPlayer === PLAYER_1 ? 2 : 6;
  const inPromotionZone = (currentPlayer === PLAYER_1 && toRow <= promotionZoneStart) || (currentPlayer === PLAYER_2 && toRow >= promotionZoneStart);
  const wasInPromotionZone = (currentPlayer === PLAYER_1 && fromRow <= promotionZoneStart) || (currentPlayer === PLAYER_2 && fromRow >= promotionZoneStart);

  const promotablePieces = [PAWN, LANCE, KNIGHT, SILVER, BISHOP, ROOK];
  const canPromote = promotablePieces.includes(piece.type) && (inPromotionZone || wasInPromotionZone);

  // Mandatory promotion check
  const lastRank = currentPlayer === PLAYER_1 ? 0 : 8;
  const secondLastRank = currentPlayer === PLAYER_1 ? 1 : 7;
  let isPromotionMandatory = false;
  if ((piece.type === PAWN || piece.type === LANCE) && toRow === lastRank) {
      isPromotionMandatory = true;
  }
  if (piece.type === KNIGHT && (toRow === lastRank || toRow === secondLastRank)) {
      isPromotionMandatory = true;
  }

  if (canPromote && !isPromotionMandatory) {
    // If promotion is optional, return a state that asks the UI for a choice.
    return {
      ...gameState,
      promotionPending: { from, to, piece },
    };
  }

  // If no promotion choice is needed (or it's mandatory), proceed with the move.
  return completeMove(gameState, from, to, isPromotionMandatory);
}

/**
 * Completes a move, including capture and promotion.
 * @param {object} gameState The current game state.
 * @param {[number, number]} from The starting [row, col].
 * @param {[number, number]} to The destination [row, col].
 * @param {boolean} promote Whether to promote the piece.
 * @returns {object} The final new game state.
 */
export function completeMove(gameState, from, to, promote) {
    const { board, currentPlayer, capturedPieces, moveHistory, pastStates } = gameState;
    const newBoard = board.map(row => [...row]);
    const [fromRow, fromCol] = from;
    const [toRow, toCol] = to;

    const piece = { ...newBoard[fromRow][fromCol] }; // Create a copy

    const capturedPiece = newBoard[toRow][toCol];
    const newCapturedPieces = {
        ...capturedPieces,
        [currentPlayer]: [...capturedPieces[currentPlayer]]
    };

    if (capturedPiece) {
        let originalType = capturedPiece.type;
        if (originalType.startsWith('+')) {
            originalType = originalType.substring(1);
        }
        newCapturedPieces[gameState.currentPlayer].push({ type: originalType, player: capturedPiece.player });
    }

    if (promote && !piece.type.startsWith('+')) {
        piece.type = `+${piece.type}`;
    }

    newBoard[toRow][toCol] = piece;
    newBoard[fromRow][fromCol] = null;

    const newMoveHistory = [...moveHistory, { from, to, piece: piece.type, promote }];

    return {
        ...gameState,
        board: newBoard,
        capturedPieces: newCapturedPieces,
        currentPlayer: currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1,
        moveHistory: newMoveHistory,
        promotionPending: null,
        pastStates: [...pastStates, gameState], // Save current state before move
        isCheck: isKingInCheck(newBoard, currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1),
        kingPositions: {
          ...gameState.kingPositions,
          [piece.player]: [toRow, toCol] // Update king position if king moved
        }
    };
}

/**
 * Drops a captured piece onto the board.
 * @param {object} gameState The current game state.
 * @param {string} pieceType The type of piece to drop.
 * @param {[number, number]} to The destination [row, col] of the drop.
 * @returns {object} The new game state.
 */
export function dropPiece(gameState, pieceType, to) {
  const { board, currentPlayer, capturedPieces, pastStates } = gameState;
  const [toRow, toCol] = to;

  // 1. Check if the destination square is empty
  if (board[toRow][toCol]) {
    return gameState; // Can only drop on empty squares
  }

  // 2. Check if the player has the piece to drop
  const capturedPieceIndex = capturedPieces[currentPlayer].findIndex(p => p.type === pieceType);
  if (capturedPieceIndex === -1) {
    return gameState; // Piece not available to drop
  }

  // 3. Check for illegal drop rules
  // Nifu check (two unpromoted pawns in the same file)
  if (pieceType === PAWN) {
    for (let r = 0; r < ROWS; r++) {
      const piece = board[r][toCol];
      if (piece && piece.type === PAWN && piece.player === currentPlayer) {
        return gameState; // Nifu rule violation
      }
    }
  }

  // Cannot drop a piece where it has no legal moves
  const player_mult = currentPlayer === PLAYER_1 ? -1 : 1;
  if ((pieceType === PAWN || pieceType === LANCE) && toRow + player_mult < 0) {
      return gameState;
  }
  if (pieceType === KNIGHT && toRow + (player_mult * 2) < 0) {
      return gameState;
  }


  // 4. Update the board and captured pieces
  const newBoard = board.map(row => [...row]);
  newBoard[toRow][toCol] = { type: pieceType, player: currentPlayer };

  const newCapturedPieces = { ...capturedPieces };
  newCapturedPieces[currentPlayer] = [...capturedPieces[currentPlayer]];
  newCapturedPieces[currentPlayer].splice(capturedPieceIndex, 1);

  const newMoveHistory = [...gameState.moveHistory, { from: 'drop', to, piece: pieceType }];

  return {
    ...gameState,
    board: newBoard,
    capturedPieces: newCapturedPieces,
    currentPlayer: gameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1,
    moveHistory: newMoveHistory,
    pastStates: [...pastStates, gameState], // Save current state before drop
    isCheck: isKingInCheck(newBoard, gameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1),
    kingPositions: {
      ...gameState.kingPositions,
      [currentPlayer]: [toRow, toCol] // Update king position if king was dropped
    }
  };
}

/**
 * Checks if a player's king is in check.
 * @param {Array<Array<object>>} board The board state.
 * @param {string} player The player to check.
 * @returns {boolean} True if the king is in check, false otherwise.
 */
export function isKingInCheck(board, player) {
  const opponent = player === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  let kingPosition = null;

  // Find the king's position
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const piece = board[r][c];
      if (piece && piece.type === KING && piece.player === player) {
        kingPosition = [r, c];
        break;
      }
    }
    if (kingPosition) break;
  }

  if (!kingPosition) return false; // Should not happen in a real game

  // Check if any opponent piece can attack the king
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const piece = board[r][c];
      if (piece && piece.player === opponent) {
        const moves = getLegalMoves(piece, r, c, board);
        for (const move of moves) {
          if (move[0] === kingPosition[0] && move[1] === kingPosition[1]) {
            return true; // King is in check
          }
        }
      }
    }
  }

  return false;
}

/**
 * Checks if a player is in checkmate.
 * @param {object} gameState The current game state.
 * @returns {boolean} True if the current player is in checkmate.
 */
export function isCheckmate(gameState) {
    const { board, currentPlayer, capturedPieces } = gameState;

    if (!isKingInCheck(board, currentPlayer)) {
        return false;
    }

    // Check if any move can get the king out of check
    for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) {
            const piece = board[r][c];
            if (piece && piece.player === currentPlayer) {
                const moves = getLegalMoves(piece, r, c, board);
                for (const move of moves) {
                    // Simulate the move
                    const tempState = movePiece({ ...gameState, promotionPending: null }, [r, c], move);
                    if (!isKingInCheck(tempState.board, currentPlayer)) {
                        return false; // Found a move to escape check
                    }
                }
            }
        }
    }

    // Check if dropping any piece can get the king out of check
    for (const captured of capturedPieces[currentPlayer]) {
        for (let r = 0; r < ROWS; r++) {
            for (let c = 0; c < COLS; c++) {
                if (!board[r][c]) { // Can only drop on empty squares
                    const tempState = dropPiece(gameState, captured.type, [r, c]);
                     if (tempState !== gameState && !isKingInCheck(tempState.board, currentPlayer)) {
                        return false; // Found a drop to escape check
                    }
                }
            }
        }
    }

    return true; // No legal moves to escape check, so it's checkmate
}