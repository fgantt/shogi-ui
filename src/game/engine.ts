import type { Piece, Move, GameState, Player, PieceType } from '../types';

export const ROWS = 9;
export const COLS = 9;

// Piece Types
export const KING: PieceType = 'K';
export const ROOK: PieceType = 'R';
export const BISHOP: PieceType = 'B';
export const GOLD: PieceType = 'G';
export const SILVER: PieceType = 'S';
export const KNIGHT: PieceType = 'N';
export const LANCE: PieceType = 'L';
export const PAWN: PieceType = 'P';

// Promoted Piece Types
export const PROMOTED_ROOK: PieceType = '+R';
export const PROMOTED_BISHOP: PieceType = '+B';
export const PROMOTED_SILVER: PieceType = '+S';
export const PROMOTED_KNIGHT: PieceType = '+N';
export const PROMOTED_LANCE: PieceType = '+L';
export const PROMOTED_PAWN: PieceType = '+P';

// Players
export const PLAYER_1: Player = 'player1';
export const PLAYER_2: Player = 'player2';

export function getInitialGameState(): GameState {
  const board: (Piece | null)[][] = Array(ROWS).fill(null).map(() => Array(COLS).fill(null));

  const piece = (type: PieceType, player: Player): Piece => ({ type, player });

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
      player1: [],
      player2: []
    },
    moveHistory: [],
    isCheck: false,
    pastStates: [],
    kingPositions: {
      player1: [8, 4],
      player2: [0, 4],
    },
    isDraw: false,
    isCheckmate: false,
  };
}

export function getLegalMoves(piece: Piece | null, row: number, col: number, board: (Piece | null)[][]): [number, number][] {
  const moves: [number, number][] = [];
  if (!piece) {
    return moves;
  }

  const { type, player } = piece;
  const player_mult = player === PLAYER_1 ? -1 : 1;

  const canMove = (r: number, c: number): boolean => {
    if (r < 0 || r >= ROWS || c < 0 || c >= COLS) {
      return false;
    }
    const targetPiece = board[r][c];
    if (targetPiece && targetPiece.player === player) {
      return false;
    }
    return true;
  };

  const addMove = (r: number, c: number) => {
    if (canMove(r, c)) {
      moves.push([r, c]);
    }
  };

  const addSlidingMoves = (directions: [number, number][]) => {
    for (const [dr, dc] of directions) {
      let r = row + dr;
      let c = col + dc;
      while (r >= 0 && r < ROWS && c >= 0 && c < COLS) {
        const targetPiece = board[r][c];
        if (targetPiece) {
          if (targetPiece.player !== player) {
            moves.push([r, c]);
          }
          break;
        }
        moves.push([r, c]);
        r += dr;
        c += dc;
      }
    }
  };

  const getGoldMoves = () => {
      const goldMoves: [number, number][] = [
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
      const knightMoves: [number, number][] = [
        [row + player_mult * 2, col + 1],
        [row + player_mult * 2, col - 1],
      ];
      knightMoves.forEach(([r, c]) => addMove(r, c));
      break;
    case SILVER:
      const silverMoves: [number, number][] = [
        [row + player_mult, col],
        [row + player_mult, col + 1],
        [row + player_mult, col - 1],
        [row - player_mult, col + 1],
        [row - player_mult, col - 1],
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
      const kingMoves: [number, number][] = [
        [row - 1, col - 1], [row - 1, col], [row - 1, col + 1],
        [row,     col - 1],                 [row,     col + 1],
        [row + 1, col - 1], [row + 1, col], [row + 1, col + 1],
      ];
      kingMoves.forEach(([r, c]) => addMove(r, c));
      break;
    case PROMOTED_BISHOP:
        addSlidingMoves([[-1, -1], [-1, 1], [1, -1], [1, 1]]);
        addMove(row + player_mult * -1, col + player_mult * 0); // King moves
        addMove(row + player_mult * 0, col + player_mult * -1);
        addMove(row + player_mult * 0, col + player_mult * 1);
        addMove(row + player_mult * 1, col + player_mult * 0);
        break;
    case PROMOTED_ROOK:
        addSlidingMoves([[-1, 0], [1, 0], [0, -1], [0, 1]]);
        addMove(row + player_mult * -1, col + player_mult * -1); // King moves
        addMove(row + player_mult * -1, col + player_mult * 0);
        addMove(row + player_mult * -1, col + player_mult * 1);
        addMove(row + player_mult * 0, col + player_mult * -1);
        addMove(row + player_mult * 0, col + player_mult * 1);
        addMove(row + player_mult * 1, col + player_mult * -1);
        addMove(row + player_mult * 1, col + player_mult * 0);
        addMove(row + player_mult * 1, col + player_mult * 1);
        break;
  }

  return moves;
}

export function movePiece(gameState: GameState, from: [number, number], to: [number, number], playerName: string, promoteOverride: boolean | null = null): GameState {
  const { board, currentPlayer } = gameState;
  const [fromRow, fromCol] = from;
  const [toRow] = to;

  const piece = board[fromRow][fromCol];

  if (!piece) {
    
    return gameState;
  }

  if (promoteOverride !== null) {
    let pieceToMove: Piece = { ...piece };
    if (promoteOverride && !pieceToMove.type.startsWith('+')) {
      pieceToMove.type = `+${pieceToMove.type}` as PieceType;
    }
    const simulatedGameState = completeMove(gameState, from, to, promoteOverride, playerName);
    if (isKingInCheck(simulatedGameState.board, currentPlayer)) {
      
      return gameState;
    }
    return simulatedGameState;
  }

  const promotionZoneStart = currentPlayer === PLAYER_1 ? 2 : 6;
  const inPromotionZone = (currentPlayer === PLAYER_1 && toRow <= promotionZoneStart) || (currentPlayer === PLAYER_2 && toRow >= promotionZoneStart);
  const wasInPromotionZone = (currentPlayer === PLAYER_1 && fromRow <= promotionZoneStart) || (currentPlayer === PLAYER_2 && fromRow >= promotionZoneStart);

  const promotablePieces: PieceType[] = [PAWN, LANCE, KNIGHT, SILVER, BISHOP, ROOK];
  const canPromote = promotablePieces.includes(piece.type) && (inPromotionZone || wasInPromotionZone);

  const lastRank = currentPlayer === PLAYER_1 ? 0 : 8;
  const secondLastRank = currentPlayer === PLAYER_1 ? 1 : 7;
  let isPromotionMandatory = false;
  if ((piece.type === PAWN || piece.type === LANCE) && toRow === lastRank) {
      isPromotionMandatory = true;
  }
  if (piece.type === KNIGHT && (toRow === lastRank || toRow === secondLastRank)) {
      isPromotionMandatory = true;
  }

  let pieceToMove: Piece = { ...piece };

  if (canPromote && !isPromotionMandatory) {
    return {
      ...gameState,
      promotionPending: { from, to, piece },
    };
  } else if (isPromotionMandatory && !pieceToMove.type.startsWith('+')) {
    pieceToMove.type = `+${pieceToMove.type}` as PieceType;
  }

  const simulatedGameState = completeMove(gameState, from, to, isPromotionMandatory, playerName);

  if (isKingInCheck(simulatedGameState.board, currentPlayer)) {
    
    return gameState;
  }

  return simulatedGameState;
}

export function completeMove(gameState: GameState, from: [number, number], to: [number, number], promote: boolean, playerName: string): GameState {
    const { board, currentPlayer, capturedPieces, moveHistory, pastStates } = gameState;
    const nextPlayer: Player = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
    const newBoard: (Piece | null)[][] = board.map(row => row.map(cell => cell ? { ...cell } : null));

    const [fromRow, fromCol] = from;
    const [toRow, toCol] = to;

    const pieceToMove = board[fromRow][fromCol];
    if (!pieceToMove) return gameState;

    let piece: Piece = { ...pieceToMove };
    if (promote && !piece.type.startsWith('+')) {
        piece.type = `+${piece.type}` as PieceType;
    }

    const capturedPiece = newBoard[toRow][toCol];
    const newCapturedPieces = {
        ...capturedPieces,
        [currentPlayer]: [...capturedPieces[currentPlayer]]
    };

    if (capturedPiece) {
        let originalType: PieceType = capturedPiece.type;
        if (originalType.startsWith('+')) {
            originalType = originalType.substring(1) as PieceType;
        }
        newCapturedPieces[gameState.currentPlayer].push({ type: originalType, player: capturedPiece.player });
    }

    newBoard[toRow][toCol] = piece;
    newBoard[fromRow][fromCol] = null;

    const isCheckAfterMove = isKingInCheck(newBoard, nextPlayer);
    let capturedValue: string | null = null;
    if (capturedPiece) {
        capturedValue = capturedPiece.type;
        if (isCheckAfterMove) {
            capturedValue += ' / check';
        }
    } else if (isCheckAfterMove) {
        capturedValue = 'check';
    }

    const newMoveHistory: Move[] = [...moveHistory, { from, to, piece: piece.type, promote, player: currentPlayer, captured: capturedValue, timestamp: new Date().toLocaleTimeString('en-US', { hour12: false }) }];

    const updatedGameState: GameState = {
        ...gameState,
        board: newBoard,
        capturedPieces: newCapturedPieces,
        currentPlayer: nextPlayer,
        moveHistory: newMoveHistory,
        promotionPending: null,
        pastStates: [...pastStates, gameState],
        isCheck: isKingInCheck(newBoard, nextPlayer),
        kingPositions: {
          ...gameState.kingPositions,
          [currentPlayer]: (piece.type === KING) ? [toRow, toCol] : gameState.kingPositions[currentPlayer]
        },
        isCheckmate: false,
        isDraw: false,
    };

    if (capturedPiece && capturedPiece.type === KING) {
        updatedGameState.isCheckmate = true;
    } else {
        updatedGameState.isCheckmate = isCheckmate(updatedGameState);
    }

    if (checkSennichite(updatedGameState)) {
      updatedGameState.isDraw = true;
    }

    const playerNum = updatedGameState.currentPlayer === PLAYER_1 ? 2 : 1;
    const color = updatedGameState.currentPlayer === PLAYER_1 ? "White" : "Black";
    const shogiNotation = getShogiNotation({ from, to, piece: piece.type, promote }, piece);

    if (playerName !== undefined && playerName)
      console.log(`Player ${playerNum} (${color}) - ${playerName} moved ${shogiNotation}`);

    return updatedGameState;
}

export function checkSennichite(gameState: GameState): boolean {
  const { pastStates } = gameState;
  const currentHash = generateSennichiteHash(gameState);

  let count = 0;
  for (const state of pastStates) {
    if (generateSennichiteHash(state) === currentHash) {
      count++;
    }
  }
  return count + 1 >= 4;
}

export function dropPiece(gameState: GameState, pieceType: PieceType, to: [number, number], playerName: string): GameState {
  const { board, currentPlayer, capturedPieces, pastStates } = gameState;
  const [toRow, toCol] = to;

  if (board[toRow][toCol]) {
    
    return gameState;
  }

  const capturedPieceIndex = capturedPieces[currentPlayer].findIndex(p => p.type === pieceType);
  if (capturedPieceIndex === -1) {
    
    return gameState;
  }

  if (pieceType === PAWN) {
    for (let r = 0; r < ROWS; r++) {
      const piece = board[r][toCol];
      if (piece && piece.type === PAWN && piece.player === currentPlayer) {
        return gameState;
      }
    }
  }

  const player_mult = currentPlayer === PLAYER_1 ? -1 : 1;
  if ((pieceType === PAWN || pieceType === LANCE) && (toRow + player_mult < 0 || toRow + player_mult >= ROWS)) {
      return gameState;
  }
  if (pieceType === KNIGHT && (toRow + (player_mult * 2) < 0 || toRow + (player_mult * 2) >= ROWS)) {
      return gameState;
  }

  const tempBoard = board.map(row => [...row]);
  tempBoard[toRow][toCol] = { type: pieceType, player: currentPlayer };

  if (isKingInCheck(tempBoard, currentPlayer)) {
    
    return gameState;
  }

  const newBoard = tempBoard;

  const newCapturedPieces = { ...capturedPieces };
  newCapturedPieces[currentPlayer] = [...capturedPieces[currentPlayer]];
  newCapturedPieces[currentPlayer].splice(capturedPieceIndex, 1);

  const nextPlayer: Player = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  const isCheckAfterDrop = isKingInCheck(newBoard, nextPlayer);

  const newMoveHistory: any[] = [...gameState.moveHistory, { from: 'drop', to, piece: pieceType, player: currentPlayer, captured: isCheckAfterDrop ? 'check' : null, timestamp: new Date().toLocaleTimeString('en-US', { hour12: false }) }];

  const finalGameState: GameState = {
    ...gameState,
    board: newBoard,
    capturedPieces: newCapturedPieces,
    currentPlayer: nextPlayer,
    moveHistory: newMoveHistory,
    pastStates: [...pastStates, gameState],
    isCheck: isCheckAfterDrop,
    kingPositions: {
      ...gameState.kingPositions,
      [currentPlayer]: (pieceType === KING) ? [toRow, toCol] : gameState.kingPositions[currentPlayer]
    },
    isCheckmate: false,
    isDraw: false,
  };

  finalGameState.isCheckmate = isCheckmate(finalGameState);

  if (checkSennichite(finalGameState)) {
    finalGameState.isDraw = true;
  }

  const playerNum = finalGameState.currentPlayer === PLAYER_1 ? 2 : 1;
  const color = finalGameState.currentPlayer === PLAYER_1 ? "White" : "Black";
  const shogiNotation = getShogiNotation({ from: 'drop', to, piece: pieceType, promote: false }, { type: pieceType, player: currentPlayer });

  if (playerName !== undefined && playerName)
  console.log(`Player ${playerNum} (${color}) - ${playerName} moved ${shogiNotation}`);

  return finalGameState;
}

export function getCheckingPiece(board: (Piece | null)[][], player: Player): [number, number] | null {
  const opponent: Player = player === PLAYER_1 ? PLAYER_2 : PLAYER_1;
  let kingPosition: [number, number] | null = null;

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
    return null;
  }

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const piece = board[r][c];
      if (piece && piece.player === opponent) {
        const moves = getLegalMoves(piece, r, c, board);
        for (const move of moves) {
          if (move[0] === kingPosition[0] && move[1] === kingPosition[1]) {
            return [r, c];
          }
        }
      }
    }
  }

  return null;
}

export function isKingInCheck(board: (Piece | null)[][], player: Player): boolean {
  return getCheckingPiece(board, player) !== null;
}

export function getLegalDrops(gameState: GameState, pieceType: PieceType): [number, number][] {
  const { board, currentPlayer } = gameState;
  const legalDrops: [number, number][] = [];

  const hasPieceToDrop = gameState.capturedPieces[currentPlayer].some(p => p.type === pieceType);
  if (!hasPieceToDrop) {
    return legalDrops;
  }

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      if (board[r][c] !== null) {
        continue;
      }

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

      const player_mult = currentPlayer === PLAYER_1 ? -1 : 1;
      if ((pieceType === PAWN || pieceType === LANCE) && (r + player_mult < 0 || r + player_mult >= ROWS)) {
        continue;
      }
      if (pieceType === KNIGHT && (r + (player_mult * 2) < 0 || r + (player_mult * 2) >= ROWS)) {
        continue;
      }

      if (pieceType === PAWN) {
        const simulatedBoard = board.map(row => [...row]);
        simulatedBoard[r][c] = { type: pieceType, player: currentPlayer };
        const opponent: Player = currentPlayer === PLAYER_1 ? PLAYER_2 : PLAYER_1;

        const originalPieceAtDrop = simulatedBoard[r][c];
        simulatedBoard[r][c] = null;
        const kingStillInCheckWithoutPawn = isKingInCheck(simulatedBoard, opponent);
        simulatedBoard[r][c] = originalPieceAtDrop;

        if (isKingInCheck(simulatedBoard, opponent) && isCheckmate({ ...gameState, board: simulatedBoard, currentPlayer: opponent }) && !kingStillInCheckWithoutPawn) {
          continue;
        }
      }

      legalDrops.push([r, c]);
    }
  }
  return legalDrops;
}

export function isCheckmate(gameState: GameState): boolean {
    const { board, currentPlayer, capturedPieces } = gameState;

    if (!isKingInCheck(board, currentPlayer)) {
        return false;
    }

    for (let r = 0; r < ROWS; r++) {
        for (let c = 0; c < COLS; c++) {
            const piece = board[r][c];
            if (piece && piece.player === currentPlayer) {
                const moves = getLegalMoves(piece, r, c, board);
                for (const move of moves) {
                    const tempBoard = board.map(row => row.map(cell => cell ? { ...cell } : null));
                    const pieceToMove = tempBoard[r][c];
                    if (!pieceToMove) continue;
                    tempBoard[move[0]][move[1]] = pieceToMove;
                    tempBoard[r][c] = null;

                    let tempKingPositions = { ...gameState.kingPositions };
                    if (pieceToMove.type === KING) {
                        tempKingPositions[currentPlayer] = [move[0], move[1]];
                    }
                    
                    if (!isKingInCheck(tempBoard, currentPlayer)) {
                        return false;
                    }
                }
            }
        }
    }

    for (const captured of capturedPieces[currentPlayer]) {
        const possibleDropSquares = getLegalDrops(gameState, captured.type);
        for (const dropSquare of possibleDropSquares) {
            const tempBoard = board.map(row => [...row]);
            tempBoard[dropSquare[0]][dropSquare[1]] = { type: captured.type, player: currentPlayer };
            
            if (!isKingInCheck(tempBoard, currentPlayer)) {
                return false;
            }
        }
    }

    return true;
}

export function getAttackedSquares(board: (Piece | null)[][], player: Player): Set<string> {
  const attackedSquares = new Set<string>();

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

export function generateStateHash(gameState: GameState): string {
  const { board, currentPlayer, capturedPieces, moveHistory } = gameState;

  let fen = '';

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

  fen += currentPlayer === PLAYER_1 ? ' b ' : ' w ';
  let capturedString = '-';
  const p1Captured = capturedPieces.player1.map(p => p.type).sort().join('');
  const p2Captured = capturedPieces.player2.map(p => p.type).sort().join('');
  if (p1Captured.length > 0) {
      capturedString = p1Captured.toUpperCase();
  }
  if (p2Captured.length > 0) {
      capturedString += p2Captured.toLowerCase();
  }
  fen += capturedString + ' ';

  fen += (moveHistory.length + 1);

  return fen;
}

export function generateSennichiteHash(gameState: GameState): string {
  const { board, currentPlayer, capturedPieces } = gameState;

  let fen = '';

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

  fen += currentPlayer === PLAYER_1 ? ' b ' : ' w ';
  let capturedString = '-';
  const p1Captured = capturedPieces.player1.map(p => p.type).sort().join('');
  const p2Captured = capturedPieces.player2.map(p => p.type).sort().join('');
  if (p1Captured.length > 0) {
      capturedString = p1Captured.toUpperCase();
  }
  if (p2Captured.length > 0) {
      capturedString += p2Captured.toLowerCase();
  }
  fen += capturedString;

  return fen;
}

function getShogiNotation(move: Partial<Move>, piece: Piece): string {
  const pieceNotation = piece.type.startsWith('+') ? `+${piece.type.charAt(1)}` : piece.type.charAt(0);

  if (move.from === 'drop') {
    const file = 9 - move.to[1];
    const rank = String.fromCharCode('a'.charCodeAt(0) + move.to[0]);
    return `${pieceNotation}*${file}${rank}`;
  }

  const toFile = 9 - move.to[1];
  const toRank = String.fromCharCode('a'.charCodeAt(0) + move.to[0]);
  const isCapture = move.captured !== undefined && move.captured !== null;
  const promotion = move.promote ? '+' : '';

  return `${pieceNotation}${isCapture ? 'x' : '-'}${toFile}${toRank}${promotion}`;
}
