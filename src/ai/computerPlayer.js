import { getLegalMoves, getLegalDrops, movePiece, dropPiece, isKingInCheck, isCheckmate, PLAYER_1, PLAYER_2, PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING, PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK } from '../game/engine';

// Piece values for evaluation (can be adjusted)
const PIECE_VALUES = {
  [PAWN]: 1,
  [LANCE]: 3,
  [KNIGHT]: 3,
  [SILVER]: 5,
  [GOLD]: 6,
  [BISHOP]: 8,
  [ROOK]: 10,
  [KING]: 0, // King's value is irrelevant for material count
  [PROMOTED_PAWN]: 6,
  [PROMOTED_LANCE]: 6,
  [PROMOTED_KNIGHT]: 6,
  [PROMOTED_SILVER]: 6,
  [PROMOTED_BISHOP]: 12,
  [PROMOTED_ROOK]: 12,
};

// Helper to score moves for move ordering
function scoreMove(move, gameState) {
  let score = 0;
  const { board, currentPlayer } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  if (move.isCapture) {
    score += 10; // High priority for captures
    // Add bonus for capturing higher value pieces
    const capturedPieceType = board[move.to[0]][move.to[1]]?.type;
    if (capturedPieceType) {
      score += PIECE_VALUES[capturedPieceType];
    }
  }
  if (move.isCheck) {
    score += 8; // Priority for checks
  }
  return score;
}

/**
 * Evaluates the board state from the perspective of the current player.
 * A positive score favors the current player, a negative score favors the opponent.
 * @param {object} gameState The current game state.
 * @returns {number} The evaluation score.
 */
function evaluateBoard(gameState) {
  let score = 0;
  const { board, currentPlayer, capturedPieces } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  // Material advantage
  let materialScore = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        if (piece.player === currentPlayer) {
          materialScore += PIECE_VALUES[piece.type];
        } else {
          materialScore -= PIECE_VALUES[piece.type];
        }
      }
    }
  }
  score += materialScore;

  // Captured pieces advantage
  let capturedScore = 0;
  capturedPieces[currentPlayer].forEach(piece => {
    const type = piece.type.startsWith('+') ? piece.type.substring(1) : piece.type;
    capturedScore += PIECE_VALUES[type];
  });
  capturedPieces[opponent].forEach(piece => {
    const type = piece.type.startsWith('+') ? piece.type.substring(1) : piece.type;
    capturedScore -= PIECE_VALUES[type];
  });
  score += capturedScore;

  // Check/Checkmate bonus/penalty
  let checkScore = 0;
  if (isKingInCheck(board, opponent)) {
    checkScore += 50; // Bonus for checking opponent
  }
  if (isKingInCheck(board, currentPlayer)) {
    checkScore -= 50; // Penalty for being in check
  }
  score += checkScore;

  // Checkmate bonus/penalty
  if (isCheckmate(gameState)) {
    if (currentPlayer === PLAYER_2) { // AI is Player 2
      score = Infinity; // AI wins
    } else {
      score = -Infinity; // AI loses
    }
  }

  // TODO: Add more sophisticated evaluation (e.g., piece mobility, king safety, pawn structure)

  return score;
}

/**
 * Performs a quiescence search to evaluate noisy positions.
 * @param {object} gameState The current game state.
 * @param {number} alpha The alpha value for alpha-beta pruning.
 * @param {number} beta The beta value for alpha-beta pruning.
 * @returns {number} The evaluation score after quiescence search.
 */
const MAX_QUIESCENCE_DEPTH = 3; // Limit quiescence search depth to prevent infinite loops

async function quiescenceSearch(gameState, alpha, beta, depth) {
  let standPat = evaluateBoard(gameState);
  
  if (depth >= MAX_QUIESCENCE_DEPTH) {
    const score = evaluateBoard(gameState);
    return score;
  }
  if (standPat >= beta) {
    return beta;
  }
  if (standPat > alpha) {
    alpha = standPat;
  }

  const { board, currentPlayer, capturedPieces } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  const possibleNoisyMoves = [];

  // Collect noisy moves (captures, checks, promotions)
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, board);
        moves.forEach(to => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const targetPieceAfterMove = simulatedGameState.board[to[0]][to[1]];
          const isCapture = targetPieceAfterMove && targetPieceAfterMove.player !== currentPlayer;
          const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer);

          if (isCapture || isCheck) {
            possibleNoisyMoves.push({ from: [r, c], to, type: 'move', isCapture, isCheck });
          }
        });
      }
    }
  }

  // Collect noisy drops (checks)
  capturedPieces[currentPlayer].forEach(capturedPiece => {
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (!board[r][c]) { // Only drop on empty squares
          const simulatedGameState = dropPiece(gameState, capturedPiece.type, [r, c]);
          const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer);
          if (isCheck) {
            possibleNoisyMoves.push({ from: 'drop', to: [r, c], type: capturedPiece.type, isCapture: false, isCheck });
          }
        }
      }
    }
  });

  // Sort noisy moves for better pruning
  possibleNoisyMoves.sort((a, b) => scoreMove(b, gameState) - scoreMove(a, gameState));

  if (possibleNoisyMoves.length === 0) {
    return alpha;
  }

  for (const move of possibleNoisyMoves) {
    await new Promise(resolve => setTimeout(resolve, 0)); // Yield control
    let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
    if (move.from === 'drop') {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to);
    }

    const score = -(await quiescenceSearch(newGameState, -beta, -alpha, depth + 1)); // Negamax: negate score from recursive call

    if (score >= beta) {
      return beta;
    }
    if (score > alpha) {
      alpha = score;
    }
  }
  return alpha;
}

/**
 * Minimax algorithm to find the best move.
 * @param {object} gameState The current game state.
 * @param {number} depth The current search depth.
 * @param {number} maxDepth The maximum search depth.
 * @param {boolean} maximizingPlayer True if the current player is maximizing, false otherwise.
 * @param {number} alpha The alpha value for alpha-beta pruning.
 * @param {number} beta The beta value for alpha-beta pruning.
 * @returns {{score: number, move: object}} The best score and corresponding move.
 */
async function minimax(gameState, depth, maxDepth, maximizingPlayer, alpha = -Infinity, beta = Infinity) {
  if (depth === maxDepth || gameState.isCheckmate) {
    const score = await quiescenceSearch(gameState, alpha, beta, 0); // Call quiescence search at max depth
    return { score, move: null }; // Return null for move at terminal nodes
  }

  const { board, currentPlayer, capturedPieces } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  const possibleMoves = [];

  // Collect all possible moves for pieces on the board
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, board);
        moves.forEach(to => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const isCapture = gameState.board[to[0]][to[1]] && gameState.board[to[0]][to[1]].player !== currentPlayer;
          const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer);
          if (isCapture || isCheck) {
            possibleMoves.push({ from: [r, c], to, type: 'move', isCapture, isCheck });
          }
        });
      }
    }
  }

  // Collect all possible drops for captured pieces
  capturedPieces[currentPlayer].forEach(capturedPiece => {
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (!board[r][c]) { // Only drop on empty squares
          // Simulate drop to check legality (e.g., Nifu, no legal moves)
          const tempState = dropPiece(gameState, capturedPiece.type, [r, c]);
          const isCheck = isKingInCheck(tempState.board, tempState.currentPlayer);
          if (tempState !== gameState && isCheck) { // If drop was legal and results in a check
            possibleMoves.push({ from: 'drop', to: [r, c], type: capturedPiece.type.startsWith('+') ? capturedPiece.type.substring(1) : capturedPiece.type, isCapture: false, isCheck });
          }
        }
      }
    }
  });

  // Sort moves for better alpha-beta pruning performance
  possibleMoves.sort((a, b) => scoreMove(b, gameState) - scoreMove(a, gameState));

  let bestScore = maximizingPlayer ? -Infinity : Infinity;
  let bestMove = null;

  for (const move of possibleMoves) {
    await new Promise(resolve => setTimeout(resolve, 0)); // Yield control to the browser
    let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
    if (move.from === 'drop') {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to);
    }

    const { score } = await minimax(newGameState, depth + 1, maxDepth, !maximizingPlayer, alpha, beta);

    if (maximizingPlayer) {
      if (score > bestScore) {
        bestScore = score;
        bestMove = move;
      }
      alpha = Math.max(alpha, bestScore);
      if (beta <= alpha) {
        break; // Beta cut-off
      }
    } else {
      if (score < bestScore) {
        bestScore = score;
        bestMove = move;
      }
      beta = Math.min(beta, bestScore);
      if (beta <= alpha) {
        break; // Alpha cut-off
      }
    }
  }
  return { score: bestScore, move: bestMove };
}

export async function getAiMove(gameState, difficulty) {
  const { currentPlayer } = gameState;
  const maximizingPlayer = currentPlayer === PLAYER_2; // AI is always Player 2
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  let bestMove = null;
  let bestScore = -Infinity;

  const possibleMoves = [];

  // Collect all possible moves for pieces on the board
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = gameState.board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, gameState.board);
        moves.forEach(to => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const isCapture = gameState.board[to[0]][to[1]] && gameState.board[to[0]][to[1]].player !== currentPlayer;
          const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer);
          const promotionZoneStart = currentPlayer === PLAYER_1 ? 2 : 6;
          const inPromotionZone = (currentPlayer === PLAYER_1 && to[0] <= promotionZoneStart) || (currentPlayer === PLAYER_2 && to[0] >= promotionZoneStart);
          const wasInPromotionZone = (currentPlayer === PLAYER_1 && r <= promotionZoneStart) || (currentPlayer === PLAYER_2 && r >= promotionZoneStart);
          const promotablePieces = [PAWN, LANCE, KNIGHT, SILVER, BISHOP, ROOK];
          const canPromote = promotablePieces.includes(piece.type) && (inPromotionZone || wasInPromotionZone);
          const lastRank = currentPlayer === PLAYER_1 ? 0 : 8;
          const secondLastRank = currentPlayer === PLAYER_1 ? 1 : 7;
          let isPromotionMandatory = false;
          if ((piece.type === PAWN || piece.type === LANCE) && to[0] === lastRank) {
              isPromotionMandatory = true;
          }
          if (piece.type === KNIGHT && (to[0] === lastRank || to[0] === secondLastRank)) {
              isPromotionMandatory = true;
          }

          let promote = false;
          if (canPromote && !isPromotionMandatory) {
            promote = true; // AI always promotes if optional
          } else if (isPromotionMandatory) {
            promote = true; // Mandatory promotion
          }

          if (!isKingInCheck(simulatedGameState.board, currentPlayer)) { // Only add if the move doesn't put own king in check
            possibleMoves.push({ from: [r, c], to, type: 'move', isCapture, isCheck, promote });
          }
        });
      }
    }
  }

  // Collect all possible drops for captured pieces
  gameState.capturedPieces[currentPlayer].forEach(capturedPiece => {
    const legalDrops = getLegalDrops(gameState, capturedPiece.type);
    legalDrops.forEach(to => {
      const simulatedGameState = dropPiece(gameState, capturedPiece.type, to);
      if (simulatedGameState !== gameState && !isKingInCheck(simulatedGameState.board, currentPlayer)) {
        const isCheck = isKingInCheck(simulatedGameState.board, opponent);
        possibleMoves.push({ from: 'drop', to, type: capturedPiece.type, isCapture: false, isPromotion: false, isCheck });
      }
    });
  });

  // Sort moves for better alpha-beta pruning performance
  possibleMoves.sort((a, b) => scoreMove(b, gameState) - scoreMove(a, gameState));

  if (possibleMoves.length === 0) {
    return null; // No legal moves available
  }

  switch (difficulty) {
    case 'easy':
      // Easy AI logic: random move
      const randomIndex = Math.floor(Math.random() * possibleMoves.length);
      return possibleMoves[randomIndex];
    case 'medium':
      bestScore = -Infinity;
      bestMove = null;

      for (const move of possibleMoves) {
        let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
        if (move.from === 'drop') {
          newGameState = dropPiece(newGameState, move.type, move.to);
        } else {
          newGameState = movePiece(newGameState, move.from, move.to, move.promote); // Pass promote flag
        }

        const { score } = await minimax(newGameState, 0, 1, !maximizingPlayer); // Shallow search depth of 1

        if (score > bestScore) {
          bestScore = score;
          bestMove = move;
        }
      }
      return bestMove;
    case 'hard':
      bestScore = -Infinity;
      bestMove = null;

      for (const move of possibleMoves) {
        await new Promise(resolve => setTimeout(resolve, 0)); // Yield control to the browser
        let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
        if (move.from === 'drop') {
          newGameState = dropPiece(newGameState, move.type, move.to);
        } else {
          newGameState = movePiece(newGameState, move.from, move.to, move.promote); // Pass promote flag
        }

        const { score } = await minimax(newGameState, 0, 1, !maximizingPlayer); // Deeper search depth of 1 (reduced for performance)

        if (score > bestScore) {
          bestScore = score;
          bestMove = move;
        }
      }
      return bestMove;
    default:
      return null;
  }
}