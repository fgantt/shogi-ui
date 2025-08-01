import { getLegalMoves, movePiece, dropPiece, isKingInCheck, isCheckmate, PLAYER_1, PLAYER_2, PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING, PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK } from '../game/engine';

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
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        if (piece.player === currentPlayer) {
          score += PIECE_VALUES[piece.type];
        } else {
          score -= PIECE_VALUES[piece.type];
        }
      }
    }
  }

  // Captured pieces advantage
  capturedPieces[currentPlayer].forEach(piece => {
    score += PIECE_VALUES[piece.type];
  });
  capturedPieces[opponent].forEach(piece => {
    score -= PIECE_VALUES[piece.type];
  });

  // Check/Checkmate bonus/penalty
  if (isKingInCheck(board, opponent)) {
    score += 50; // Bonus for checking opponent
  }
  if (isKingInCheck(board, currentPlayer)) {
    score -= 50; // Penalty for being in check
  }

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
  console.log(`minimax: depth ${depth}, maxDepth ${maxDepth}, maximizingPlayer ${maximizingPlayer}, alpha ${alpha}, beta ${beta}`);
  if (depth === maxDepth || gameState.isCheckmate) {
    const score = evaluateBoard(gameState);
    console.log(`minimax: Terminal node reached. Score: ${score}`);
    return { score, move: null }; // Return null for move at terminal nodes
  }

  const { board, currentPlayer, capturedPieces } = gameState;
  const possibleMoves = [];

  // Collect all possible moves for pieces on the board
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, board);
        moves.forEach(to => {
          possibleMoves.push({ from: [r, c], to, type: 'move' });
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
          if (tempState !== gameState) { // If drop was legal
            possibleMoves.push({ from: 'drop', to: [r, c], type: capturedPiece.type.startsWith('+') ? capturedPiece.type.substring(1) : capturedPiece.type });
          }
        }
      }
    }
  });

  let bestScore = maximizingPlayer ? -Infinity : Infinity;
  let bestMove = null;

  for (const move of possibleMoves) {
    await new Promise(resolve => setTimeout(resolve, 0)); // Yield control to the browser
    let newGameState;
    if (move.from === 'drop') {
      newGameState = dropPiece(gameState, move.type, move.to);
    } else {
      newGameState = movePiece(gameState, move.from, move.to);
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
  console.log(`minimax: Returning bestScore ${bestScore} for depth ${depth}`);
  return { score: bestScore, move: bestMove };
}

export async function getAiMove(gameState, difficulty) {
  console.log(`getAiMove: Starting AI move calculation for difficulty ${difficulty}`);
  const { currentPlayer } = gameState;
  const maximizingPlayer = currentPlayer === PLAYER_2; // AI is always Player 2

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
          if (!isKingInCheck(simulatedGameState.board, currentPlayer)) { // Only add if the move doesn't put own king in check
            possibleMoves.push({ from: [r, c], to, type: 'move' });
          }
        });
      }
    }
  }

  // Collect all possible drops for captured pieces
  gameState.capturedPieces[currentPlayer].forEach(capturedPiece => {
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (!gameState.board[r][c]) { // Only drop on empty squares
          const simulatedGameState = dropPiece(gameState, capturedPiece.type, [r, c]);
          if (!isKingInCheck(simulatedGameState.board, currentPlayer)) { // Only add if the drop doesn't put own king in check
            possibleMoves.push({ from: 'drop', to: [r, c], type: capturedPiece.type });
          }
        }
      }
    }
  });

  if (possibleMoves.length === 0) {
    console.log('getAiMove: No legal moves available.');
    return null; // No legal moves available
  }

  switch (difficulty) {
    case 'easy':
      // Easy AI logic: random move
      const randomIndex = Math.floor(Math.random() * possibleMoves.length);
      console.log(`getAiMove: Easy mode returning random move: ${JSON.stringify(possibleMoves[randomIndex])}`);
      return possibleMoves[randomIndex];
    case 'medium':
      bestScore = -Infinity;
      bestMove = null;

      for (const move of possibleMoves) {
        let newGameState;
        if (move.from === 'drop') {
          newGameState = dropPiece(gameState, move.type, move.to);
        } else {
          newGameState = movePiece(gameState, move.from, move.to);
        }

        const { score } = await minimax(newGameState, 0, 2, !maximizingPlayer); // Shallow search depth of 2

        if (score > bestScore) {
          bestScore = score;
          bestMove = move;
        }
      }
      console.log(`getAiMove: Medium mode returning best move: ${JSON.stringify(bestMove)} with score ${bestScore}`);
      return bestMove;
    case 'hard':
      bestScore = -Infinity;
      bestMove = null;

      for (const move of possibleMoves) {
        await new Promise(resolve => setTimeout(resolve, 0)); // Yield control to the browser
        let newGameState;
        if (move.from === 'drop') {
          newGameState = dropPiece(gameState, move.type, move.to);
        } else {
          newGameState = movePiece(gameState, move.from, move.to);
        }

        const { score } = await minimax(newGameState, 0, 2, !maximizingPlayer); // Deeper search depth of 2 (reduced for performance)

        if (score > bestScore) {
          bestScore = score;
          bestMove = move;
        }
      }
      console.log(`getAiMove: Hard mode returning best move: ${JSON.stringify(bestMove)} with score ${bestScore}`);
      return bestMove;
    default:
      console.log('getAiMove: Unknown difficulty, returning null.');
      return null;
  }
}