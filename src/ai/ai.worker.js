import { getLegalMoves, getLegalDrops, movePiece, dropPiece, isKingInCheck, isCheckmate, generateStateHash, PLAYER_1, PLAYER_2, PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING, PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK } from '../game/engine';
import openingBook from './openingBook.json';

let transpositionTable = new Map();

// This is the main entry point for the worker
self.onmessage = async (event) => {
  transpositionTable.clear();
  const { gameState, difficulty } = event.data;
  const bestMove = await getAiMove(gameState, difficulty);
  self.postMessage(bestMove);
};

// Piece values for evaluation (can be adjusted)
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

// Piece-Square Tables (PSTs)
const PST = {
  [PAWN]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [5, 5, 5, 5, 5, 5, 5, 5, 5],
    [10, 10, 10, 10, 10, 10, 10, 10, 10],
    [15, 15, 15, 15, 15, 15, 15, 15, 15],
    [20, 20, 20, 20, 20, 20, 20, 20, 20],
    [25, 25, 25, 25, 25, 25, 25, 25, 25],
    [30, 30, 30, 30, 30, 30, 30, 30, 30],
    [35, 35, 35, 35, 35, 35, 35, 35, 35],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [LANCE]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [KNIGHT]: [
    [-10, -10, -10, -10, -10, -10, -10, -10, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 5, 10, 15, 10, 5, 0, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, -10, -10, -10, -10, -10, -10, -10, -10]
  ],
  [SILVER]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [GOLD]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [BISHOP]: [
    [-10, -10, -10, -10, -10, -10, -10, -10, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, 0, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 5, 10, 15, 10, 5, 0, -10],
    [-10, 0, 5, 10, 10, 10, 5, 0, -10],
    [-10, 0, 5, 5, 5, 5, 5, 0, -10],
    [-10, 0, 0, 0, 0, 0, 0, 0, -10],
    [-10, -10, -10, -10, -10, -10, -10, -10, -10]
  ],
  [ROOK]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 5, 5, 5, 5, 5, 5, 5, 0],
    [0, 5, 10, 10, 10, 10, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 15, 20, 15, 10, 5, 0],
    [0, 5, 10, 15, 15, 15, 10, 5, 0],
    [0, 5, 10, 10, 10, 10, 10, 5, 0],
    [0, 5, 5, 5, 5, 5, 5, 5, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [KING]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [PROMOTED_PAWN]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [PROMOTED_LANCE]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [PROMOTED_KNIGHT]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [PROMOTED_SILVER]: [
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0]
  ],
  [PROMOTED_BISHOP]: [
    [-10, -10, -10, -10, -10, -10, -10, -10, -10],
    [-10, 5, 5, 5, 5, 5, 5, 5, -10],
    [-10, 5, 10, 10, 10, 10, 10, 5, -10],
    [-10, 5, 10, 15, 15, 15, 10, 5, -10],
    [-10, 5, 10, 15, 20, 15, 10, 5, -10],
    [-10, 5, 10, 15, 15, 15, 10, 5, -10],
    [-10, 5, 10, 10, 10, 10, 10, 5, -10],
    [-10, 5, 5, 5, 5, 5, 5, 5, -10],
    [-10, -10, -10, -10, -10, -10, -10, -10, -10]
  ],
  [PROMOTED_ROOK]: [
    [5, 5, 5, 5, 5, 5, 5, 5, 5],
    [5, 10, 10, 10, 10, 10, 10, 10, 5],
    [5, 10, 15, 15, 15, 15, 15, 10, 5],
    [5, 10, 15, 20, 20, 20, 15, 10, 5],
    [5, 10, 15, 20, 25, 20, 15, 10, 5],
    [5, 10, 15, 20, 20, 20, 15, 10, 5],
    [5, 10, 15, 15, 15, 15, 15, 10, 5],
    [5, 10, 10, 10, 10, 10, 10, 10, 5],
    [5, 5, 5, 5, 5, 5, 5, 5, 5]
  ],
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

  // Material and positional advantage
  let materialScore = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        const pst = PST[piece.type];
        const pstValue = piece.player === PLAYER_1 ? pst[r][c] : pst[8 - r][8 - c];
        if (piece.player === currentPlayer) {
          materialScore += PIECE_VALUES[piece.type] + pstValue;
        } else {
          materialScore -= (PIECE_VALUES[piece.type] + pstValue);
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

  // King Safety Evaluation
  let kingSafetyScore = 0;
  let kingPos = null;

  // Find the king's position
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === KING && piece.player === currentPlayer) {
        kingPos = [r, c];
        break;
      }
    }
    if (kingPos) break;
  }

  if (kingPos) {
    const [kingR, kingC] = kingPos;

    // Define a safety zone around the king (e.g., 3x3 area)
    const safetyZone = [];
    for (let r = Math.max(0, kingR - 1); r <= Math.min(8, kingR + 1); r++) {
      for (let c = Math.max(0, kingC - 1); c <= Math.min(8, kingC + 1); c++) {
        safetyZone.push([r, c]);
      }
    }

    // Count friendly defenders and enemy attackers in the safety zone
    let friendlyDefenders = 0;
    let enemyAttackers = 0;

    for (const [r, c] of safetyZone) {
      const piece = board[r][c];
      if (piece) {
        if (piece.player === currentPlayer) {
          friendlyDefenders += PIECE_VALUES[piece.type] / 100; // Smaller bonus for defenders
        } else {
          enemyAttackers += PIECE_VALUES[piece.type] / 50; // Larger penalty for attackers
        }
      }
    }

    kingSafetyScore += friendlyDefenders - enemyAttackers;

    // Penalize open files/ranks/diagonals leading to the king
    // Check file
    let fileOpen = true;
    for (let r = 0; r < 9; r++) {
      if (r !== kingR && board[r][kingC] && board[r][kingC].player === currentPlayer && board[r][kingC].type !== PAWN) {
        fileOpen = false;
        break;
      }
    }
    if (fileOpen) {
      kingSafetyScore -= 20; // Penalty for open file
    }

    // Check rank (less critical in Shogi, but still relevant)
    let rankOpen = true;
    for (let c = 0; c < 9; c++) {
      if (c !== kingC && board[kingR][c] && board[kingR][c].player === currentPlayer && board[kingR][c].type !== PAWN) {
        rankOpen = false;
        break;
      }
    }
    if (rankOpen) {
      kingSafetyScore -= 10; // Penalty for open rank
    }

    // Check diagonals (simplified for now)
    // Top-left to bottom-right diagonal
    let diag1Open = true;
    for (let i = -8; i <= 8; i++) {
      const r = kingR + i;
      const c = kingC + i;
      if (r >= 0 && r < 9 && c >= 0 && c < 9 && r !== kingR && c !== kingC && board[r][c] && board[r][c].player === currentPlayer && board[r][c].type !== PAWN) {
        diag1Open = false;
        break;
      }
    }
    if (diag1Open) {
      kingSafetyScore -= 15; // Penalty for open diagonal
    }

    // Top-right to bottom-left diagonal
    let diag2Open = true;
    for (let i = -8; i <= 8; i++) {
      const r = kingR + i;
      const c = kingC - i;
      if (r >= 0 && r < 9 && c >= 0 && c < 9 && r !== kingR && c !== kingC && board[r][c] && board[r][c].player === currentPlayer && board[r][c].type !== PAWN) {
        diag2Open = false;
        break;
      }
    }
    if (diag2Open) {
      kingSafetyScore -= 15; // Penalty for open diagonal
    }
  }

  score += kingSafetyScore;

  return score;

  // Piece Mobility
  let mobilityScore = 0;
  let currentPlayerMobility = 0;
  let opponentMobility = 0;

  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        const moves = getLegalMoves(piece, r, c, board);
        if (piece.player === currentPlayer) {
          currentPlayerMobility += moves.length;
        } else {
          opponentMobility += moves.length;
        }
      }
    }
  }
  mobilityScore = (currentPlayerMobility - opponentMobility) * 0.5; // Adjust multiplier as needed
  score += mobilityScore;

  // Pawn Structure
  let pawnStructureScore = 0;
  const pawns = { [PLAYER_1]: [], [PLAYER_2]: [] };

  // Collect pawn positions
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === PAWN) {
        pawns[piece.player].push([r, c]);
      }
    }
  }

  // Evaluate pawn chains (connected pawns)
  for (const player of [PLAYER_1, PLAYER_2]) {
    for (const [r, c] of pawns[player]) {
      const direction = player === PLAYER_1 ? -1 : 1; // Pawns move up for player1, down for player2
      // Check for pawn in front-left or front-right
      if (c > 0 && board[r + direction]?.[c - 1]?.type === PAWN && board[r + direction][c - 1].player === player) {
        pawnStructureScore += (player === currentPlayer ? 5 : -5); // Bonus for connected pawns
      }
      if (c < 8 && board[r + direction]?.[c + 1]?.type === PAWN && board[r + direction][c + 1].player === player) {
        pawnStructureScore += (player === currentPlayer ? 5 : -5); // Bonus for connected pawns
      }
    }
  }

  // Penalize isolated pawns (no friendly pawns on adjacent files)
  for (const player of [PLAYER_1, PLAYER_2]) {
    for (const [r, c] of pawns[player]) {
      let isolated = true;
      if (c > 0) {
        for (let r2 = 0; r2 < 9; r2++) {
          if (board[r2][c - 1]?.type === PAWN && board[r2][c - 1].player === player) {
            isolated = false;
            break;
          }
        }
      }
      if (c < 8) {
        for (let r2 = 0; r2 < 9; r2++) {
          if (board[r2][c + 1]?.type === PAWN && board[r2][c + 1].player === player) {
            isolated = false;
            break;
          }
        }
      }
      if (isolated) {
        pawnStructureScore += (player === currentPlayer ? -10 : 10); // Penalty for isolated pawns
      }
    }
  }
  score += pawnStructureScore;

  // Threats and Defenses
  let threatDefenseScore = 0;

  // Initialize attack and defense counts for each square for both players
  const attackCounts = {
    [PLAYER_1]: Array(9).fill(0).map(() => Array(9).fill(0)),
    [PLAYER_2]: Array(9).fill(0).map(() => Array(9).fill(0)),
  };
  const defenseCounts = {
    [PLAYER_1]: Array(9).fill(0).map(() => Array(9).fill(0)),
    [PLAYER_2]: Array(9).fill(0).map(() => Array(9).fill(0)),
  };

  // Populate attack and defense counts
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        const moves = getLegalMoves(piece, r, c, board);
        moves.forEach(to => {
          const targetPiece = board[to[0]][to[1]];
          if (targetPiece && targetPiece.player !== piece.player) {
            // This piece attacks an opponent's piece
            attackCounts[piece.player][to[0]][to[1]]++;
          } else if (targetPiece && targetPiece.player === piece.player) {
            // This piece defends a friendly piece
            defenseCounts[piece.player][to[0]][to[1]]++;
          } else {
            // This piece attacks an empty square (control of square) - can be considered for mobility or general influence
            // For now, we'll focus on direct attacks/defenses of pieces
          }
        });
      }
    }
  }

  // Evaluate based on attack and defense counts
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        const currentPieceAttacks = attackCounts[currentPlayer][r][c];
        const opponentPieceAttacks = attackCounts[opponent][r][c];
        const currentPieceDefenses = defenseCounts[currentPlayer][r][c];
        const opponentPieceDefenses = defenseCounts[opponent][r][c];

        if (piece.player === currentPlayer) {
          // If our piece is attacked more than defended by us
          if (opponentPieceAttacks > currentPieceDefenses) {
            threatDefenseScore -= PIECE_VALUES[piece.type] * 0.5; // Penalty for undefended attacked piece
          }
          // If our piece is defended more than attacked by opponent
          if (currentPieceDefenses > opponentPieceAttacks) {
            threatDefenseScore += PIECE_VALUES[piece.type] * 0.1; // Bonus for well-defended piece
          }
        } else { // Opponent's piece
          // If opponent's piece is attacked more than defended by them
          if (currentPieceAttacks > opponentPieceDefenses) {
            threatDefenseScore += PIECE_VALUES[piece.type] * 0.5; // Bonus for attacking undefended opponent piece
          }
          // If opponent's piece is defended more than attacked by us
          if (opponentPieceDefenses > currentPieceAttacks) {
            threatDefenseScore -= PIECE_VALUES[piece.type] * 0.1; // Penalty for well-defended opponent piece
          }
        }
      }
    }
  }
  score += threatDefenseScore;

  // Passed Pawns
  let passedPawnScore = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === PAWN) {
        let isPassed = true;
        const direction = piece.player === PLAYER_1 ? -1 : 1; // Direction to opponent's side
        const startRank = piece.player === PLAYER_1 ? r - 1 : r + 1;
        const endRank = piece.player === PLAYER_1 ? -1 : 9;

        // Check current file and adjacent files for opponent pawns
        for (let i = startRank; i !== endRank; i += direction) {
          if (board[i]?.[c]?.type === PAWN && board[i][c].player !== piece.player) {
            isPassed = false;
            break;
          }
          if (c > 0 && board[i]?.[c - 1]?.type === PAWN && board[i][c - 1].player !== piece.player) {
            isPassed = false;
            break;
          }
          if (c < 8 && board[i]?.[c + 1]?.type === PAWN && board[i][c + 1].player !== piece.player) {
            isPassed = false;
            break;
          }
        }

        if (isPassed) {
          // Bonus for passed pawns, higher if closer to promotion
          const distanceToPromotion = piece.player === PLAYER_1 ? r : 8 - r;
          passedPawnScore += (piece.player === currentPlayer ? 20 + (distanceToPromotion * 5) : -(20 + (distanceToPromotion * 5)));
        }
      }
    }
  }
  score += passedPawnScore;

  // King's Escape Squares
  let kingEscapeScore = 0;
  if (kingPos) {
    const [kingR, kingC] = kingPos;
    let escapeSquares = 0;

    // Check all 8 surrounding squares (and the king's own square if it can move there)
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        if (dr === 0 && dc === 0) continue; // Skip king's current square

        const newR = kingR + dr;
        const newC = kingC + dc;

        if (newR >= 0 && newR < 9 && newC >= 0 && newC < 9) {
          // Temporarily move the king to the new square to check for checks
          const originalPiece = board[newR][newC];
          const tempBoard = JSON.parse(JSON.stringify(board)); // Deep copy
          tempBoard[newR][newC] = { type: KING, player: currentPlayer };
          tempBoard[kingR][kingC] = null;

          if (!isKingInCheck(tempBoard, currentPlayer)) {
            escapeSquares++;
          }
        }
      }
    }
    kingEscapeScore = escapeSquares * 5; // Bonus for each escape square
  }
  score += kingEscapeScore;

  // Piece Development (Active Squares)
  let developmentScore = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece) {
        if (piece.player === currentPlayer) {
          // Reward pieces in opponent's territory
          if (currentPlayer === PLAYER_1 && r <= 3) { // Rows 0-3 for Player 1
            developmentScore += PIECE_VALUES[piece.type] * 0.05; // Small bonus
          } else if (currentPlayer === PLAYER_2 && r >= 5) { // Rows 5-8 for Player 2
            developmentScore += PIECE_VALUES[piece.type] * 0.05; // Small bonus
          }
        } else { // Opponent's piece
          // Penalize opponent's pieces in our territory
          if (opponent === PLAYER_1 && r <= 3) { // Rows 0-3 for Player 1
            developmentScore -= PIECE_VALUES[piece.type] * 0.05; // Small penalty
          } else if (opponent === PLAYER_2 && r >= 5) { // Rows 5-8 for Player 2
            developmentScore -= PIECE_VALUES[piece.type] * 0.05; // Small penalty
          }
        }
      }
    }
  }
  score += developmentScore;

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
const LMR_DEPTH = 3; // Minimum depth for LMR to apply
const LMR_REDUCTION = 1; // How much to reduce the depth by

async function quiescenceSearch(gameState, alpha, beta, depth, startTime, timeLimit) {
  if (Date.now() - startTime > timeLimit) {
    console.log(`Quiescence search time limit exceeded at depth ${depth}`);
    return null; // Indicate that the search was cut short
  }
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

    const { score } = await minimax(newGameState, depth + 1 + reduction, maxDepth, !maximizingPlayer, alpha, beta, startTime, timeLimit, newHistory);

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

async function getAiMove(gameState, difficulty) {
  const { currentPlayer } = gameState;
  const maximizingPlayer = currentPlayer === PLAYER_2; // AI is always Player 2
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  // Check opening book
  const currentHash = generateStateHash(gameState);
  if (openingBook[currentHash]) {
    const moves = openingBook[currentHash];
    const randomIndex = Math.floor(Math.random() * moves.length);
    console.log("Move from opening book:", moves[randomIndex]);
    return moves[randomIndex];
  }

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