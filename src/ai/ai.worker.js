import {
  getLegalMoves,
  getLegalDrops,
  movePiece,
  dropPiece,
  isKingInCheck,
  isCheckmate,
  generateStateHash,
  getAttackedSquares,
  PLAYER_1,
  PLAYER_2,
  PAWN,
  LANCE,
  KNIGHT,
  SILVER,
  GOLD,
  BISHOP,
  ROOK,
  KING,
  PROMOTED_PAWN,
  PROMOTED_LANCE,
  PROMOTED_KNIGHT,
  PROMOTED_SILVER,
  PROMOTED_BISHOP,
  PROMOTED_ROOK,
} from "../game/engine";
import openingBook from "./openingBook.json";

let transpositionTable = new Map();
let historyTable = Array(9)
  .fill(0)
  .map(() => Array(9).fill(0)); // For history heuristic
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
  [0, 0, 0, 0, 0, 0, 0, 0, 0],
  [5, 5, 5, 5, 5, 5, 5, 5, 5],
  [10, 10, 10, 10, 10, 10, 10, 10, 10],
  [15, 15, 15, 15, 15, 15, 15, 15, 15],
  [20, 20, 20, 20, 20, 20, 20, 20, 20],
  [25, 25, 25, 25, 25, 25, 25, 25, 25],
  [30, 30, 30, 30, 30, 30, 30, 30, 30],
  [35, 35, 35, 35, 35, 35, 35, 35, 35],
  [0, 0, 0, 0, 0, 0, 0, 0, 0],
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
  [0, 0, 5, 10, 10, 10, 5, 0, 0],
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
  [-10, -10, -10, -10, -10, -10, -10, -10, -10],
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
  [0, 0, 0, 0, 0, 0, 0, 0, 0],
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
  [0, 0, 0, 0, 0, 0, 0, 0, 0],
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
  [-10, -10, -10, -10, -10, -10, -10, -10, -10],
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
  [0, 5, 10, 15, 15, 15, 10, 5, 0],
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
    [-1, -1],
    [-1, 0],
    [-1, 1],
    [0, -1],
    [0, 1],
    [1, -1],
    [1, 0],
    [1, 1],
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
    if (
      move.to[0] >= 3 &&
      move.to[0] <= 5 &&
      move.to[1] >= 3 &&
      move.to[1] <= 5
    ) {
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
        const pstValue =
          piece.player === PLAYER_1 ? pst[r][c] : pst[8 - r][8 - c];
        if (piece.player === currentPlayer) {
          score += PIECE_VALUES[piece.type] + pstValue;
        } else {
          score -= PIECE_VALUES[piece.type] + pstValue;
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

  // Enhanced Pawn Structure Analysis
  score += evaluatePawnStructure(board, currentPlayer);
  score -= evaluatePawnStructure(board, opponent);

  // King Activity in Endgame
  score += evaluateKingActivity(board, currentPlayer, playerKingPos, capturedPieces);
  score -= evaluateKingActivity(board, opponent, opponentKingPos, capturedPieces);

  // Connected Rooks Bonus
  score += evaluateConnectedRooks(board, currentPlayer);
  score -= evaluateConnectedRooks(board, opponent);

  // Mobility Score
  const playerMobility = getMobilityScore(board, currentPlayer);
  const opponentMobility = getMobilityScore(board, opponent);
  score += (playerMobility - opponentMobility) * 10; // Adjust weight as needed

  // Threat Analysis
  const attackedByPlayer = getAttackedSquares(board, currentPlayer);
  const attackedByOpponent = getAttackedSquares(board, opponent);

  for (const square of attackedByPlayer) {
    const [r, c] = square.split(",").map(Number);
    const piece = board[r][c];
    if (piece && piece.player === opponent) {
      score += PIECE_VALUES[piece.type] / 5;
    }
  }

  for (const square of attackedByOpponent) {
    const [r, c] = square.split(",").map(Number);
    const piece = board[r][c];
    if (piece && piece.player === currentPlayer) {
      score -= PIECE_VALUES[piece.type] / 5;
    }
  }

  return score;
}

/**
 * Finds the position of a player's king on the board
 * @param {Array<Array>} board - The current board state
 * @param {string} player - The player whose king to find
 * @returns {Array|null} - [row, col] of the king, or null if not found
 */
function findKingPosition(board, player) {
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === KING && piece.player === player) {
        return [r, c];
      }
    }
  }
  return null;
}

/**
 * Evaluates the safety of a drop move by checking if the piece would be immediately captured
 * @param {object} gameState - The current game state
 * @param {string} pieceType - The type of piece being dropped
 * @param {Array} dropPosition - [row, col] where the piece would be dropped
 * @returns {object} - {isSafe: boolean, safetyScore: number, immediateThreats: Array}
 */
function evaluateDropSafety(gameState, pieceType, dropPosition) {
  const { board, currentPlayer } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  const [dropRow, dropCol] = dropPosition;
  
  // Simulate the drop
  const simulatedBoard = board.map(row => [...row]);
  simulatedBoard[dropRow][dropCol] = { type: pieceType, player: currentPlayer };
  
  // Get all squares the opponent can attack after the drop
  const attackedSquares = getAttackedSquares(simulatedBoard, opponent);
  const dropSquareKey = `${dropRow},${dropCol}`;
  
  // Check if the drop square is under immediate attack
  const isUnderImmediateAttack = attackedSquares.has(dropSquareKey);
  
  if (isUnderImmediateAttack) {
    // Find what pieces can attack this square
    const immediateThreats = [];
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        const piece = simulatedBoard[r][c];
        if (piece && piece.player === opponent) {
          const moves = getLegalMoves(piece, r, c, simulatedBoard);
          if (moves.some(([moveRow, moveCol]) => moveRow === dropRow && moveCol === dropCol)) {
            immediateThreats.push({
              piece: piece.type,
              position: [r, c],
              value: PIECE_VALUES[piece.type]
            });
          }
        }
      }
    }
    
    // Calculate safety score - heavily penalize drops that would be captured
    const pieceValue = PIECE_VALUES[pieceType];
    const totalThreatValue = immediateThreats.reduce((sum, threat) => sum + threat.value, 0);
    
    // If the piece would be captured by a piece of equal or higher value, it's very bad
    const worstThreat = Math.max(...immediateThreats.map(t => t.value));
    let safetyScore = -pieceValue * 2; // Base penalty for being under attack
    
    if (worstThreat >= pieceValue) {
      safetyScore -= pieceValue * 3; // Additional penalty for being captured by equal/higher value piece
    }
    
    console.log(`AI: Unsafe drop detected for ${pieceType} at [${dropRow},${dropCol}]. Threats:`, immediateThreats.map(t => `${t.piece}(${t.value})`).join(', '), `Safety score: ${safetyScore}`);
    
    return {
      isSafe: false,
      safetyScore,
      immediateThreats,
      worstThreat
    };
  }
  
  // Check for potential future threats (pieces that could move to attack this square)
  const potentialThreats = [];
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = simulatedBoard[r][c];
      if (piece && piece.player === opponent) {
        // Check if this piece could move to attack the drop square in one move
        const moves = getLegalMoves(piece, r, c, simulatedBoard);
        for (const [moveRow, moveCol] of moves) {
          // If the piece can move to a position where it could then attack the drop square
          const tempBoard = simulatedBoard.map(row => [...row]);
          tempBoard[moveRow][moveCol] = piece;
          tempBoard[r][c] = null;
          
          const movesAfterMove = getLegalMoves(piece, moveRow, moveCol, tempBoard);
          if (movesAfterMove.some(([finalRow, finalCol]) => finalRow === dropRow && finalCol === dropCol)) {
            potentialThreats.push({
              piece: piece.type,
              position: [r, c],
              value: PIECE_VALUES[piece.type]
            });
            break;
          }
        }
      }
    }
  }
  
  // Small penalty for potential future threats
  const potentialThreatScore = potentialThreats.length * -10;
  
  // Bonus for drops that create tactical advantages
  let tacticalBonus = 0;
  
  // Bonus for drops that attack opponent pieces
  const movesAfterDrop = getLegalMoves({ type: pieceType, player: currentPlayer }, dropRow, dropCol, simulatedBoard);
  for (const [moveRow, moveCol] of movesAfterDrop) {
    const targetPiece = simulatedBoard[moveRow][moveCol];
    if (targetPiece && targetPiece.player === opponent) {
      // Higher bonus for attacking higher-value pieces
      const attackBonus = PIECE_VALUES[targetPiece.type] / 8;
      tacticalBonus += attackBonus;
      
      // Extra bonus if the attack creates a fork (attacking multiple pieces)
      let forkBonus = 0;
      for (const [otherMoveRow, otherMoveCol] of movesAfterDrop) {
        if (otherMoveRow !== moveRow || otherMoveCol !== moveCol) {
          const otherTarget = simulatedBoard[otherMoveRow][otherMoveCol];
          if (otherTarget && otherTarget.player === opponent) {
            forkBonus += PIECE_VALUES[otherTarget.type] / 15;
          }
        }
      }
      tacticalBonus += forkBonus;
    }
  }
  
  // Bonus for drops that control important squares (center, near opponent king)
  if (dropRow >= 3 && dropRow <= 5 && dropCol >= 3 && dropCol <= 5) {
    tacticalBonus += 15; // Center control bonus
  }
  
  // Bonus for drops that create fork or pin opportunities
  const opponentKingPos = findKingPosition(simulatedBoard, opponent);
  if (opponentKingPos) {
    const [kingRow, kingCol] = opponentKingPos;
    const distanceToKing = Math.abs(dropRow - kingRow) + Math.abs(dropCol - kingCol);
    if (distanceToKing <= 3) {
      tacticalBonus += 20; // Bonus for drops near opponent king
    }
  }
  
  // Bonus for drops that protect friendly pieces
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = simulatedBoard[r][c];
      if (piece && piece.player === currentPlayer && piece.type !== KING) {
        const distance = Math.abs(dropRow - r) + Math.abs(dropCol - c);
        if (distance === 1) {
          tacticalBonus += 10; // Small bonus for protecting adjacent friendly pieces
        }
      }
    }
  }
  
  // Consider opponent's defensive capabilities
  let defensiveBonus = 0;
  const opponentAttackedSquares = getAttackedSquares(simulatedBoard, opponent);
  
  // Bonus if the drop square is protected by our pieces
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = simulatedBoard[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, simulatedBoard);
        if (moves.some(([moveRow, moveCol]) => moveRow === dropRow && moveCol === dropCol)) {
          defensiveBonus += 15; // Bonus for protected drops
          break;
        }
      }
    }
  }
  
  // Penalty if opponent can easily defend against the drop
  let opponentDefenseCount = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = simulatedBoard[r][c];
      if (piece && piece.player === opponent) {
        const moves = getLegalMoves(piece, r, c, simulatedBoard);
        if (moves.some(([moveRow, moveCol]) => {
          // Check if this move would defend against our drop
          const tempBoard = simulatedBoard.map(row => [...row]);
          tempBoard[moveRow][moveCol] = piece;
          tempBoard[r][c] = null;
          
          // If after this move, our drop piece can't attack effectively
          const movesAfterOpponentMove = getLegalMoves({ type: pieceType, player: currentPlayer }, dropRow, dropCol, tempBoard);
          return movesAfterOpponentMove.length === 0 || 
                 movesAfterOpponentMove.every(([finalRow, finalCol]) => {
                   const target = tempBoard[finalRow][finalCol];
                   return !target || target.player === currentPlayer;
                 });
        })) {
          opponentDefenseCount++;
        }
      }
    }
  }
  
  if (opponentDefenseCount > 0) {
    defensiveBonus -= opponentDefenseCount * 10; // Penalty for easily defended drops
  }
  
  // Consider whether this is a good time to drop this piece
  let timingBonus = 0;
  
  // Bonus for dropping pieces when we have a material advantage (can afford to lose the piece)
  const materialAdvantage = evaluateBoard(gameState) - evaluateBoard({...gameState, currentPlayer: opponent});
  if (materialAdvantage > 500) {
    timingBonus += 25; // Bonus for dropping when ahead
  }
  
  // Penalty for dropping valuable pieces early in the game
  if (gameState.moveHistory && gameState.moveHistory.length < 20) {
    if (pieceType === BISHOP || pieceType === ROOK) {
      timingBonus -= 30; // Prefer to keep major pieces early
    } else if (pieceType === GOLD || pieceType === SILVER) {
      timingBonus -= 20; // Prefer to keep minor pieces early
    }
  }
  
  const finalSafetyScore = potentialThreatScore + tacticalBonus + defensiveBonus + timingBonus;
  
  // Log safe drops with high tactical value for debugging
  if (tacticalBonus > 50) {
    console.log(`AI: Safe drop for ${pieceType} at [${dropRow},${dropCol}]. Tactical bonus: ${tacticalBonus}, Final score: ${finalSafetyScore}`);
  }
  
  return {
    isSafe: true,
    safetyScore: finalSafetyScore,
    immediateThreats: [],
    potentialThreats
  };
}

async function getAiMove(gameState, difficulty) {
  const { currentPlayer, moveHistory } = gameState;

  // Check opening book first
  const tempGameState = {
    ...gameState,
    currentPlayer: currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1,
  };
  const boardHash = generateStateHash(tempGameState);
  console.log("boardHash", boardHash);
  const availableOpenings = [];
  for (const opening of openingBook) {
    if (opening.moves[boardHash]) {
      availableOpenings.push(opening);
    }
  }

  if (availableOpenings.length > 0) {
    const chosenOpening =
      availableOpenings[Math.floor(Math.random() * availableOpenings.length)];
    console.log(`AI: Choosing move from opening: ${chosenOpening.name}`);
    const moves = chosenOpening.moves[boardHash];
    const move = moves[Math.floor(Math.random() * moves.length)];

    // Convert Shogi coordinates to internal coordinates
    const from = [
      parseInt(move.from[1], 10) - 1,
      9 - parseInt(move.from[0], 10),
    ];
    const to = [parseInt(move.to[1], 10) - 1, 9 - parseInt(move.to[0], 10)];

    return {
      from,
      to,
      type: "move",
      isCapture: false,
      isCheck: false,
      promote: false,
    };
  }

  const maximizingPlayer = currentPlayer === PLAYER_2; // AI is always Player 2

  const possibleMoves = [];

  // Collect all possible moves for pieces on the board
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = gameState.board[r][c];
      if (piece && piece.player === currentPlayer) {
        const moves = getLegalMoves(piece, r, c, gameState.board);
        moves.forEach((to) => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const isCapture =
            gameState.board[to[0]][to[1]] &&
            gameState.board[to[0]][to[1]].player !== currentPlayer;
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer,
          );
          const promotionZoneStart = currentPlayer === PLAYER_1 ? 2 : 6;
          const inPromotionZone =
            (currentPlayer === PLAYER_1 && to[0] <= promotionZoneStart) ||
            (currentPlayer === PLAYER_2 && to[0] >= promotionZoneStart);
          const wasInPromotionZone =
            (currentPlayer === PLAYER_1 && r <= promotionZoneStart) ||
            (currentPlayer === PLAYER_2 && r >= promotionZoneStart);
          const promotablePieces = [PAWN, LANCE, KNIGHT, SILVER, BISHOP, ROOK];
          const canPromote =
            promotablePieces.includes(piece.type) &&
            (inPromotionZone || wasInPromotionZone);
          const lastRank = currentPlayer === PLAYER_1 ? 0 : 8;
          const secondLastRank = currentPlayer === PLAYER_1 ? 1 : 7;
          let isPromotionMandatory = false;
          if (
            (piece.type === PAWN || piece.type === LANCE) &&
            to[0] === lastRank
          ) {
            isPromotionMandatory = true;
          }
          if (
            piece.type === KNIGHT &&
            (to[0] === lastRank || to[0] === secondLastRank)
          ) {
            isPromotionMandatory = true;
          }

          let promote = false;
          if (canPromote && !isPromotionMandatory) {
            promote = true; // AI always promotes if optional
          } else if (isPromotionMandatory) {
            promote = true; // Mandatory promotion
          }

          if (!isKingInCheck(simulatedGameState.board, currentPlayer)) {
            // Only add if the move doesn't put own king in check
            possibleMoves.push({
              from: [r, c],
              to,
              type: "move",
              isCapture,
              isCheck,
              promote,
            });
          }
        });
      }
    }
  }

  // Collect all possible drops for captured pieces
  gameState.capturedPieces[currentPlayer].forEach((capturedPiece) => {
    const legalDrops = getLegalDrops(gameState, capturedPiece.type);
    legalDrops.forEach((to) => {
      const simulatedGameState = dropPiece(gameState, capturedPiece.type, to);
      if (
        simulatedGameState !== gameState &&
        !isKingInCheck(simulatedGameState.board, currentPlayer)
      ) {
        // Evaluate drop safety before adding to possible moves
        const dropSafety = evaluateDropSafety(gameState, capturedPiece.type, to);
        
        // Only add drops that aren't extremely unsafe (immediate capture by equal/higher value piece)
        const pieceValue = PIECE_VALUES[capturedPiece.type];
        const isExtremelyUnsafe = !dropSafety.isSafe && dropSafety.worstThreat >= pieceValue;
        
        if (!isExtremelyUnsafe) {
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1,
          );
          possibleMoves.push({
            from: "drop",
            to,
            type: capturedPiece.type,
            isCapture: false,
            isPromotion: false,
            isCheck,
          });
        }
      }
    });
  });

  // Enhanced move ordering for better alpha-beta pruning
  possibleMoves.sort(
    (a, b) => scoreMoveEnhanced(b, gameState) - scoreMoveEnhanced(a, gameState),
  );

  const startTime = Date.now();
  const timeLimit =
    difficulty === "easy" ? 1000 : difficulty === "medium" ? 3000 : 9000; // 1s for easy, 3s for medium, 9s for hard

  if (possibleMoves.length === 0) {
    console.log("No legal moves available for AI.");
    return null; // No legal moves available
  }

  // Iterative Deepening with Time Management
  let bestMove = possibleMoves[0];
  let bestScore = -Infinity;
  let currentDepth = 1;
  const maxDepth = difficulty === "easy" ? 3 : difficulty === "medium" ? 4 : 6;
  
  // Reserve some time for the final iteration
  const timeReserve = timeLimit * 0.1; // Reserve 10% of time
  const searchTimeLimit = timeLimit - timeReserve;
  
  console.log(`AI: Starting iterative deepening search with time limit: ${searchTimeLimit}ms`);
  
  // Aspiration windows for better search efficiency
  let alpha = -Infinity;
  let beta = Infinity;
  let windowSize = 100; // Initial window size
  
  // Dynamic time management based on position complexity
  let timePerMove = searchTimeLimit / maxDepth;
  let earlyExitThreshold = 0.8; // Exit early if we're using too much time
  
  while (currentDepth <= maxDepth && (Date.now() - startTime) < searchTimeLimit) {
    const timeUsed = Date.now() - startTime;
    const timeRemaining = searchTimeLimit - timeUsed;
    
    // Adjust search time based on position complexity
    if (currentDepth > 3) {
      const complexityFactor = Math.min(2.0, Math.max(0.5, Math.abs(bestScore) / 500));
      timePerMove = (timeRemaining / (maxDepth - currentDepth + 1)) * complexityFactor;
    }
    
    console.log(`AI: Searching at depth ${currentDepth} with window [${alpha}, ${beta}], time remaining: ${timeRemaining}ms`);
    
    try {
      const { move, score } = await pvs(
        gameState,
        currentDepth,
        alpha,
        beta,
        maximizingPlayer,
        startTime,
        searchTimeLimit,
        new Set(),
      );
      
      if (move && score !== null) {
        bestMove = move;
        bestScore = score;
        console.log(`AI: Depth ${currentDepth} completed. Best move:`, bestMove, `Score: ${bestScore}`);
        
        // Update aspiration window for next iteration
        if (score <= alpha) {
          // Score is below lower bound, expand window downward
          beta = alpha;
          alpha = score - windowSize;
          windowSize *= 2; // Increase window size
        } else if (score >= beta) {
          // Score is above upper bound, expand window upward
          alpha = beta;
          beta = score + windowSize;
          windowSize *= 2; // Increase window size
        } else {
          // Score is within bounds, narrow window for next iteration
          alpha = score - windowSize / 2;
          beta = score + windowSize / 2;
          windowSize = Math.max(50, windowSize / 2); // Decrease window size but maintain minimum
        }
        
        // Early exit if we're clearly winning and have limited time
        if (bestScore > 800 && timeRemaining < searchTimeLimit * earlyExitThreshold) {
          console.log(`AI: Position is clearly winning (${bestScore}), exiting early due to time constraints`);
          break;
        }
        
      } else {
        // Time limit exceeded or search failed
        console.log(`AI: Search at depth ${currentDepth} was interrupted or failed`);
        break;
      }
      
      currentDepth++;
      
      // If we're already winning by a lot, don't search deeper
      if (bestScore > 1000 && currentDepth > 3) {
        console.log(`AI: Position is clearly winning (${bestScore}), stopping search at depth ${currentDepth - 1}`);
        break;
      }
      
      // If we're losing by a lot, search deeper to find better moves
      if (bestScore < -1000 && currentDepth <= maxDepth) {
        console.log(`AI: Position is difficult (${bestScore}), continuing search to depth ${currentDepth}`);
      }
      
      // Check if we should continue based on time and score stability
      if (currentDepth > 3) {
        const timePerDepth = timeUsed / (currentDepth - 1);
        const estimatedTimeForNextDepth = timePerDepth * 2; // Rough estimate
        
        if (estimatedTimeForNextDepth > timeRemaining * 0.7) {
          console.log(`AI: Estimated time for next depth (${estimatedTimeForNextDepth}ms) exceeds available time (${timeRemaining}ms), stopping search`);
          break;
        }
      }
      
    } catch (error) {
      console.log(`AI: Error at depth ${currentDepth}:`, error);
      break;
    }
  }
  
  const totalTime = Date.now() - startTime;
  console.log(`AI: Search completed in ${totalTime}ms. Final depth: ${currentDepth - 1}. Best move:`, bestMove, `Score: ${bestScore}`);
  
  return bestMove;
}

async function pvs(
  gameState,
  depth,
  alpha,
  beta,
  maximizingPlayer,
  startTime,
  timeLimit,
  history,
) {
  if (Date.now() - startTime > timeLimit) {
    return { score: 0, move: null };
  }

  const hash = generateStateHash(gameState);
  if (history.has(hash)) {
    return { score: 0, move: null };
  }

  // Check endgame tablebase first
  const endgameResult = getEndgameTablebaseMove(gameState.board, gameState.currentPlayer, gameState.capturedPieces);
  if (endgameResult) {
    if (endgameResult.isDraw) {
      return { score: 0, move: null };
    } else if (endgameResult.isWinning) {
      const score = maximizingPlayer ? endgameResult.score : -endgameResult.score;
      return { score, move: null };
    }
  }

  if (depth === 0) {
    const score = await quiescenceSearch(
      gameState,
      alpha,
      beta,
      0,
      startTime,
      timeLimit,
    );
    return { score, move: null };
  }

  // Null Move Pruning - if we can skip a move and still be winning, the position is very good
  if (depth >= 3 && !isKingInCheck(gameState.board, gameState.currentPlayer)) {
    const nullMoveState = { ...gameState, currentPlayer: gameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1 };
    const nullMoveResult = await pvs(
      nullMoveState,
      depth - 3, // Reduced depth for null move
      -beta,
      -beta + 1,
      !maximizingPlayer,
      startTime,
      timeLimit,
      new Set(history),
    );
    
    if (-nullMoveResult.score >= beta) {
      return { score: beta, move: null }; // Beta cutoff
    }
  }

  // Futility Pruning - if we're way ahead and this is a quiet move, skip deep search
  const standPat = evaluateBoard(gameState);
  const futilityMargin = 300; // Material value threshold
  
  if (depth <= 2 && !maximizingPlayer && standPat - futilityMargin >= beta) {
    return { score: standPat, move: null };
  }
  
  if (depth <= 2 && maximizingPlayer && standPat + futilityMargin <= alpha) {
    return { score: standPat, move: null };
  }

  const possibleMoves = getPossibleMoves(gameState);
  possibleMoves.sort(
    (a, b) => scoreMoveEnhanced(b, gameState) - scoreMoveEnhanced(a, gameState),
  );

  let bestMove = null;
  let score = -Infinity;

  for (let i = 0; i < possibleMoves.length; i++) {
    const move = possibleMoves[i];
    let newGameState = { ...gameState, pastStates: [] };
    if (move.from === "drop") {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to, move.promote);
    }

    const newHistory = new Set(history);
    newHistory.add(hash);

    let result;
    if (i === 0) {
      result = await pvs(
        newGameState,
        depth - 1,
        -beta,
        -alpha,
        !maximizingPlayer,
        startTime,
        timeLimit,
        newHistory,
      );
      score = -result.score;
    } else {
      result = await pvs(
        newGameState,
        depth - 1,
        -alpha - 1,
        -alpha,
        !maximizingPlayer,
        startTime,
        timeLimit,
        newHistory,
      );
      score = -result.score;
      if (alpha < score && score < beta) {
        result = await pvs(
          newGameState,
          depth - 1,
          -beta,
          -score,
          !maximizingPlayer,
          startTime,
          timeLimit,
          newHistory,
        );
        score = -result.score;
      }
    }

    if (score > alpha) {
      alpha = score;
      bestMove = move;
      
      // Update killer moves for quiet moves
      if (!move.isCapture && !move.isCheck) {
        if (killerMoves[0] === null || (killerMoves[0].from !== move.from || 
            killerMoves[0].to[0] !== move.to[0] || killerMoves[0].to[1] !== move.to[1])) {
          killerMoves[1] = killerMoves[0];
          killerMoves[0] = { from: move.from, to: move.to };
        }
      }
      
      // Update history table
      if (move.from !== "drop") {
        historyTable[move.from[0]][move.from[1]] += depth * depth;
      }
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
    const attackingPieceType =
      move.from === "drop"
        ? move.type
        : board[move.from[0]][move.from[1]]?.type;

    if (capturedPieceType && attackingPieceType) {
      score +=
        PIECE_VALUES[capturedPieceType] * 10 - PIECE_VALUES[attackingPieceType];
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
    if (
      killerMoves[0] &&
      move.from === killerMoves[0].from &&
      move.to[0] === killerMoves[0].to[0] &&
      move.to[1] === killerMoves[0].to[1]
    ) {
      score += 900;
    } else if (
      killerMoves[1] &&
      move.from === killerMoves[1].from &&
      move.to[0] === killerMoves[1].to[0] &&
      move.to[1] === killerMoves[1].to[1]
    ) {
      score += 800;
    }
  }

  // 5. History Heuristic Bonus
  if (
    move.from !== "drop" &&
    historyTable[move.from[0]] &&
    historyTable[move.from[0]][move.from[1]]
  ) {
    score += historyTable[move.from[0]][move.from[1]];
  }

  // 6. Escape from Attack Priority
  if (move.from !== "drop") {
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
  if (
    move.to[0] >= 3 &&
    move.to[0] <= 5 &&
    move.to[1] >= 3 &&
    move.to[1] <= 5
  ) {
    score += 20; // Small bonus for moving towards the center
  }

  // 8. Check Priority
  if (move.isCheck) {
    score += 50;
  }

  // 9. Drop Safety Evaluation (NEW)
  if (move.from === "drop") {
    const dropSafety = evaluateDropSafety(gameState, move.type, move.to);
    score += dropSafety.safetyScore;
    
    // Additional penalty for very unsafe drops
    if (!dropSafety.isSafe && dropSafety.worstThreat >= PIECE_VALUES[move.type]) {
      score -= PIECE_VALUES[move.type] * 5; // Heavy penalty for drops that would be captured by equal/higher value pieces
    }
    
    // Log the final score for drop moves
    console.log(`AI: Drop move ${move.type} to [${move.to[0]},${move.to[1]}] scored: ${score} (base: ${score - dropSafety.safetyScore}, safety: ${dropSafety.safetyScore})`);
  }

  return score;
}

function scoreMoveEnhanced(move, gameState) {
  let score = 0;
  const { board, currentPlayer, moveHistory } = gameState;
  const opponent = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

  // 1. Promotion Priority (highest priority)
  if (move.promote) {
    score += 800; // Strong bonus for promoting a piece
  }

  // 2. Enhanced MVV-LVA for Captures with SEE
  if (move.isCapture) {
    const capturedPieceType = board[move.to[0]][move.to[1]]?.type;
    const attackingPieceType =
      move.from === "drop"
        ? move.type
        : board[move.from[0]][move.from[1]]?.type;

    if (capturedPieceType && attackingPieceType) {
      // Enhanced MVV-LVA: Most Valuable Victim - Least Valuable Attacker
      const victimValue = PIECE_VALUES[capturedPieceType];
      const attackerValue = PIECE_VALUES[attackingPieceType];
      
      // Base capture score: victim value * 10 - attacker value
      score += victimValue * 10 - attackerValue;
      
      // Additional bonus for good captures (capturing higher value pieces)
      if (victimValue > attackerValue) {
        score += 500; // Bonus for winning exchanges
      } else if (victimValue === attackerValue) {
        score += 100; // Neutral exchange
      } else {
        score -= 200; // Penalty for losing exchanges
      }
      
      // SEE (Static Exchange Evaluation) - quick evaluation of the exchange
      const seeScore = calculateSEE(board, move, currentPlayer);
      score += seeScore * 2; // Weight SEE heavily in move ordering
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
    if (
      killerMoves[0] &&
      move.from === killerMoves[0].from &&
      move.to[0] === killerMoves[0].to[0] &&
      move.to[1] === killerMoves[0].to[1]
    ) {
      score += 900;
    } else if (
      killerMoves[1] &&
      move.from === killerMoves[1].from &&
      move.to[0] === killerMoves[1].to[0] &&
      move.to[1] === killerMoves[1].to[1]
    ) {
      score += 800;
    }
  }

  // 5. History Heuristic Bonus
  if (
    move.from !== "drop" &&
    historyTable[move.from[0]] &&
    historyTable[move.from[0]][move.from[1]]
  ) {
    score += historyTable[move.from[0]][move.from[1]];
  }

  // 6. Escape from Attack Priority
  if (move.from !== "drop") {
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
  if (
    move.to[0] >= 3 &&
    move.to[0] <= 5 &&
    move.to[1] >= 3 &&
    move.to[1] <= 5
  ) {
    score += 20; // Small bonus for moving towards the center
  }

  // 8. Check Priority
  if (move.isCheck) {
    score += 50;
  }

  // 9. Drop Safety Evaluation
  if (move.from === "drop") {
    const dropSafety = evaluateDropSafety(gameState, move.type, move.to);
    score += dropSafety.safetyScore;
    
    // Additional penalty for very unsafe drops
    if (!dropSafety.isSafe && dropSafety.worstThreat >= PIECE_VALUES[move.type]) {
      score -= PIECE_VALUES[move.type] * 5; // Heavy penalty for drops that would be captured by equal/higher value pieces
    }
    
    // Log the final score for drop moves
    console.log(`AI: Drop move ${move.type} to [${move.to[0]},${move.to[1]}] scored: ${score} (base: ${score - dropSafety.safetyScore}, safety: ${dropSafety.safetyScore})`);
  }

  return score;
}

/**
 * Calculate Static Exchange Evaluation (SEE) for a move
 * This evaluates the material value of a capture sequence
 */
function calculateSEE(board, move, currentPlayer) {
  if (!move.isCapture) return 0;
  
  const [toRow, toCol] = move.to;
  let totalScore = 0;
  let currentBoard = board.map(row => [...row]);
  let currentPlayerTurn = currentPlayer;
  
  // Simulate the initial capture
  const attackingPiece = move.from === "drop" 
    ? { type: move.type, player: currentPlayer }
    : currentBoard[move.from[0]][move.from[1]];
  
  if (!attackingPiece) return 0;
  
  const victimPiece = currentBoard[toRow][toCol];
  if (!victimPiece) return 0;
  
  // Initial capture
  totalScore = PIECE_VALUES[victimPiece.type];
  currentBoard[toRow][toCol] = attackingPiece;
  if (move.from !== "drop") {
    currentBoard[move.from[0]][move.from[1]] = null;
  }
  
  // Find the next attacker to this square
  let nextAttacker = findLeastValuableAttacker(currentBoard, toRow, toCol, currentPlayerTurn === currentPlayer ? PLAYER_1 : PLAYER_2);
  
  // Continue the exchange until no more captures are possible
  let depth = 0;
  const maxDepth = 6; // Limit exchange depth to prevent infinite loops
  
  while (nextAttacker && depth < maxDepth) {
    const attackerPiece = currentBoard[nextAttacker.row][nextAttacker.col];
    if (!attackerPiece) break;
    
    // Make the capture
    currentBoard[toRow][toCol] = attackerPiece;
    currentBoard[nextAttacker.row][nextAttacker.col] = null;
    
    // Update score (negative because it's opponent's turn)
    if (depth % 2 === 0) {
      totalScore -= PIECE_VALUES[attackerPiece.type];
    } else {
      totalScore += PIECE_VALUES[attackerPiece.type];
    }
    
    // Find next attacker
    currentPlayerTurn = currentPlayerTurn === PLAYER_1 ? PLAYER_2 : PLAYER_1;
    nextAttacker = findLeastValuableAttacker(currentBoard, toRow, toCol, currentPlayerTurn);
    depth++;
  }
  
  return totalScore;
}

/**
 * Find the least valuable piece that can attack a given square
 */
function findLeastValuableAttacker(board, row, col, player) {
  let bestAttacker = null;
  let bestValue = Infinity;
  
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.player === player) {
        // Check if this piece can attack the target square
        const moves = getLegalMoves(piece, r, c, board);
        if (moves.some(([moveRow, moveCol]) => moveRow === row && moveCol === col)) {
          const pieceValue = PIECE_VALUES[piece.type];
          if (pieceValue < bestValue) {
            bestValue = pieceValue;
            bestAttacker = { row: r, col: c, piece: piece.type, value: pieceValue };
          }
        }
      }
    }
  }
  
  return bestAttacker;
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
        moves.forEach((to) => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const isCapture =
            board[to[0]][to[1]] && board[to[0]][to[1]].player !== currentPlayer;
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer,
          );
          const promotionZoneStart = currentPlayer === PLAYER_1 ? 2 : 6;
          const inPromotionZone =
            (currentPlayer === PLAYER_1 && to[0] <= promotionZoneStart) ||
            (currentPlayer === PLAYER_2 && to[0] >= promotionZoneStart);
          const wasInPromotionZone =
            (currentPlayer === PLAYER_1 && r <= promotionZoneStart) ||
            (currentPlayer === PLAYER_2 && r >= promotionZoneStart);
          const promotablePieces = [PAWN, LANCE, KNIGHT, SILVER, BISHOP, ROOK];
          const canPromote =
            promotablePieces.includes(piece.type) &&
            (inPromotionZone || wasInPromotionZone);
          const lastRank = currentPlayer === PLAYER_1 ? 0 : 8;
          const secondLastRank = currentPlayer === PLAYER_1 ? 1 : 7;
          let isPromotionMandatory = false;
          if (
            (piece.type === PAWN || piece.type === LANCE) &&
            to[0] === lastRank
          ) {
            isPromotionMandatory = true;
          }
          if (
            piece.type === KNIGHT &&
            (to[0] === lastRank || to[0] === secondLastRank)
          ) {
            isPromotionMandatory = true;
          }

          let promote = false;
          if (canPromote && !isPromotionMandatory) {
            promote = true; // AI always promotes if optional
          } else if (isPromotionMandatory) {
            promote = true; // Mandatory promotion
          }

          if (!isKingInCheck(simulatedGameState.board, currentPlayer)) {
            // Only add if the move doesn't put own king in check
            possibleMoves.push({
              from: [r, c],
              to,
              type: "move",
              isCapture,
              isCheck,
              promote,
            });
          }
        });
      }
    }
  }

  // Collect all possible drops for captured pieces
  capturedPieces[currentPlayer].forEach((capturedPiece) => {
    const legalDrops = getLegalDrops(gameState, capturedPiece.type);
    legalDrops.forEach((to) => {
      const simulatedGameState = dropPiece(gameState, capturedPiece.type, to);
      if (
        simulatedGameState !== gameState &&
        !isKingInCheck(simulatedGameState.board, currentPlayer)
      ) {
        // Evaluate drop safety before adding to possible moves
        const dropSafety = evaluateDropSafety(gameState, capturedPiece.type, to);
        
        // Only add drops that aren't extremely unsafe (immediate capture by equal/higher value piece)
        const pieceValue = PIECE_VALUES[capturedPiece.type];
        const isExtremelyUnsafe = !dropSafety.isSafe && dropSafety.worstThreat >= pieceValue;
        
        if (!isExtremelyUnsafe) {
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1,
          );
          possibleMoves.push({
            from: "drop",
            to,
            type: capturedPiece.type,
            isCapture: false,
            isPromotion: false,
            isCheck,
          });
        }
      }
    });
  });

  return possibleMoves;
}

async function quiescenceSearch(
  gameState,
  alpha,
  beta,
  depth,
  startTime,
  timeLimit,
) {
  if (Date.now() - startTime > timeLimit) {
    console.log(`Quiescence search time limit exceeded at depth ${depth}`);
    return null; // Indicate that the search was cut short
  }
  let standPat = evaluateBoard(gameState);

  if (depth >= 5) {
    // Limit quiescence search depth
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
        moves.forEach((to) => {
          const simulatedGameState = movePiece(gameState, [r, c], to);
          const targetPieceAfterMove = simulatedGameState.board[to[0]][to[1]];
          const isCapture =
            targetPieceAfterMove &&
            targetPieceAfterMove.player !== currentPlayer;
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer,
          );

          if (isCapture || isCheck) {
            possibleNoisyMoves.push({
              from: [r, c],
              to,
              type: "move",
              isCapture,
              isCheck,
            });
          }
        });
      }
    }
  }

  // Collect noisy drops (checks)
  capturedPieces[currentPlayer].forEach((capturedPiece) => {
    for (let r = 0; r < 9; r++) {
      for (let c = 0; c < 9; c++) {
        if (!board[r][c]) {
          // Only drop on empty squares
          const simulatedGameState = dropPiece(gameState, capturedPiece.type, [
            r,
            c,
          ]);
          const isCheck = isKingInCheck(
            simulatedGameState.board,
            simulatedGameState.currentPlayer,
          );
          if (isCheck) {
            possibleNoisyMoves.push({
              from: "drop",
              to: [r, c],
              type: capturedPiece.type,
              isCapture: false,
              isCheck,
            });
          }
        }
      }
    }
  });

  // Sort noisy moves for better pruning using enhanced scoring
  possibleNoisyMoves.sort(
    (a, b) => scoreMoveEnhanced(b, gameState) - scoreMoveEnhanced(a, gameState),
  );

  if (possibleNoisyMoves.length === 0) {
    return alpha;
  }

  for (const move of possibleNoisyMoves) {
    if (Date.now() - startTime > timeLimit) {
      return 0; // Abort if time limit exceeded during move iteration
    }
    await new Promise((resolve) => setTimeout(resolve, 0)); // Yield control
    let newGameState = { ...gameState, pastStates: [] }; // Deep copy relevant parts of gameState, omit pastStates
    if (move.from === "drop") {
      newGameState = dropPiece(newGameState, move.type, move.to);
    } else {
      newGameState = movePiece(newGameState, move.from, move.to);
    }

    const score = -(await quiescenceSearch(
      newGameState,
      -beta,
      -alpha,
      depth + 1,
      startTime,
      timeLimit,
    )); // Negamax: negate score from recursive call

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
 * Evaluate pawn structure for a player
 */
function evaluatePawnStructure(board, player) {
  let score = 0;
  const pawns = [];
  
  // Collect all pawns for this player
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === PAWN && piece.player === player) {
        pawns.push([r, c]);
      }
    }
  }
  
  if (pawns.length === 0) return 0;
  
  // Bonus for pawn chains
  for (let i = 0; i < pawns.length; i++) {
    for (let j = i + 1; j < pawns.length; j++) {
      const [r1, c1] = pawns[i];
      const [r2, c2] = pawns[j];
      
      // Check if pawns are adjacent horizontally or vertically
      if ((Math.abs(r1 - r2) === 1 && c1 === c2) || (Math.abs(c1 - c2) === 1 && r1 === r2)) {
        score += 15; // Bonus for connected pawns
      }
    }
  }
  
  // Bonus for advanced pawns (closer to promotion zone)
  const promotionZoneStart = player === PLAYER_1 ? 2 : 6;
  for (const [r, c] of pawns) {
    if (player === PLAYER_1 && r <= promotionZoneStart) {
      score += (promotionZoneStart - r) * 10; // Bonus for advancing towards promotion
    } else if (player === PLAYER_2 && r >= promotionZoneStart) {
      score += (r - promotionZoneStart) * 10;
    }
  }
  
  // Penalty for isolated pawns
  for (const [r, c] of pawns) {
    let isIsolated = true;
    for (let dr = -1; dr <= 1; dr++) {
      for (let dc = -1; dc <= 1; dc++) {
        if (dr === 0 && dc === 0) continue;
        const nr = r + dr;
        const nc = c + dc;
        if (nr >= 0 && nr < 9 && nc >= 0 && nc < 9) {
          const piece = board[nr][nc];
          if (piece && piece.type === PAWN && piece.player === player) {
            isIsolated = false;
            break;
          }
        }
      }
      if (!isIsolated) break;
    }
    if (isIsolated) {
      score -= 20; // Penalty for isolated pawns
    }
  }
  
  return score;
}

/**
 * Evaluate king activity in endgame situations
 */
function evaluateKingActivity(board, player, kingPos, capturedPieces) {
  if (!kingPos) return 0;
  
  // Only consider king activity in endgame (few pieces remaining)
  const totalPieces = countPieces(board) + countCapturedPieces(capturedPieces);
  if (totalPieces > 20) return 0; // Not endgame yet
  
  let score = 0;
  const [kingR, kingC] = kingPos;
  
  // Bonus for king activity in endgame
  if (totalPieces <= 10) {
    // In very late endgame, encourage king to move towards center
    const centerDistance = Math.abs(kingR - 4) + Math.abs(kingC - 4);
    score += (9 - centerDistance) * 5;
  }
  
  // Bonus for king being near opponent's side in endgame
  const opponentSide = player === PLAYER_1 ? 8 : 0;
  const distanceToOpponentSide = Math.abs(kingR - opponentSide);
  if (totalPieces <= 15) {
    score += (9 - distanceToOpponentSide) * 3;
  }
  
  return score;
}

/**
 * Evaluate connected rooks (rooks that can support each other)
 */
function evaluateConnectedRooks(board, player) {
  let score = 0;
  const rooks = [];
  
  // Collect all rooks for this player
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (piece && piece.type === ROOK && piece.player === player) {
        rooks.push([r, c]);
      }
    }
  }
  
  if (rooks.length < 2) return 0;
  
  // Check if rooks are on the same rank or file
  for (let i = 0; i < rooks.length; i++) {
    for (let j = i + 1; j < rooks.length; j++) {
      const [r1, c1] = rooks[i];
      const [r2, c2] = rooks[j];
      
      if (r1 === r2 || c1 === c2) {
        // Check if there are no pieces blocking the connection
        let isConnected = true;
        if (r1 === r2) {
          // Same rank, check if no pieces between
          const minC = Math.min(c1, c2);
          const maxC = Math.max(c1, c2);
          for (let c = minC + 1; c < maxC; c++) {
            if (board[r1][c]) {
              isConnected = false;
              break;
            }
          }
        } else {
          // Same file, check if no pieces between
          const minR = Math.min(r1, r2);
          const maxR = Math.max(r1, r2);
          for (let r = minR + 1; r < maxR; r++) {
            if (board[r][c1]) {
              isConnected = false;
              break;
            }
          }
        }
        
        if (isConnected) {
          score += 30; // Bonus for connected rooks
        }
      }
    }
  }
  
  return score;
}

/**
 * Count total pieces on the board
 */
function countPieces(board) {
  let count = 0;
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      if (board[r][c]) count++;
    }
  }
  return count;
}

/**
 * Count total captured pieces
 */
function countCapturedPieces(capturedPieces) {
  let count = 0;
  for (const player in capturedPieces) {
    count += capturedPieces[player].length;
  }
  return count;
}

/**
 * Simple endgame tablebase for basic positions
 * Returns the best move and score for known endgame positions
 */
function getEndgameTablebaseMove(board, currentPlayer, capturedPieces) {
  const totalPieces = countPieces(board) + countCapturedPieces(capturedPieces);
  
  // Only use tablebase for very simple endgames
  if (totalPieces > 6) return null;
  
  // King vs King (draw)
  if (totalPieces === 2) {
    return { move: null, score: 0, isDraw: true };
  }
  
  // King + Pawn vs King
  if (totalPieces === 3) {
    const result = evaluateKingPawnEndgame(board, currentPlayer);
    if (result) return result;
  }
  
  // King + 2 Pawns vs King
  if (totalPieces === 4) {
    const result = evaluateKingTwoPawnsEndgame(board, currentPlayer);
    if (result) return result;
  }
  
  return null;
}

/**
 * Evaluate King + Pawn vs King endgame
 */
function evaluateKingPawnEndgame(board, currentPlayer) {
  let playerKing = null;
  let opponentKing = null;
  let playerPawn = null;
  
  // Find kings and pawn
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (!piece) continue;
      
      if (piece.type === KING) {
        if (piece.player === currentPlayer) {
          playerKing = [r, c];
        } else {
          opponentKing = [r, c];
        }
      } else if (piece.type === PAWN && piece.player === currentPlayer) {
        playerPawn = [r, c];
      }
    }
  }
  
  if (!playerKing || !opponentKing || !playerPawn) return null;
  
  const [pawnR, pawnC] = playerPawn;
  const [oppKingR, oppKingC] = opponentKing;
  
  // Check if pawn can promote
  const promotionRank = currentPlayer === PLAYER_1 ? 0 : 8;
  const distanceToPromotion = Math.abs(pawnR - promotionRank);
  
  // Check if opponent king can catch the pawn
  const distanceToPawn = Math.abs(oppKingR - pawnR) + Math.abs(oppKingC - pawnC);
  
  // If pawn is close to promotion and opponent king is far, this is winning
  if (distanceToPromotion <= 2 && distanceToPawn > 3) {
    return { move: null, score: 1000, isWinning: true };
  }
  
  // If opponent king is very close to pawn, this might be a draw
  if (distanceToPawn <= 2) {
    return { move: null, score: 100, isDraw: true };
  }
  
  return null;
}

/**
 * Evaluate King + 2 Pawns vs King endgame
 */
function evaluateKingTwoPawnsEndgame(board, currentPlayer) {
  let playerKing = null;
  let opponentKing = null;
  const playerPawns = [];
  
  // Find kings and pawns
  for (let r = 0; r < 9; r++) {
    for (let c = 0; c < 9; c++) {
      const piece = board[r][c];
      if (!piece) continue;
      
      if (piece.type === KING) {
        if (piece.player === currentPlayer) {
          playerKing = [r, c];
        } else {
          opponentKing = [r, c];
        }
      } else if (piece.type === PAWN && piece.player === currentPlayer) {
        playerPawns.push([r, c]);
      }
    }
  }
  
  if (!playerKing || !opponentKing || playerPawns.length !== 2) return null;
  
  // Check if either pawn is close to promotion
  const promotionRank = currentPlayer === PLAYER_1 ? 0 : 8;
  let hasAdvancedPawn = false;
  
  for (const [pawnR, pawnC] of playerPawns) {
    const distanceToPromotion = Math.abs(pawnR - promotionRank);
    if (distanceToPromotion <= 2) {
      hasAdvancedPawn = true;
      break;
    }
  }
  
  // Two pawns with one close to promotion is usually winning
  if (hasAdvancedPawn) {
    return { move: null, score: 1500, isWinning: true };
  }
  
  return null;
}
