import {
  PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING,
  PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK
} from '../game/engine';

export const KANJI_MAP = {
  [PAWN]: '歩',
  [LANCE]: '香',
  [KNIGHT]: '桂',
  [SILVER]: '銀',
  [GOLD]: '金',
  [BISHOP]: '角',
  [ROOK]: '飛',
  [KING]: '王',
  [PROMOTED_PAWN]: 'と',
  [PROMOTED_LANCE]: '成香',
  [PROMOTED_KNIGHT]: '成桂',
  [PROMOTED_SILVER]: '成銀',
  [PROMOTED_BISHOP]: '竜馬',
  [PROMOTED_ROOK]: '竜王',
};

export const ENGLISH_MAP = {
  [PAWN]: 'P',
  [LANCE]: 'L',
  [KNIGHT]: 'N',
  [SILVER]: 'S',
  [GOLD]: 'G',
  [BISHOP]: 'B',
  [ROOK]: 'R',
  [KING]: 'K',
  [PROMOTED_PAWN]: 'P+',
  [PROMOTED_LANCE]: 'L+',
  [PROMOTED_KNIGHT]: 'N+',
  [PROMOTED_SILVER]: 'S+',
  [PROMOTED_BISHOP]: 'B+',
  [PROMOTED_ROOK]: 'R+',
};

export const ENGLISH_NAMES = {
  [PAWN]: 'Pawn',
  [LANCE]: 'Lance',
  [KNIGHT]: 'Knight',
  [SILVER]: 'Silver',
  [GOLD]: 'Gold',
  [BISHOP]: 'Bishop',
  [ROOK]: 'Rook',
  [KING]: 'King',
  [PROMOTED_PAWN]: 'Promoted Pawn',
  [PROMOTED_LANCE]: 'Promoted Lance',
  [PROMOTED_KNIGHT]: 'Promoted Knight',
  [PROMOTED_SILVER]: 'Promoted Silver',
  [PROMOTED_BISHOP]: 'Promoted Bishop',
  [PROMOTED_ROOK]: 'Promoted Rook',
};

export function getOppositeLabel(pieceType, currentLabelType) {
  if (currentLabelType === 'kanji') {
    return ENGLISH_MAP[pieceType];
  } else {
    return KANJI_MAP[pieceType];
  }
}

export function getEnglishName(pieceType) {
  return ENGLISH_NAMES[pieceType];
}
