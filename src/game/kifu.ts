import { getInitialGameState, movePiece, dropPiece, getLegalMoves, ROWS, COLS } from './engine';
import type { GameState, Move, Piece as PieceType, Player, Coords } from '../types';

const pieceMap: { [key: string]: string } = {
  'K': '玉', 'R': '飛', 'B': '角', 'G': '金', 'S': '銀', 'N': '桂', 'L': '香', 'P': '歩',
  '+R': '龍', '+B': '馬', '+S': '成銀', '+N': '成桂', '+L': '成香', '+P': 'と',
  '玉': 'K', '飛': 'R', '角': 'B', '金': 'G', '銀': 'S', '桂': 'N', '香': 'L', '歩': 'P',
  '龍': '+R', '馬': '+B', '成銀': '+S', '成桂': '+N', '成香': '+L', 'と': '+P',
};

const fileMap: { [key: string]: number } = { '１': 1, '２': 2, '３': 3, '４': 4, '５': 5, '６': 6, '７': 7, '８': 8, '９': 9 };
const rankMap: { [key: string]: number } = { '一': 1, '二': 2, '三': 3, '四': 4, '五': 5, '六': 6, '七': 7, '八': 8, '九': 9 };
const fileMapReverse: { [key: number]: string } = { 1: '１', 2: '２', 3: '３', 4: '４', 5: '５', 6: '６', 7: '７', 8: '８', 9: '９' };
const rankMapReverse: { [key: number]: string } = { 1: '一', 2: '二', 3: '三', 4: '四', 5: '五', 6: '六', 7: '七', 8: '八', 9: '九' };


export function parseKifu(kifu: string): GameState {
  const lines = kifu.split('\n');
  let gameState = getInitialGameState();
  let lastMove: Move | null = null;

  for (const line of lines) {
    const trimmedLine = line.trim();
    if (trimmedLine.length === 0 || !/^\d/.test(trimmedLine)) {
      continue;
    }

    const moveNumberMatch = trimmedLine.match(/^(\d+)/);
    if (!moveNumberMatch) continue;

    const moveNumber = parseInt(moveNumberMatch[1], 10);
    const player: Player = (moveNumber % 2 !== 0) ? 'player1' : 'player2';

    const moveStringWithParens = trimmedLine.substring(moveNumberMatch[0].length).trim();
    const moveString = moveStringWithParens.replace(/^[▲△]\s*/, '').split('(')[0].trim();

    const move = parseMove(moveString, gameState, lastMove, player);

    if (move) {
      let newGameState: GameState;
      if (move.from === 'drop') {
        newGameState = dropPiece(gameState, move.piece, move.to as Coords, player);
      } else {
        newGameState = movePiece(gameState, move.from as Coords, move.to as Coords, player, move.promote);
      }
      
      if (newGameState !== gameState) {
          gameState = newGameState;
          lastMove = move;
      } else {
          console.warn("Kifu parsing: Illegal move skipped", moveStringWithParens);
      }
    } else {
        console.warn("Kifu parsing: Could not parse move", moveStringWithParens);
    }
  }

  return gameState;
}

export function getMoveString(move: Move, gameState: GameState, lastMove: Move | null): string {
  const { from, to, piece, promote } = move;
  const toFile = fileMapReverse[9 - to[1]];
  const toRank = rankMapReverse[to[0] + 1];

  let moveStr = '';

  if (lastMove && to[0] === lastMove.to[0] && to[1] === lastMove.to[1]) {
    moveStr += '同\u3000';
  } else {
    moveStr += `${toFile}${toRank}`;
  }
  
  moveStr += pieceMap[piece];

  if (from === 'drop') {
    moveStr += '打';
    return moveStr;
  }

  if (promote) {
    moveStr += '成';
  }

  // Disambiguation logic
  const board = gameState.board;
  const otherPieces: Coords[] = [];
  const unpromotedPiece = piece.startsWith('+') ? piece.substring(1) : piece;

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const boardPiece = board[r][c];
      if (boardPiece && boardPiece.player === move.player && (boardPiece.type === unpromotedPiece || boardPiece.type === piece)) {
        const legalMoves = getLegalMoves(boardPiece, r, c, board);
        if (legalMoves.some(m => m[0] === to[0] && m[1] === to[1])) {
          otherPieces.push([r, c]);
        }
      }
    }
  }

  if (otherPieces.length > 0) {
    const fromRow = from[0];
    const fromCol = from[1];
    const toRow = to[0];
    const toCol = to[1];

    const canMoveStraight = piece === 'R' || piece === 'L' || piece === '+R';

    if (fromCol === toCol && canMoveStraight) {
      moveStr += '直';
    } else if (fromRow > toRow) {
      moveStr += '上';
    } else if (fromRow < toRow) {
      moveStr += '引';
    } else if (fromCol !== toCol) {
      moveStr += '寄';
    }
  }

  return moveStr;
}

function parseMove(moveString: string, gameState: GameState, lastMove: Move | null, player: Player): Move | null {
    let match;
    let isSameAsLast = false;
    let fileChar, rankChar, pieceChar, actionChar;

    if (moveString.startsWith('同')) {
        isSameAsLast = true;
        const sameMoveRegex = /同\s*?(成銀|成桂|成香|龍|馬|玉|飛|角|金|銀|桂|香|歩|と)(成|不成|打|引|寄|上|右|左|直)?/;
        match = moveString.match(sameMoveRegex);
        if (!match) return null;
        pieceChar = match[1];
        actionChar = match[2];
    } else {
        const moveRegex = /([１-９])([一二三四五六七八九])(成銀|成桂|成香|龍|馬|玉|飛|角|金|銀|桂|香|歩|と)(成|不成|打|引|寄|上|右|左|直)?/;
        match = moveString.match(moveRegex);
        if (!match) return null;
        fileChar = match[1];
        rankChar = match[2];
        pieceChar = match[3];
        actionChar = match[4];
    }

  const to: Coords = isSameAsLast && lastMove ? lastMove.to as Coords : [rankMap[rankChar] - 1, 9-fileMap[fileChar!]];
  const piece = pieceMap[pieceChar];
  const isDrop = actionChar === '打';
  const promote = actionChar === '成';

  if (isDrop) {
    return { from: 'drop', to, piece, player, timestamp: new Date().toISOString() };
  }

  // Find the 'from' position
  const possibleSources: Coords[] = [];
  const board = gameState.board;
  const unpromotedPiece = piece.startsWith('+') ? piece.substring(1) : piece;

  for (let r = 0; r < ROWS; r++) {
    for (let c = 0; c < COLS; c++) {
      const boardPiece = board[r][c];
      if (boardPiece && boardPiece.player === player && (boardPiece.type === unpromotedPiece || boardPiece.type === piece)) {
        const legalMoves = getLegalMoves(boardPiece, r, c, board);
        if (legalMoves.some(move => move[0] === to[0] && move[1] === to[1])) {
          possibleSources.push([r, c]);
        }
      }
    }
  }

  if (possibleSources.length === 0) {
    return null; // Should not happen in a valid kifu
  }

  if (possibleSources.length === 1) {
    return { from: possibleSources[0], to, piece, player, promote, timestamp: new Date().toISOString() };
  }

  // Handle ambiguities
  let from: Coords | null = null;
  if (actionChar) {
    switch (actionChar) {
      case '引': // Move backward
        from = possibleSources.find(source => source[0] < to[0]) || null;
        break;
      case '寄': // Move sideways
        from = possibleSources.find(source => source[0] === to[0]) || null;
        break;
      case '上': // Move forward
        from = possibleSources.find(source => source[0] > to[0]) || null;
        break;
      case '直': // Straight forward (for rook, bishop, etc.)
        from = possibleSources.find(source => source[1] === to[1]) || null;
        break;
      case '右':
        from = possibleSources.sort((a, b) => b[1] - a[1])[0];
        break;
      case '左':
        from = possibleSources.sort((a, b) => a[1] - b[1])[0];
        break;
    }
  }

  if (from) {
    return { from, to, piece, player, promote, timestamp: new Date().toISOString() };
  }
  
  return { from: possibleSources[0], to, piece, player, promote, timestamp: new Date().toISOString() };
}

export function generateKifu(gameState: GameState): string {
  const { moveHistory } = gameState;
  let kifu = '';
  const header = [
    `先手：Player 1`,
    `後手：Player 2`,
    `手合割：平手`,
    `手数----指手---------消費時間--`,
  ].join('\n');
  kifu += header + '\n';

  let lastMove: Move | null = null;
  moveHistory.forEach((move, index) => {
    const moveNumber = index + 1;
    const playerChar = move.player === 'player1' ? '▲' : '△';
    const moveString = getMoveString(move, gameState, lastMove);
    kifu += `${moveNumber} ${playerChar}${moveString}\n`;
    lastMove = move;
  });

  return kifu;
}

export function getKifuTooltipText(kifuMoveString: string): string {
  let player = '';
  let moveString = kifuMoveString;

  if (kifuMoveString.startsWith('▲')) {
    player = 'Black';
    moveString = kifuMoveString.substring(1);
  } else if (kifuMoveString.startsWith('△')) {
    player = 'White';
    moveString = kifuMoveString.substring(1);
  }

  let piece = '';
  let toSquare = '';
  let action = '';
  let promoted = false;
  let dropped = false;
  let captured = false;

  // Handle '同' (same square capture)
  if (moveString.startsWith('同')) {
    captured = true;
    toSquare = 'the same square';
    moveString = moveString.substring(1).trim();
  }

  // Extract destination coordinates
  const coordMatch = moveString.match(/^([１-９])([一二三四五六七八九])/);
  if (coordMatch) {
    const file = fileMap[coordMatch[1]];
    const rank = rankMap[coordMatch[2]];
    toSquare = `the ${file}${rank} square`;
    moveString = moveString.substring(coordMatch[0].length).trim();
  }

  // Extract piece
  for (const kifuPiece in pieceMap) {
    if (moveString.startsWith(kifuPiece)) {
      piece = pieceMap[kifuPiece];
      moveString = moveString.substring(kifuPiece.length).trim();
      break;
    }
  }

  // Extract action/promotion
  if (moveString.includes('成')) {
    promoted = true;
    action = 'promoted it';
  } else if (moveString.includes('打')) {
    dropped = true;
    action = 'dropped it';
  } else if (moveString.includes('不成')) {
    action = 'did not promote it';
  }

  let sentence = `${player} moved ${piece}`;

  if (dropped) {
    sentence = `${player} dropped ${piece}`;
  }

  if (toSquare) {
    sentence += ` to ${toSquare}`;
  }

  if (captured) {
    sentence += ` and captured a piece`;
  }

  if (promoted) {
    sentence += ` and ${action}`;
  } else if (action) {
    sentence += ` and ${action}`;
  }

  sentence += '.';

  return sentence;
}

export function getWesternTooltipText(westernMoveString: string): string {
  let piece = '';
  let toSquare = '';
  let action = '';
  let promoted = false;
  let dropped = false;
  let captured = false;

  // Regex to parse Western notation
  const moveRegex = /^([KRGBNSLP])([x*])?([1-9][a-i])(\+)?$/;
  const match = westernMoveString.match(moveRegex);

  if (match) {
    piece = match[1];
    const captureOrDrop = match[2];
    toSquare = match[3];
    promoted = !!match[4];

    if (captureOrDrop === 'x') {
      captured = true;
    } else if (captureOrDrop === '*') {
      dropped = true;
    }

    if (promoted) {
      action = ' and promoted';
    } else if (dropped) {
      action = ' and dropped';
    }

    let sentence = `Moved ${piece} to ${toSquare}`;

    if (captured) {
      sentence += ' and captured a piece';
    }

    sentence += `${action}.`;

    return sentence;
  }

  return `Invalid Western move: ${westernMoveString}`;
}
