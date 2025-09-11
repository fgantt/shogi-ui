import { PieceType } from 'tsshogi';

// Convert USI move to Western notation
export function usiToWestern(usi: string): string {
  if (usi.includes('*')) {
    // Drop move: P*5d -> P*5d
    return usi;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> 7g-7f+
    const from = usi.substring(0, 2);
    const to = usi.substring(2, 4);
    return `${from}-${to}+`;
  } else {
    // Normal move: 7g7f -> 7g-7f
    const from = usi.substring(0, 2);
    const to = usi.substring(2, 4);
    return `${from}-${to}`;
  }
}

// Convert USI move to Kifu notation
export function usiToKifu(usi: string, isBlack: boolean): string {
  const playerSymbol = isBlack ? '▲' : '△';
  
  if (usi.includes('*')) {
    // Drop move: P*5d -> ▲5四歩打
    const piece = usi.charAt(0);
    const position = usi.substring(2);
    const pieceKanji = getPieceKanji(piece);
    const positionKanji = convertPositionToKanji(position);
    return `${playerSymbol}${positionKanji}${pieceKanji}打`;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> ▲7六歩成
    const from = usi.substring(0, 2);
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(usi); // This would need to be determined from context
    const pieceKanji = getPieceKanji(piece);
    const toKanji = convertPositionToKanji(to);
    return `${playerSymbol}${toKanji}${pieceKanji}成`;
  } else {
    // Normal move: 7g7f -> ▲7六歩
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(usi); // This would need to be determined from context
    const pieceKanji = getPieceKanji(piece);
    const toKanji = convertPositionToKanji(to);
    return `${playerSymbol}${toKanji}${pieceKanji}`;
  }
}

// Helper function to get piece kanji from USI piece character
function getPieceKanji(piece: string): string {
  const pieceMap: { [key: string]: string } = {
    'P': '歩',
    'L': '香',
    'N': '桂',
    'S': '銀',
    'G': '金',
    'B': '角',
    'R': '飛',
    'K': '王',
  };
  return pieceMap[piece] || piece;
}

// Helper function to convert position to kanji (e.g., "5d" -> "5四")
function convertPositionToKanji(position: string): string {
  const file = position.charAt(0);
  const rank = position.charAt(1);
  
  const rankMap: { [key: string]: string } = {
    'a': '一', 'b': '二', 'c': '三', 'd': '四', 'e': '五',
    'f': '六', 'g': '七', 'h': '八', 'i': '九'
  };
  
  return `${file}${rankMap[rank] || rank}`;
}

// Helper function to determine piece from move (simplified - would need game context)
function getPieceFromMove(usi: string): string {
  // This is a simplified version - in reality, we'd need the game state
  // to determine which piece is moving. For now, we'll use a default.
  return 'P'; // Default to pawn
}

// Convert move record to display format based on notation type
export function formatMoveForDisplay(move: any, notation: 'western' | 'kifu', isBlack: boolean): string {
  // Check if this is a regular move with USI string
  if (move.move && typeof move.move === 'object' && 'usi' in move.move) {
    const usi = move.move.usi;
    if (notation === 'western') {
      return usiToWestern(usi);
    } else {
      return usiToKifu(usi, isBlack);
    }
  } 
  // Check if this is a special move
  else if (move.move && typeof move.move === 'object' && 'type' in move.move) {
    // For special moves, return the display text as is
    return move.displayText || '';
  }
  // Fallback to display text
  else {
    return move.displayText || '';
  }
}