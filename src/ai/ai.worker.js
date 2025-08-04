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

function evaluateBoard(gameState) {
    let score = 0;
    const { board, currentPlayer, capturedPieces, pastStates, moveHistory } = gameState;
    const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

    // Tempo Bonus
    if (moveHistory.length > 0) {
        const lastMove = moveHistory[moveHistory.length - 1];
        if (lastMove.piece !== KING) {
            score += 10;
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

    // King Safety and Threat Analysis
    let kingPos = null;
    for (let r = 0; r < 9; r++) {
        for (let c = 0; c < 9; c++) {
            if (board[r][c] && board[r][c].type === KING && board[r][c].player === currentPlayer) {
                kingPos = [r, c];
                break;
            }
        }
        if (kingPos) break;
    }

    if (kingPos) {
        const [kingR, kingC] = kingPos;
        const safetyZone = [];
        for (let r = Math.max(0, kingR - 1); r <= Math.min(8, kingR + 1); r++) {
            for (let c = Math.max(0, kingC - 1); c <= Math.min(8, kingC + 1); c++) {
                safetyZone.push([r, c]);
            }
        }

        let friendlyDefenders = 0;
        let enemyAttackers = 0;
        for (const [r, c] of safetyZone) {
            const piece = board[r][c];
            if (piece) {
                if (piece.player === currentPlayer) {
                    friendlyDefenders++;
                } else {
                    enemyAttackers++;
                }
            }
        }
        score += (friendlyDefenders - enemyAttackers) * 50;
    }

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
  if (openingBook[boardHash]) {
    const moves = openingBook[boardHash];
    return moves[Math.floor(Math.random() * moves.length)];
  }

  // Opening Randomness
  if (moveHistory.length < 5) {
    const possibleMoves = [];
    for (let r = 0; r < 9; r++) {
        for (let c = 0; c < 9; c++) {
            const piece = gameState.board[r][c];
            if (piece && piece.player === currentPlayer) {
                const moves = getLegalMoves(piece, r, c, gameState.board);
                moves.forEach(to => {
                    const simulatedGameState = movePiece(gameState, [r, c], to);
                    if (!isKingInCheck(simulatedGameState.board, currentPlayer)) {
                        possibleMoves.push({ from: [r, c], to, type: 'move' });
                    }
                });
            }
        }
    }
    const moveScores = await Promise.all(possibleMoves.map(async move => {
        let newGameState = { ...gameState, pastStates: [] };
        newGameState = movePiece(newGameState, move.from, move.to, false);
        const score = evaluateBoard(newGameState);
        return { move, score };
    }));
    moveScores.sort((a, b) => b.score - a.score);
    const topMoves = moveScores.slice(0, 3).map(ms => ms.move);
    return topMoves[Math.floor(Math.random() * topMoves.length)];
  }

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
  const timeLimit = difficulty === 'medium' ? 1000 : 3000; // 1 second for medium, 3 for hard

  if (possibleMoves.length === 0) {
    console.log("No legal moves available for AI.");
    return null; // No legal moves available
  }

  console.log(`AI thinking for difficulty: ${difficulty}, Time limit: ${timeLimit}ms`);
  console.log(`Initial possible moves count: ${possibleMoves.length}`);

  switch (difficulty) {
    case 'easy':
      // Easy AI logic: random move
      const randomIndex = Math.floor(Math.random() * possibleMoves.length);
      return possibleMoves[randomIndex];
    case 'medium':
    case 'hard':
      let currentBestMove = possibleMoves[0]; // Initialize with the first possible move
      let currentBestScore = -Infinity;

      for (let depth = 1; depth <= 5; depth++) { // Iterate up to a maximum depth
        let iterationBestMove = null;
        let iterationBestScore = -Infinity;

        for (const move of possibleMoves) {
          if (Date.now() - startTime > timeLimit) {
            console.log(`Time limit exceeded at depth ${depth}. Returning best move found so far.`);
            return currentBestMove; // Return the best move found so far
          }

          let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
          if (move.from === 'drop') {
            newGameState = dropPiece(newGameState, move.type, move.to);
          } else {
            newGameState = movePiece(newGameState, move.from, move.to, move.promote); // Pass promote flag
          }

          const { score } = await minimax(newGameState, 0, depth, !maximizingPlayer, -Infinity, Infinity, startTime, timeLimit, new Set()); // Use current depth
          console.log(`Minimax returned score: ${score} for move:`, move);

          if (score === null) {
            // If minimax returned null, it means time limit was exceeded for this branch
            // We should stop this iteration and return the best move found so far from previous depths.
            console.log(`Minimax returned null score, time limit hit. Returning currentBestMove.`);
            return currentBestMove;
          }
        }
        // If a full iteration is completed, update the overall best move
        if (iterationBestMove) {
          currentBestMove = iterationBestMove;
          currentBestScore = iterationBestScore;
        }
      }
      return currentBestMove;
    default:
      return null;
  }
}

function scoreMove(move, gameState) {
  let score = 0;
  const { board, currentPlayer } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  if (move.isCapture) {
    // MVV-LVA: Most Valuable Victim - Least Valuable Attacker
    const capturedPieceType = board[move.to[0]][move.to[1]]?.type;
    const attackingPieceType = board[move.from[0]][move.from[1]]?.type;

    if (capturedPieceType && attackingPieceType) {
      score += PIECE_VALUES[capturedPieceType] * 10 - PIECE_VALUES[attackingPieceType];
    }
    score += 1000; // Base bonus for captures
  }

  // Killer moves bonus
  if (move.from === killerMoves[0]?.from && move.to === killerMoves[0]?.to) {
    score += 900; // High bonus for first killer move
  } else if (move.from === killerMoves[1]?.from && move.to === killerMoves[1]?.to) {
    score += 800; // High bonus for second killer move
  }

  // History heuristic bonus
  if (move.from !== 'drop' && historyTable[move.from[0]] && historyTable[move.from[0]][move.from[1]]) {
    score += historyTable[move.from[0]][move.from[1]];
  }

  if (move.isCheck) {
    score += 50; // Priority for checks
  }
  return score;
}

async function minimax(gameState, depth, maxDepth, maximizingPlayer, alpha = -Infinity, beta = Infinity, startTime, timeLimit, history = new Set()) {
  const { board, currentPlayer, capturedPieces } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  if (Date.now() - startTime > timeLimit) {
    console.log(`Minimax time limit exceeded at depth ${depth}`);
    return { score: null, move: null }; // Indicate that the search was cut short
  }
  const hash = generateStateHash(gameState);

  // Check for repetition in actual game history
  for (const pastState of gameState.pastStates) {
    if (generateStateHash(pastState) === hash) {
      return { score: -100000, move: null }; // Strong repetition penalty
    }
  }

  // Check for repetition within the current search branch
  if (history.has(hash)) {
    return { score: -50000, move: null }; // Penalty for immediate repetition in search
  }

  if (transpositionTable.has(hash)) {
    const cached = transpositionTable.get(hash);
    if (cached.depth >= depth) {
      return { score: cached.score, move: null };
    }
  }

  // Null Move Pruning
  if (depth >= 3 && !isKingInCheck(board, currentPlayer)) { // Apply if depth is sufficient and not in check
    const nullMoveGameState = { ...gameState, currentPlayer: opponent };
    const { score: nullMoveScore } = await minimax(nullMoveGameState, depth - 1 - 2, maxDepth, !maximizingPlayer, -beta, -alpha, startTime, timeLimit, new Set()); // Reduced depth, inverted alpha-beta

    if (nullMoveScore === null) return { score: null, move: null }; // Propagate time limit exceeded

    if (nullMoveScore >= beta) {
      return { score: beta, move: null };
    }
  }

  if (depth === maxDepth || gameState.isCheckmate) {
    const score = await quiescenceSearch(gameState, alpha, beta, 0, startTime, timeLimit); // Call quiescence search at max depth
    transpositionTable.set(hash, { depth, score });
    return { score, move: null }; // Return null for move at terminal nodes
  }

  // Futility Pruning
  if (depth < maxDepth && !isKingInCheck(gameState.board, currentPlayer)) { // Only apply if not in check
    const evalScore = evaluateBoard(gameState);
    const margin = 200; // Adjust margin as needed
    if (maximizingPlayer && evalScore + margin <= alpha) {
      return { score: alpha, move: null };
    }
    if (!maximizingPlayer && evalScore - margin >= beta) {
      return { score: beta, move: null };
    }
  }

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

  for (let i = 0; i < possibleMoves.length; i++) {
    const move = possibleMoves[i];
    if (Date.now() - startTime > timeLimit) {
      return { score: 0, move: null }; // Abort if time limit exceeded during move iteration
    }
    await new Promise(resolve => setTimeout(resolve, 0)); // Yield control to the browser
    let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
    if (move.from === 'drop') {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to);
    }

    const newHistory = new Set(history);
    newHistory.add(hash);

    let reduction = 0;
    if (depth >= LMR_DEPTH && i >= 4 && !move.isCapture && !move.isCheck) { // Apply LMR if conditions met
      reduction = LMR_REDUCTION;
    }

    const { score, move: bestChildMove } = await minimax(newGameState, depth + 1 + reduction, maxDepth, !maximizingPlayer, alpha, beta, startTime, timeLimit, newHistory);

    if (score === null) return { score: null, move: null }; // Propagate time limit exceeded

    if (maximizingPlayer) {
      if (score > bestScore) {
        bestScore = score;
        bestMove = move;
      }
      alpha = Math.max(alpha, bestScore);
      if (beta <= alpha) {
        // Beta cut-off, this is a good move, add to killer moves and history
        if (!move.isCapture && !move.isCheck) { // Only non-captures and non-checks
          killerMoves[1] = killerMoves[0];
          killerMoves[0] = move;
        }
        if (move.from !== 'drop') {
          historyTable[move.from[0]][move.from[1]] += depth; // Update history score
        }
        break; // Beta cut-off
      }
    } else {
      if (score < bestScore) {
        bestScore = score;
        bestMove = move;
      }
      beta = Math.min(beta, bestScore);
      if (beta <= alpha) {
        // Alpha cut-off, this is a good move, add to killer moves and history
        if (!move.isCapture && !move.isCheck) { // Only non-captures and non-checks
          killerMoves[1] = killerMoves[0];
          killerMoves[0] = move;
        }
        if (move.from !== 'drop') {
          historyTable[move.from[0]][move.from[1]] += depth; // Update history score
        }
        break; // Alpha cut-off
      }
    }
  }
  return { score: bestScore, move: bestMove };
}

async function quiescenceSearch(gameState, alpha, beta, depth, startTime, timeLimit) {
  if (Date.now() - startTime > timeLimit) {
    console.log(`Quiescence search time limit exceeded at depth ${depth}`);
    return null; // Indicate that the search was cut short
  }
  let standPat = evaluateBoard(gameState);
  
  if (depth >= 3) { // Limit quiescence search depth
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