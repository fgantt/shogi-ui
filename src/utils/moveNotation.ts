// import { PieceType } from 'tsshogi'; // Not currently used

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
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove({ move: { pieceType: 'pawn', promote: true } }); // Default for USI-only case
    const pieceKanji = getPieceKanji(piece);
    const toKanji = convertPositionToKanji(to);
    return `${playerSymbol}${toKanji}${pieceKanji}成`;
  } else {
    // Normal move: 7g7f -> ▲7六歩
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove({ move: { pieceType: 'pawn', promote: false } }); // Default for USI-only case
    const pieceKanji = getPieceKanji(piece);
    const toKanji = convertPositionToKanji(to);
    return `${playerSymbol}${toKanji}${pieceKanji}`;
  }
}

// Convert USI move to Kifu notation using move object for accurate piece type
export function usiToKifuWithMove(usi: string, move: any, isBlack: boolean): string {
  const playerSymbol = isBlack ? '▲' : '△';
  
  if (usi.includes('*')) {
    // Drop move: P*5d -> ▲5四歩打
    const piece = usi.charAt(0);
    const position = usi.substring(2);
    const pieceKanji = getPieceKanji(piece);
    const positionKanji = convertPositionToKanji(position);
    return `${playerSymbol}${positionKanji}${pieceKanji}打`;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> ▲7六馬成 (for promoted bishop/horse)
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move); // Use actual move object
    const pieceKanji = getPieceKanji(piece);
    const toKanji = convertPositionToKanji(to);
    return `${playerSymbol}${toKanji}${pieceKanji}成`;
  } else {
    // Normal move: 7g7f -> ▲7六歩
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move); // Use actual move object
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
    // Promoted pieces
    '+P': 'と', // Promoted Pawn
    '+L': '杏', // Promoted Lance
    '+N': '圭', // Promoted Knight
    '+S': '全', // Promoted Silver
    '+B': '馬', // Horse (Promoted Bishop)
    '+R': '龍', // Dragon (Promoted Rook)
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

// Helper function to determine piece from move object
function getPieceFromMove(move: any): string {
  // Check if this is a regular move with piece type information
  if (move.move && typeof move.move === 'object' && 'pieceType' in move.move) {
    const pieceType = move.move.pieceType;
    const isPromoted = move.move.promote || false;
    
    // Convert tsshogi piece type to USI character
    const pieceChar = tsshogiPieceTypeToUsi(pieceType, isPromoted);
    return pieceChar;
  }
  
  // Fallback to pawn if we can't determine the piece
  return 'P';
}

// Helper function to convert tsshogi piece type to USI character
function tsshogiPieceTypeToUsi(pieceType: any, isPromoted: boolean = false): string {
  // Map tsshogi PieceType enum values to USI characters
  const pieceMap: { [key: string]: string } = {
    'pawn': 'P',
    'lance': 'L', 
    'knight': 'N',
    'silver': 'S',
    'gold': 'G',
    'bishop': 'B',
    'rook': 'R',
    'king': 'K',
    // Promoted pieces
    'promPawn': '+P',
    'promLance': '+L',
    'promKnight': '+N', 
    'promSilver': '+S',
    'horse': '+B', // Promoted bishop
    'dragon': '+R', // Promoted rook
  };
  
  return pieceMap[pieceType] || 'P';
}

// Convert move record to display format based on notation type
export function formatMoveForDisplay(move: any, notation: 'western' | 'kifu', isBlack: boolean): string {
  // Check if this is a regular move with USI string
  if (move.move && typeof move.move === 'object' && 'usi' in move.move) {
    const usi = move.move.usi;
    if (notation === 'western') {
      return usiToWestern(usi);
    } else {
      // For kifu notation, we need to use the move object to get the correct piece type
      return usiToKifuWithMove(usi, move, isBlack);
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