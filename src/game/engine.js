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

const formatShogiCoords = (r, c) => `[${r + 1}, ${9 - c}]`;

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
    if (r < 0 || r >= ROWS || c < 0 || c >= COLS) {
      return false; // Off-board
    }
    const targetPiece = board[r][c];
    if (targetPiece && targetPiece.player === player) {
      return false; // Cannot capture own piece
    }
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
          break; // Stop after hitting any piece (friendly or enemy)
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
      // Move forward
      addMove(row + player_mult, col);
      // In Shogi, pawns capture by moving one square directly forward onto an opponent's piece.
      // The `addMove` function already handles this by checking if the target square is occupied by an opponent's piece.
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
export function movePiece(gameState, from, to, promoteOverride = null) {
  const { board, currentPlayer } = gameState;
  const [fromRow, fromCol] = from;
  const [toRow, toCol] = to;

  const piece = board[fromRow][fromCol];
  if (!piece) return gameState;

  if (promoteOverride !== null) {
    // AI is making the decision, directly complete the move with the AI's promotion choice
    let pieceToMove = { ...piece };
    if (promoteOverride && !pieceToMove.type.startsWith('+')) {
      pieceToMove.type = `+${pieceToMove.type}`;
    }
    const simulatedGameState = completeMove(gameState, from, to, promoteOverride, pieceToMove);
    return simulatedGameState;
  }

  // --- Promotion Logic (for human player) ---
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

  let pieceToMove = { ...piece };

  if (canPromote && !isPromotionMandatory) {
    // If promotion is optional, return a state that asks the UI for a choice.
    return {
      ...gameState,
      promotionPending: { from, to, piece },
    };
  } else if (isPromotionMandatory && !pieceToMove.type.startsWith('+')) {
    // If promotion is mandatory, promote the piece
    pieceToMove.type = `+${pieceToMove.type}`;
  }

  // Proceed with the move (either promoted or not, depending on the above logic)
  const simulatedGameState = completeMove(gameState, from, to, isPromotionMandatory, pieceToMove);
  return simulatedGameState;
}

/**
 * Completes a move, including capture and promotion.
 * @param {object} gameState The current game state.
 * @param {[number, number]} from The starting [row, col].
 * @param {[number, number]} to The destination [row, col].
 * @param {boolean} promote Whether to promote the piece.
 * @returns {object} The final new game state.
 */
export function completeMove(gameState, from, to, promote, movedPiece = null) {
    const { board, currentPlayer, capturedPieces, moveHistory, pastStates } = gameState;
    const nextPlayer = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
    // Deep copy the board
    const newBoard = board.map(row => row.map(cell => cell ? { ...cell } : null));

    const [fromRow, fromCol] = from;
    const [toRow, toCol] = to;

    // Get the piece that is actually moving from the original board
    const pieceToMove = board[fromRow][fromCol];
    if (!pieceToMove) return gameState; // Should not happen if movePiece is called correctly

    let piece = { ...pieceToMove }; // Create a new object for the piece that will be on the board
    if (promote && !piece.type.startsWith('+')) {
        piece.type = `+${piece.type}`;
    }

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

    newBoard[toRow][toCol] = piece;
    newBoard[fromRow][fromCol] = null;

    const isCheckAfterMove = isKingInCheck(newBoard, nextPlayer);
    let capturedValue = null;
    if (capturedPiece) {
        capturedValue = capturedPiece.type;
        if (isCheckAfterMove) {
            capturedValue += ' / check';
        }
    } else if (isCheckAfterMove) {
        capturedValue = 'check';
    }

    const newMoveHistory = [...moveHistory, { from, to, piece: piece.type, promote, player: currentPlayer, captured: capturedValue, timestamp: new Date().toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit', second: '2-digit', hour12: false }), promotionDeclined: !promote && gameState.promotionPending }];

    const updatedGameState = {
        ...gameState,
        board: newBoard,
        capturedPieces: newCapturedPieces,
        currentPlayer: nextPlayer,
        moveHistory: newMoveHistory,
        promotionPending: null,
        pastStates: [...pastStates, gameState], // Save current state before move
        isCheck: isKingInCheck(newBoard, nextPlayer),
        kingPositions: {
          ...gameState.kingPositions,
          [currentPlayer]: (piece.type === KING) ? [toRow, toCol] : gameState.kingPositions[currentPlayer]
        }
    };


  
    return updatedGameState;
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
      const nextPlayer = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
      return { ...gameState, currentPlayer: nextPlayer };
  }


  // 4. Update the board and captured pieces
  

  // Simulate the drop on a temporary board to check for self-check
  const tempBoard = board.map(row => [...row]);
  tempBoard[toRow][toCol] = { type: pieceType, player: currentPlayer };

  // Check if this simulated drop puts the *current player's* king in check
  if (isKingInCheck(tempBoard, currentPlayer)) {
    const nextPlayer = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
    return { ...gameState, currentPlayer: nextPlayer }; // Illegal drop: current player dropped into check
  }

  // If the drop is legal, proceed with updating the actual game state
  const newBoard = tempBoard; // Use the board with the piece dropped

  const newCapturedPieces = { ...capturedPieces };
  newCapturedPieces[currentPlayer] = [...capturedPieces[currentPlayer]];
  newCapturedPieces[currentPlayer].splice(capturedPieceIndex, 1);

  const nextPlayer = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  const isCheckAfterDrop = isKingInCheck(newBoard, nextPlayer); // Check if opponent is in check

  const newMoveHistory = [...gameState.moveHistory, { from: 'drop', to, piece: pieceType, player: currentPlayer, captured: isCheckAfterDrop ? 'check' : null, timestamp: new Date().toLocaleTimeString('en-US', { hour12: false }) }];

  const finalGameState = {
    ...gameState,
    board: newBoard,
    capturedPieces: newCapturedPieces,
    currentPlayer: nextPlayer, // Switch player here
    moveHistory: newMoveHistory,
    pastStates: [...pastStates, gameState], // Save current state before drop
    isCheck: isCheckAfterDrop, // Set isCheck for the next player
    kingPositions: {
      ...gameState.kingPositions,
      [currentPlayer]: (pieceType === KING) ? [toRow, toCol] : gameState.kingPositions[currentPlayer]
    }
  };

  

  return finalGameState;
}

/**
 * Gets the position of the piece checking the king.
 * @param {Array<Array<object>>} board The board state.
 * @param {string} player The player to check.
 * @returns {[number, number] | null} The [row, col] of the checking piece, or null if not in check.
 */
export function getCheckingPiece(board, player) {
  const opponent = player === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  let kingPosition = null;

  // Find the king's position directly from the board
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

  if (!kingPosition) {
    return null; // Should not happen in a real game
  }

  // Check if any opponent piece can attack the king
  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const piece = board[r][c];
      if (piece && piece.player === opponent) {
        const moves = getLegalMoves(piece, r, c, board);
        for (const move of moves) {
          if (move[0] === kingPosition[0] && move[1] === kingPosition[1]) {
            return [r, c]; // Return the position of the checking piece
          }
        }
      }
    }
  }

  return null; // King is not in check
}


/**
 * Checks if a player's king is in check.
 * @param {Array<Array<object>>} board The board state.
 * @param {string} player The player to check.
 * @returns {boolean} True if the king is in check, false otherwise.
 */
export function isKingInCheck(board, player) {
  return getCheckingPiece(board, player) !== null;
}

/**
 * Gets all legal drop squares for a captured piece.
 * @param {object} gameState The current game state.
 * @param {string} pieceType The type of piece to drop.
 * @returns {Array<[number, number]>} An array of [row, col] pairs representing legal drop squares.
 */
export function getLegalDrops(gameState, pieceType) {
  const { board, currentPlayer } = gameState;
  const legalDrops = [];

  // Check if the player actually has the piece to drop
  const hasPieceToDrop = gameState.capturedPieces[currentPlayer].some(p => p.type === pieceType);
  if (!hasPieceToDrop) {
    return legalDrops;
  }

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      // 1. Must be an empty square
      if (board[r][c] !== null) {
        continue;
      }

      // 2. Nifu check (two unpromoted pawns in the same file)
      if (pieceType === PAWN) {
        let nifu = false;
        for (let row = 0; row < ROWS; row++) {
          const piece = board[row][c];
          if (piece && piece.type === PAWN && piece.player === currentPlayer) {
            nifu = true;
            break;
          }
        }
        if (nifu) {
          continue;
        }
      }

      // 3. Cannot drop a piece where it has no legal moves (e.g., Pawn on last rank, Knight on last two ranks, Lance on last rank)
      const player_mult = currentPlayer === PLAYER_1 ? -1 : 1;
      if ((pieceType === PAWN || pieceType === LANCE) && (r + player_mult < 0 || r + player_mult >= ROWS)) {
        continue;
      }
      if (pieceType === KNIGHT && (r + (player_mult * 2) < 0 || r + (player_mult * 2) >= ROWS)) {
        continue;
      }

      // 4. Cannot drop a pawn to give an immediate checkmate (Uchifu-zume)
      if (pieceType === PAWN) {
        const simulatedBoard = board.map(row => [...row]);
        simulatedBoard[r][c] = { type: pieceType, player: currentPlayer };
        const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

        // Temporarily remove the pawn to check if the king is still in check without it
        const originalPieceAtDrop = simulatedBoard[r][c];
        simulatedBoard[r][c] = null; // Remove the dropped pawn temporarily
        const kingStillInCheckWithoutPawn = isKingInCheck(simulatedBoard, opponent);
        simulatedBoard[r][c] = originalPieceAtDrop; // Restore the dropped pawn

        if (isKingInCheck(simulatedBoard, opponent) && isCheckmate({ ...gameState, board: simulatedBoard, currentPlayer: opponent }) && !kingStillInCheckWithoutPawn) {
          continue; // Uchifu-zume rule violation
        }
      }

      legalDrops.push([r, c]);
    }
  }
  return legalDrops;
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
                    const tempState = completeMove(gameState, [r, c], move, false); // Assume no promotion for checkmate simulation
                    
                    if (!isKingInCheck(tempState.board, currentPlayer)) {
                        
                        return false; // Found a move to escape check
                    }
                }
            }
        }
    }

    // Check if dropping any piece can get the king out of check
    
        // Use getLegalDrops to find valid drop squares
        const possibleDropSquares = getLegalDrops(gameState, captured.type);
        for (const dropSquare of possibleDropSquares) {
            const tempBoard = board.map(row => [...row]);
            tempBoard[dropSquare[0]][dropSquare[1]] = { type: captured.type, player: currentPlayer };
            
            if (!isKingInCheck(tempBoard, currentPlayer)) {
                
                return false; // Found a drop to escape check
            }
        }
    }

    
    

/**
 * Gets all squares attacked by a player.
 * @param {Array<Array<object>>} board The board state.
 * @param {string} player The player who is attacking.
 * @returns {Set<string>} A set of attacked squares in "row,col" format.
 */
export function getAttackedSquares(board, player) {
  const attackedSquares = new Set();

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const piece = board[r][c];
      if (piece && piece.player === player) {
        const moves = getLegalMoves(piece, r, c, board);
        for (const move of moves) {
          attackedSquares.add(`${move[0]},${move[1]}`);
        }
      }
    }
  }

  return attackedSquares;
}

/**
 * Generates a unique hash for the current game state.
 * @param {object} gameState The current game state.
 * @returns {string} A unique string representing the game state.
 */
export function generateStateHash(gameState) {
  const { board, currentPlayer, capturedPieces, moveHistory } = gameState;

  let fen = '';

  // Board state
  for (let i = 0; i < 9; i++) {
    let empty = 0;
    for (let j = 0; j < 9; j++) {
      const piece = board[i][j];
      if (piece) {
        if (empty > 0) {
          fen += empty;
          empty = 0;
        }
        let pieceChar = piece.type.toLowerCase();
        if (piece.player === PLAYER_1) {
          pieceChar = pieceChar.toUpperCase();
        }
        if (piece.type.startsWith('+')) {
            pieceChar = '+' + pieceChar[1];
        }
        fen += pieceChar;
      } else {
        empty++;
      }
    }
    if (empty > 0) {
      fen += empty;
    }
    if (i < 8) {
      fen += '/';
    }
  }

  // Active player
  fen += currentPlayer === PLAYER_1 ? ' w ' : ' b ';

  // Captured pieces
  let capturedString = '-';
  const p1Captured = capturedPieces[PLAYER_1].map(p => p.type).sort().join('');
  const p2Captured = capturedPieces[PLAYER_2].map(p => p.type).sort().join('');
  if (p1Captured.length > 0) {
      capturedString = p1Captured.toUpperCase();
  }
  if (p2Captured.length > 0) {
      capturedString += p2Captured.toLowerCase();
  }
  fen += capturedString + ' ';

  return fen;
}
