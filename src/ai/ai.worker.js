import { getLegalMoves, getLegalDrops, movePiece, dropPiece, isKingInCheck, isCheckmate, generateStateHash, getAttackedSquares, PLAYER_1, PLAYER_2, PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING, PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK } from '../game/engine';
import openingBook from './openingBook.json';

let transpositionTable = new Map();
let historyTable = Array(9).fill(0).map(() => Array(9).fill(0)); // For history heuristic
let killerMoves = Array(2).fill(null); // For killer moves (stores 2 best non-captures at current depth)
const LMR_DEPTH = 3; // Minimum depth for LMR to apply
const LMR_REDUCTION = 1; // How much to reduce the depth by

// This is the main entry point for the worker
self.onmessage = async (event) => {
  transpositionTable.clear();
  const { gameState, difficulty } = event.data;
  const bestMove = await getAiMove(gameState, difficulty);
  self.postMessage(bestMove);
};

const PIECE_VALUES = {
  [PAWN]: 100,
  [LANCE]: 300,
  [KNIGHT]: 320,
  [SILVER]: 450,
  [GOLD]: 500,
  [BISHOP]: 800,
  [ROOK]: 1000,
  [KING]: 20000,
  [PROMOTED_PAWN]: 500,
  [PROMOTED_LANCE]: 500,
  [PROMOTED_KNIGHT]: 500,
  [PROMOTED_SILVER]: 500,
  [PROMOTED_BISHOP]: 1200,
  [PROMOTED_ROOK]: 1300,
};

// Symmetrical Piece-Square Tables (PSTs)
const PAWN_PST = [
    [0,  0,  0,  0,  0,  0,  0,  0,  0],
    [5,  5,  5,  5,  5,  5,  5,  5,  5],
    [10, 10, 10, 10, 10, 10, 10, 10, 10],
    [15, 15, 15, 15, 15, 15, 15, 15, 15],
    [20, 20, 20, 20, 20, 20, 20, 20, 20],
    [25, 25, 25, 25, 25, 25, 25, 25, 25],
    [30, 30, 30, 30, 30, 30, 30, 30, 30],
    [35, 35, 35, 35, 35, 35, 35, 35, 35],
    [0,  0,  0,  0,  0,  0,  0,  0,  0]
];

const LANCE_PST = [
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0],
    [0, 0, 5, 10, 10, 10, 5, 0, 0]
];

const KNIGHT_PST = [
    [-10, -10, -10, -10, -10, -10, -10, -10, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 10, 15, 10, 5, 0, -10],
    [-10, 0, 10, 15, 20, 15, 10, 0, -10],
    [-10, 0, 5, 10, 15, 10, 5, 0, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, -10, -10, -10, -10, -10, -10, -10, -10]
];

const SILVER_PST = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
];

const GOLD_PST = [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
];

const BISHOP_PST = [
    [-10, -10, -10, -10, -10, -10, -10, -10, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 5, 10, 15, 15, 15, 10, 5, -10],
    [-10, 10, 15, 20, 20, 20, 15, 10, -10],
    [-10, 10, 15, 20, 20, 20, 15, 10, -10],
    [-10, 5, 10, 15, 15, 15, 10, 5, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, -10, -10, -10, -10, -10, -10, -10, -10]
];

const ROOK_PST = [
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0]
];

const PSTs = {
    [PAWN]: PAWN_PST,
    [LANCE]: LANCE_PST,
    [KNIGHT]: KNIGHT_PST,
    [SILVER]: SILVER_PST,
    [GOLD]: GOLD_PST,
    [BISHOP]: BISHOP_PST,
    [ROOK]: ROOK_PST,
    [KING]: Array(9).fill(Array(9).fill(0)), // King has no positional value
    [PROMOTED_PAWN]: GOLD_PST,
    [PROMOTED_LANCE]: GOLD_PST,
    [PROMOTED_KNIGHT]: GOLD_PST,
    [PROMOTED_SILVER]: GOLD_PST,
    [PROMOTED_BISHOP]: BISHOP_PST,
    [PROMOTED_ROOK]: ROOK_PST,
};

function getKingSafetyScore(board, player, kingPos) {
  if (!kingPos) return 0;

  const [kingR, kingC] = kingPos;
  let safetyScore = 0;

  // King Shield: Reward for having friendly pieces nearby
  const shieldOffsets = [
    [-1, -1], [-1, 0], [-1, 1],
    [0, -1],           [0, 1],
    [1, -1], [1, 0], [1, 1]
  ];

  for (const [dr, dc] of shieldOffsets) {
    const r = kingR + dr;
    const c = kingC + dc;

    if (r >= 0 && r < 9 && c >= 0 && c < 9) {
      const piece = board[r][c];
      if (piece && piece.player === player) {
        switch (piece.type) {
          case GOLD:
            safetyScore += 40;
            break;
          case SILVER:
            safetyScore += 30;
            break;
          case KNIGHT:
            safetyScore += 20;
            break;
          case LANCE:
            safetyScore += 15;
            break;
          case PAWN:
            safetyScore += 10;
            break;
          default:
            safetyScore += 5;
        }
      }
    }
  }

  // Penalize for nearby enemy pieces
  let enemyAttackers = 0;
  for (let r = Math.max(0, kingR - 2); r <= Math.min(8, kingR + 2); r++) {
    for (let c = Math.max(0, kingC - 2); c <= Math.min(8, kingC + 2); c++) {
      const piece = board[r][c];
      if (piece && piece.player !== player) {
        enemyAttackers++;
      }
    }
  }
  safetyScore -= enemyAttackers * 30;

  return safetyScore;
}

function getMobilityScore(board, player) {
  let mobility = 0;
  const legalMoves = getLegalMoves(board, player);
  mobility = legalMoves.length;

  // Bonus for controlling the center
  for (const move of legalMoves) {
    if (move.to[0] >= 3 && move.to[0] <= 5 && move.to[1] >= 3 && move.to[1] <= 5) {
      mobility += 0.1; // Small bonus for each move that lands in the center
    }
  }
  return mobility;
}

function evaluateBoard(gameState) {
    let score = 0;
    const { board, currentPlayer, capturedPieces, moveHistory } = gameState;
    const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

    // Tempo Bonus
    if (moveHistory.length > 0) {
        const lastMove = moveHistory[moveHistory.length - 1];
        if (lastMove.piece !== KING) {
            score += 10;
        }
    }

    // Find Kings
    let playerKingPos = null;
    let opponentKingPos = null;
    for (let r = 0; r < 9; r++) {
        for (let c = 0; c < 9; c++) {
            if (board[r][c] && board[r][c].type === KING) {
                if (board[r][c].player === currentPlayer) {
                    playerKingPos = [r, c];
                } else {
                    opponentKingPos = [r, c];
                }
            }
        }
    }

    // Material and Positional Score
    for (let r = 0; r < 9; r++) {
        for (let c = 0; c < 9; c++) {
            const piece = board[r][c];
            if (piece) {
                const pst = PSTs[piece.type];
                const pstValue = piece.player === PLAYER_1 ? pst[r][c] : pst[8 - r][8 - c];
                if (piece.player === currentPlayer) {
                    score += PIECE_VALUES[piece.type] + pstValue;
                } else {
                    score -= (PIECE_VALUES[piece.type] + pstValue);
                }
            }
        }
    }

    // Captured Pieces Score
    for (const piece of capturedPieces[currentPlayer]) {
        score += PIECE_VALUES[piece.type];
    }
    for (const piece of capturedPieces[opponent]) {
        score -= PIECE_VALUES[piece.type];
    }

    // King Safety Score
    score += getKingSafetyScore(board, currentPlayer, playerKingPos);
    score -= getKingSafetyScore(board, opponent, opponentKingPos);

    // Mobility Score
    const playerMobility = getMobilityScore(board, currentPlayer);
    const opponentMobility = getMobilityScore(board, opponent);
    score += (playerMobility - opponentMobility) * 10; // Adjust weight as needed

    // Threat Analysis
    const attackedByPlayer = getAttackedSquares(board, currentPlayer);
    const attackedByOpponent = getAttackedSquares(board, opponent);

    for (const square of attackedByPlayer) {
        const [r, c] = square.split(',').map(Number);
        const piece = board[r][c];
        if (piece && piece.player === opponent) {
            score += PIECE_VALUES[piece.type] / 5;
        }
    }

    for (const square of attackedByOpponent) {
        const [r, c] = square.split(',').map(Number);
        const piece = board[r][c];
        if (piece && piece.player === currentPlayer) {
            score -= PIECE_VALUES[piece.type] / 5;
        }
    }

    return score;
}

async function getAiMove(gameState, difficulty) {
  const { currentPlayer, moveHistory } = gameState;

  // Check opening book first
  const boardHash = generateStateHash(gameState);
  for (const opening of openingBook) {
    if (opening.moves[boardHash]) {
      console.log(`AI: Choosing move from opening: ${opening.name}`);
      const moves = opening.moves[boardHash];
      return moves[Math.floor(Math.random() * moves.length)];
    }
  }

  const maximizingPlayer = currentPlayer === PLAYER_2; // AI is always Player 2

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
        const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1);
        possibleMoves.push({ from: 'drop', to, type: capturedPiece.type, isCapture: false, isPromotion: false, isCheck });
      }
    });
  });

  // Sort moves for better alpha-beta pruning performance
  possibleMoves.sort((a, b) => scoreMove(b, gameState) - scoreMove(a, gameState));

  const startTime = Date.now();
  const timeLimit = difficulty === 'easy' ? 1000 : difficulty === 'medium' ? 3000 : 9000; // 1s for easy, 3s for medium, 9s for hard

  if (possibleMoves.length === 0) {
    console.log("No legal moves available for AI.");
    return null; // No legal moves available
  }

  let bestMove = possibleMoves[0];
  for (let depth = 1; depth <= 5; depth++) {
    const { move } = await pvs(gameState, depth, -Infinity, Infinity, maximizingPlayer, startTime, timeLimit, new Set());
    if (move) {
      bestMove = move;
    } else {
      // Time limit likely exceeded, return the best move from the previous depth
      break;
    }
  }
  return bestMove;
}


async function pvs(gameState, depth, alpha, beta, maximizingPlayer, startTime, timeLimit, history) {
  if (Date.now() - startTime > timeLimit) {
    return { score: 0, move: null };
  }

  const hash = generateStateHash(gameState);
  if (history.has(hash)) {
    return { score: 0, move: null };
  }

  if (depth === 0) {
    const score = await quiescenceSearch(gameState, alpha, beta, 0, startTime, timeLimit);
    return { score, move: null };
  }

  const possibleMoves = getPossibleMoves(gameState);
  possibleMoves.sort((a, b) => scoreMove(b, gameState) - scoreMove(a, gameState));

  let bestMove = null;
  let score = -Infinity;

  for (let i = 0; i < possibleMoves.length; i++) {
    const move = possibleMoves[i];
    let newGameState = { ...gameState, pastStates: [] };
    if (move.from === 'drop') {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to, move.promote);
    }

    const newHistory = new Set(history);
    newHistory.add(hash);

    let result;
    if (i === 0) {
      result = await pvs(newGameState, depth - 1, -beta, -alpha, !maximizingPlayer, startTime, timeLimit, newHistory);
      score = -result.score;
    } else {
      result = await pvs(newGameState, depth - 1, -alpha - 1, -alpha, !maximizingPlayer, startTime, timeLimit, newHistory);
      score = -result.score;
      if (alpha < score && score < beta) {
        result = await pvs(newGameState, depth - 1, -beta, -score, !maximizingPlayer, startTime, timeLimit, newHistory);
        score = -result.score;
      }
    }

    if (score > alpha) {
      alpha = score;
      bestMove = move;
    }

    if (alpha >= beta) {
      break; // Beta cut-off
    }
  }

  return { score: alpha, move: bestMove };
}

function scoreMove(move, gameState) {
  let score = 0;
  const { board, currentPlayer, moveHistory } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  // 1. Promotion Priority
  if (move.promote) {
    score += 800; // Strong bonus for promoting a piece
  }

  // 2. MVV-LVA for Captures
  if (move.isCapture) {
    const capturedPieceType = board[move.to[0]][move.to[1]]?.type;
    const attackingPieceType = move.from === 'drop' ? move.type : board[move.from[0]][move.from[1]]?.type;

    if (capturedPieceType && attackingPieceType) {
      score += PIECE_VALUES[capturedPieceType] * 10 - PIECE_VALUES[attackingPieceType];
    }
    score += 1000; // Base bonus for any capture
  }

  // 3. Recapture Priority
  if (moveHistory.length > 0) {
    const lastMove = moveHistory[moveHistory.length - 1];
    if (lastMove.to[0] === move.to[0] && lastMove.to[1] === move.to[1]) {
      score += 1200; // High bonus for immediate recapture
    }
  }

  // 4. Killer Moves Bonus (for quiet moves)
  if (!move.isCapture) {
    if (killerMoves[0] && move.from === killerMoves[0].from && move.to[0] === killerMoves[0].to[0] && move.to[1] === killerMoves[0].to[1]) {
      score += 900;
    } else if (killerMoves[1] && move.from === killerMoves[1].from && move.to[0] === killerMoves[1].to[0] && move.to[1] === killerMoves[1].to[1]) {
      score += 800;
    }
  }

  // 5. History Heuristic Bonus
  if (move.from !== 'drop' && historyTable[move.from[0]] && historyTable[move.from[0]][move.from[1]]) {
    score += historyTable[move.from[0]][move.from[1]];
  }

  // 6. Escape from Attack Priority
  if (move.from !== 'drop') {
    const attackedSquares = getAttackedSquares(board, opponent);
    const fromKey = `${move.from[0]},${move.from[1]}`;
    if (attackedSquares.has(fromKey)) {
        const attackingPiece = board[move.from[0]][move.from[1]];
        if (attackingPiece) {
            score += PIECE_VALUES[attackingPiece.type] / 4; // Bonus for escaping attack, proportional to piece value
        }
    }
  }

  // 7. Center Control Bonus
  if (move.to[0] >= 3 && move.to[0] <= 5 && move.to[1] >= 3 && move.to[1] <= 5) {
      score += 20; // Small bonus for moving towards the center
  }

  // 8. Check Priority
  if (move.isCheck) {
    score += 50;
  }

  return score;
}

function getPossibleMoves(gameState) {
  const { board, currentPlayer, capturedPieces } = gameState;
  const possibleMoves = [];

  // Collect all possible moves for pieces on the board
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, board);
        moves.forEach(to => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const isCapture = board[to[0]][to[1]] && board[to[0]][to[1]].player !== currentPlayer;
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
  capturedPieces[currentPlayer].forEach(capturedPiece => {
    const legalDrops = getLegalDrops(gameState, capturedPiece.type);
    legalDrops.forEach(to => {
      const simulatedGameState = dropPiece(gameState, capturedPiece.type, to);
      if (simulatedGameState !== gameState && !isKingInCheck(simulatedGameState.board, currentPlayer)) {
        const isCheck = isKingInCheck(simulatedGameState.board, simulatedGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1);
        possibleMoves.push({ from: 'drop', to, type: capturedPiece.type, isCapture: false, isPromotion: false, isCheck });
      }
    });
  });

  return possibleMoves;
}



async function quiescenceSearch(gameState, alpha, beta, depth, startTime, timeLimit) {
  if (Date.now() - startTime > timeLimit) {
    console.log(`Quiescence search time limit exceeded at depth ${depth}`);
    return null; // Indicate that the search was cut short
  }
  let standPat = evaluateBoard(gameState);
  
  if (depth >= 5) { // Limit quiescence search depth
    return standPat;
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
    if (Date.now() - startTime > timeLimit) {
      return 0; // Abort if time limit exceeded during move iteration
    }
    await new Promise(resolve => setTimeout(resolve, 0)); // Yield control
    let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
    if (move.from === 'drop') {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to);
    }

    const score = -(await quiescenceSearch(newGameState, -beta, -alpha, depth + 1, startTime, timeLimit)); // Negamax: negate score from recursive call

    if (score >= beta) {
      return beta;
    }
    if (score > alpha) {
      alpha = score;
    }
  }
  return alpha;
}