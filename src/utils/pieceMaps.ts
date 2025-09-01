import { PieceType } from 'tsshogi';

export const KANJI_MAP: Record<PieceType, string> = {
  pawn: '歩',
  lance: '香',
  knight: '桂',
  silver: '銀',
  gold: '金',
  bishop: '角',
  rook: '飛',
  king: '王',
  promPawn: 'と',
  promLance: '成香',
  promKnight: '成桂',
  promSilver: '成銀',
  horse: '竜馬',
  dragon: '竜王',
};

export const ENGLISH_MAP: Record<PieceType, string> = {
  pawn: 'P',
  lance: 'L',
  knight: 'N',
  silver: 'S',
  gold: 'G',
  bishop: 'B',
  rook: 'R',
  king: 'K',
  promPawn: 'P+',
  promLance: 'L+',
  promKnight: 'N+',
  promSilver: 'S+',
  horse: 'B+',
  dragon: 'R+',
};

export const ENGLISH_NAMES: Record<PieceType, string> = {
  pawn: 'Pawn',
  lance: 'Lance',
  knight: 'Knight',
  silver: 'Silver',
  gold: 'Gold',
  bishop: 'Bishop',
  rook: 'Rook',
  king: 'King',
  promPawn: 'Promoted Pawn',
  promLance: 'Promoted Lance',
  promKnight: 'Promoted Knight',
  promSilver: 'Promoted Silver',
  horse: 'Promoted Bishop',
  dragon: 'Promoted Rook',
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