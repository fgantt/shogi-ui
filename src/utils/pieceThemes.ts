import { PieceType } from 'tsshogi';

// CSA notation mapping for piece types
export const PIECE_TYPE_TO_CSA: Record<PieceType, string> = {
  pawn: 'FU',
  lance: 'KY', 
  knight: 'KE',
  silver: 'GI',
  gold: 'KI',
  bishop: 'KA',
  rook: 'HI',
  king: 'OU',
  promPawn: 'TO',
  promLance: 'NY',
  promKnight: 'NK',
  promSilver: 'NG',
  horse: 'UM',
  dragon: 'RY',
};

// Available piece theme folders from public/piece-themes
export const AVAILABLE_PIECE_THEMES = [
  '13xforever-1-kanji',
  '13xforever-2-kanji',
  'Hari-Seldon-1-kanji',
  'Hari-Seldon-2-kanji',
  'Kinki-1-kanji',
  'Kinki-2-kanji',
  'Minase-1-kanji',
  'Minase-2-kanji',
  'Ryoko-1-kanji',
  'Ryoko-2-kanji',
];

// Get SVG path for a piece in a specific theme
export function getSvgPathForPiece(
  pieceType: PieceType,
  player: 'player1' | 'player2',
  theme: string
): string {
  let csaCode = PIECE_TYPE_TO_CSA[pieceType];
  
  // For SVG themes, use GY for black king instead of OU
  if (pieceType === 'king' && player === 'player1') {
    csaCode = 'GY';
  }
  
  const playerPrefix = player === 'player1' ? '0' : '1';
  return `/piece-themes/${theme}/${playerPrefix}${csaCode}.svg`;
}

// Check if a theme is an SVG theme (not English/Kanji)
export function isSvgTheme(theme: string): boolean {
  return theme !== 'english' && theme !== 'kanji';
}
