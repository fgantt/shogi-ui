// import { PieceType } from 'tsshogi'; // Not currently used

// Convert USI move to English notation (legacy function - use usiToWesternWithMove for accurate piece types)
export function usiToWestern(usi: string): string {
  if (usi.includes('*')) {
    // Drop move: P*5d -> P*5d (already in correct format)
    return usi;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> P-7f+ (need piece type and destination only)
    const to = usi.substring(2, 4);
    // For now, default to P (pawn) - this should be improved with actual piece type
    return `P-${to}+`;
  } else {
    // Normal move: 7g7f -> P-7f (need piece type and destination only)
    const to = usi.substring(2, 4);
    // For now, default to P (pawn) - this should be improved with actual piece type
    return `P-${to}`;
  }
}

// Convert USI move to English notation using move object for accurate piece type
export function usiToWesternWithMove(usi: string, move: any): string {
  if (usi.includes('*')) {
    // Drop move: P*5d -> P*5d (already in correct format)
    return usi;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> P-7f+ (move results in promotion)
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move);
    // For moves that result in promotion, add + at the end
    return `${piece}-${to}+`;
  } else {
    // Normal move: 7g7f -> P-7f or +R-2b (if piece is already promoted)
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move);
    return `${piece}-${to}`;
  }
}

// Convert USI move to USI notation (already in correct format)
export function usiToUsi(usi: string): string {
  // USI notation is already in the correct format
  return usi;
}

// Convert USI move to CSA notation using move object for accurate piece type
export function usiToCsaWithMove(usi: string, move: any, isBlack: boolean): string {
  const player = isBlack ? '+' : '-';
  
  if (usi.includes('*')) {
    // Drop move: P*5d -> +0055FU
    const piece = usi.charAt(0);
    const position = usi.substring(2);
    const csaPiece = usiPieceToCsa(piece);
    const csaPosition = convertPositionToCsa(position);
    return `${player}00${csaPosition}${csaPiece}`;
  } else if (usi.includes('+')) {
    // Promotion move: 7g7f+ -> +7776TO (pawn promotes to TO, not FU+)
    const from = usi.substring(0, 2);
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move);
    // For promotion, use the promoted piece code directly
    const csaPiece = usiPieceToCsaPromoted(piece);
    const csaFrom = convertPositionToCsa(from);
    const csaTo = convertPositionToCsa(to);
    return `${player}${csaFrom}${csaTo}${csaPiece}`;
  } else {
    // Normal move: 7g7f -> +7776FU
    const from = usi.substring(0, 2);
    const to = usi.substring(2, 4);
    const piece = getPieceFromMove(move);
    const csaPiece = usiPieceToCsa(piece);
    const csaFrom = convertPositionToCsa(from);
    const csaTo = convertPositionToCsa(to);
    return `${player}${csaFrom}${csaTo}${csaPiece}`;
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
    
    // Convert tsshogi piece type to USI character
    const pieceChar = tsshogiPieceTypeToUsi(pieceType);
    return pieceChar;
  }
  
  // Fallback to pawn if we can't determine the piece
  return 'P';
}

// Helper function to convert tsshogi piece type to USI character
function tsshogiPieceTypeToUsi(pieceType: any): string {
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
    // Promoted pieces (already promoted pieces get + prefix)
    'promPawn': '+P',
    'promLance': '+L',
    'promKnight': '+N', 
    'promSilver': '+S',
    'horse': '+B', // Promoted bishop (Horse)
    'dragon': '+R', // Promoted rook (Dragon)
  };
  
  return pieceMap[pieceType] || 'P';
}

// Helper function to convert USI piece character to CSA piece code
function usiPieceToCsa(piece: string): string {
  const pieceMap: { [key: string]: string } = {
    'P': 'FU',   // Pawn
    'L': 'KY',   // Lance
    'N': 'KE',   // Knight
    'S': 'GI',   // Silver
    'G': 'KI',   // Gold
    'B': 'KA',   // Bishop
    'R': 'HI',   // Rook
    'K': 'OU',   // King
    // Promoted pieces
    '+P': 'TO',  // Promoted Pawn
    '+L': 'NY',  // Promoted Lance
    '+N': 'NK',  // Promoted Knight
    '+S': 'NG',  // Promoted Silver
    '+B': 'UM',  // Horse (Promoted Bishop)
    '+R': 'RY',  // Dragon (Promoted Rook)
  };
  
  return pieceMap[piece] || 'FU';
}

// Helper function to convert USI piece character to CSA promoted piece code
function usiPieceToCsaPromoted(piece: string): string {
  const promotedPieceMap: { [key: string]: string } = {
    'P': 'TO',   // Pawn -> Promoted Pawn
    'L': 'NY',   // Lance -> Promoted Lance
    'N': 'NK',   // Knight -> Promoted Knight
    'S': 'NG',   // Silver -> Promoted Silver
    'B': 'UM',   // Bishop -> Horse
    'R': 'RY',   // Rook -> Dragon
    // Already promoted pieces
    '+P': 'TO',  // Promoted Pawn
    '+L': 'NY',  // Promoted Lance
    '+N': 'NK',  // Promoted Knight
    '+S': 'NG',  // Promoted Silver
    '+B': 'UM',  // Horse (Promoted Bishop)
    '+R': 'RY',  // Dragon (Promoted Rook)
  };
  
  return promotedPieceMap[piece] || 'TO';
}

// Helper function to convert position to CSA format (e.g., "5d" -> "55")
function convertPositionToCsa(position: string): string {
  const file = position.charAt(0);
  const rank = position.charAt(1);
  
  const rankMap: { [key: string]: string } = {
    'a': '1', 'b': '2', 'c': '3', 'd': '4', 'e': '5',
    'f': '6', 'g': '7', 'h': '8', 'i': '9'
  };
  
  return `${file}${rankMap[rank] || rank}`;
}

// Convert move record to display format based on notation type
export function formatMoveForDisplay(move: any, notation: 'western' | 'kifu' | 'usi' | 'csa', isBlack: boolean): string {
  // Check if this is a regular move with USI string
  if (move.move && typeof move.move === 'object' && 'usi' in move.move) {
    const usi = move.move.usi;
    if (notation === 'western') {
      return usiToWesternWithMove(usi, move);
    } else if (notation === 'usi') {
      return usiToUsi(usi);
    } else if (notation === 'csa') {
      return usiToCsaWithMove(usi, move, isBlack);
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