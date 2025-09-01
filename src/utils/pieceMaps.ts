import { PieceType } from 'tsshogi';

export const KANJI_MAP: Record<PieceType, string> = {
  PAWN: '歩',
  LANCE: '香',
  KNIGHT: '桂',
  SILVER: '銀',
  GOLD: '金',
  BISHOP: '角',
  ROOK: '飛',
  KING: '王',
  PROM_PAWN: 'と',
  PROM_LANCE: '成香',
  PROM_KNIGHT: '成桂',
  PROM_SILVER: '成銀',
  HORSE: '竜馬',
  DRAGON: '竜王',
};

export const ENGLISH_MAP: Record<PieceType, string> = {
  PAWN: 'P',
  LANCE: 'L',
  KNIGHT: 'N',
  SILVER: 'S',
  GOLD: 'G',
  BISHOP: 'B',
  ROOK: 'R',
  KING: 'K',
  PROM_PAWN: 'P+',
  PROM_LANCE: 'L+',
  PROM_KNIGHT: 'N+',
  PROM_SILVER: 'S+',
  HORSE: 'B+',
  DRAGON: 'R+',
};

export const ENGLISH_NAMES: Record<PieceType, string> = {
  PAWN: 'Pawn',
  LANCE: 'Lance',
  KNIGHT: 'Knight',
  SILVER: 'Silver',
  GOLD: 'Gold',
  BISHOP: 'Bishop',
  ROOK: 'Rook',
  KING: 'King',
  PROM_PAWN: 'Promoted Pawn',
  PROM_LANCE: 'Promoted Lance',
  PROM_KNIGHT: 'Promoted Knight',
  PROM_SILVER: 'Promoted Silver',
  HORSE: 'Promoted Bishop',
  DRAGON: 'Promoted Rook',
};

export function getOppositeLabel(pieceType: PieceType, currentLabelType: 'kanji' | 'english'): string {
  if (currentLabelType === 'kanji') {
    return ENGLISH_MAP[pieceType];
  } else {
    return KANJI_MAP[pieceType];
  }
}

export function getEnglishName(pieceType: PieceType): string {
  return ENGLISH_NAMES[pieceType];
}