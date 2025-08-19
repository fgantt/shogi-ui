import { getInitialGameState, movePiece, dropPiece, getLegalMoves, ROWS, COLS } from './engine';
import type { GameState, Move } from '../types';

export function parseKifu(kifu: string): GameState {
  const lines = kifu.split('\n');
  const header: { [key: string]: string } = {};
  let moveLines: string[] = [];
  let inHeader = true;
  let gameState = getInitialGameState();

  // Parse header and separate move lines
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    if (inHeader) {
      if (line.length === 0) {
        continue;
      }
      if (/^\d/.test(line)) { // If the line starts with a digit, it's a move line
        inHeader = false;
        moveLines.push(line); // This line is the first move line
      } else {
        const match = line.match(/^(.*?)[\u003A\uFF1A](.*)$/);
        if (match) {
          header[match[1]] = match[2].trim();
        } else {
          // If it's not a header line and doesn't start with a digit, it's an unexpected line in header section
          // For now, we'll just log it and skip.
          console.warn("parseKifu: Skipping unexpected line in header section:", line);
        }
      }
    } else {
      moveLines.push(line);
    }
  }

  // Parse moves
  const moves: Move[] = [];
  const moveLineRegex = /^\d+\s+([^\s]+)(?:\s+\(.*\))?$/; // Matches "NUMBER MOVE_STRING (TIME_INFO)"
                                                        // Captures MOVE_STRING in group 1
  for (let i = 0; i < moveLines.length; i++) {
    const line = moveLines[i].trim();
    if (line.length === 0) {
      continue;
    }
    const match = line.match(moveLineRegex);
    if (match) {
      const moveString = match[1]; // This should be "８六歩" or "８三歩打"
      const moveNumber = parseInt(line.split(' ')[0]); // Get move number from the start of the line

      const currentPlayerForMove = (moveNumber % 2 !== 0) ? 'player1' : 'player2';
      const parsedMove = parseMove(moveString, gameState, currentPlayerForMove);
      if (parsedMove) {
        parsedMove.player = currentPlayerForMove; // Ensure player is set correctly in the parsed move

        // Apply the move to the gameState immediately
        if (parsedMove.from === 'drop') {
          gameState = dropPiece(gameState, parsedMove.piece, parsedMove.to, parsedMove.player);
        } else {
          gameState = movePiece(gameState, parsedMove.from, parsedMove.to, parsedMove.player, parsedMove.promote);
        }
        moves.push(parsedMove);
      }
    } else {
      // If a line in the move section doesn't match the expected move format,
      // it might be an unexpected line or the end of the moves.
      // For now, we'll just log it and skip.
      console.warn("parseKifu: Skipping malformed move line:", line);
    }
  }

  

   console.log("Final gameState from parseKifu:", gameState);
  return gameState;
}

export function generateKifu(gameState: GameState): string {
  const { moveHistory } = gameState;

  const header = [
    `先手：${'Player 1'}`,
    `後手：${'Player 2'}`,
    `手合割：平手`,
    `開始日時：${new Date().toLocaleString()}`,
    `終了日時：${new Date().toLocaleString()}`,
    ''
  ].join('\n');

  const moves = moveHistory.map((move, index) => {
    const moveNumber = index + 1;
    const moveString = getMoveString(move);
    const timeString = `( 0:00/00:00:00)`; // Placeholder for time
    return `${moveNumber} ${moveString} ${timeString}`;
  }).join('\n');

  return `${header}\n${moves}`;
}

function getMoveString(move: Move): string {
  const { from, to, piece, promote } = move;

  const pieceMap: { [key: string]: string } = {
    'K': '玉',
    'R': '飛',
    'B': '角',
    'G': '金',
    'S': '銀',
    'N': '桂',
    'L': '香',
    'P': '歩',
    '+R': '龍',
    '+B': '馬',
    '+S': '成銀',
    '+N': '成桂',
    '+L': '成香',
    '+P': 'と',
  };

  const fileMap: { [key: number]: string } = {
    0: '１', 1: '２', 2: '３', 3: '４', 4: '５', 5: '６', 6: '７', 7: '８', 8: '９'
  };

  const rankMap: { [key: number]: string } = {
    0: '一', 1: '二', 2: '三', 3: '四', 4: '五', 5: '六', 6: '七', 7: '八', 8: '九'
  };

  if (from === 'drop') {
    return `${fileMap[to[1]]}${rankMap[to[0]]}${pieceMap[piece]}打`;
  }

  const fromString = `${fileMap[from[1]]}${rankMap[from[0]]}`;
  const toString = `${fileMap[to[1]]}${rankMap[to[0]]}`;
  const promoteString = promote ? '成' : '';

  return `${toString}${pieceMap[piece]}${promoteString}`;
}

function parseMove(moveString: string, gameState: GameState, player: Player): Move | null {
  const moveRegex = /^([１２３４５６７８９])(一|二|三|四|五|六|七|八|九)([玉飛角金銀桂香歩龍馬成銀成桂成香と])(成|打)?$/;
  const match = moveString.match(moveRegex);

  if (!match) {
    console.log("parseMove: No match for moveString:", moveString);
    return null;
  }

  const pieceMap: { [key: string]: string } = {
    '玉': 'K', '飛': 'R', '角': 'B', '金': 'G', '銀': 'S', '桂': 'N', '香': 'L', '歩': 'P',
    '龍': '+R', '馬': '+B', '成銀': '+S', '成桂': '+N', '成香': '+L', 'と': '+P',
  };

  const fileMap: { [key: string]: number } = {
    '１': 0, '２': 1, '３': 2, '４': 3, '５': 4, '６': 5, '７': 6, '８': 7, '９': 8
  };

  const rankMap: { [key: string]: number } = {
    '一': 0, '二': 1, '三': 2, '四': 3, '五': 4, '六': 5, '七': 6, '八': 7, '九': 8
  };

  const toFileChar = match[1];
  const toRankChar = match[2];
  const pieceChar = match[3];
  const suffix = match[4]; // '成' or '打' or undefined

  const to: [number, number] = [rankMap[toRankChar], fileMap[toFileChar]];
  const piece = pieceMap[pieceChar];
  const promote = suffix === '成';
  const isDrop = suffix === '打';

  let from: [number, number] | 'drop';

  if (isDrop) {
    from = 'drop';
  } else {
    // For simple moves, 'from' is not explicitly given in the KIF string.
    // We need to find the piece on the board that could have made this move.
    // Iterate through the board to find the piece that could have made this move.
    let foundFrom: [number, number] | null = null;
    const board = gameState.board;

    // Determine the unpromoted piece type for comparison
    let unpromotedPieceType = piece;
    if (piece.startsWith('+')) {
      unpromotedPieceType = piece.substring(1);
    }

    for (let r = 0; r < ROWS; r++) {
      for (let c = 0; c < COLS; c++) {
        const pieceOnBoard = board[r][c];
        if (pieceOnBoard && pieceOnBoard.player === player) {
          // Compare piece type, considering promoted pieces
          let pieceOnBoardType = pieceOnBoard.type;
          if (pieceOnBoardType.startsWith('+')) {
            pieceOnBoardType = pieceOnBoardType.substring(1);
          }

          if (pieceOnBoardType === unpromotedPieceType) {
            const legalMoves = getLegalMoves(pieceOnBoard, r, c, board);
            for (const legalMove of legalMoves) {
              if (legalMove[0] === to[0] && legalMove[1] === to[1]) {
                foundFrom = [r, c];
                break;
              }
            }
          }
        }
        if (foundFrom) break;
      }
      if (foundFrom) break;
    }

    if (foundFrom) {
      from = foundFrom;
    } else {
      // If no valid 'from' is found, it indicates an error in KIF or game state.
      // For now, return null to indicate parsing failure.
      console.warn(`parseMove: Could not find 'from' for move: ${moveString} to ${to} by ${player}`);
      return null;
    }
  }

  return { from, to, piece, promote, player };
}
