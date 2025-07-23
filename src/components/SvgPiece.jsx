import React from 'react';
import {
  PAWN, LANCE, KNIGHT, SILVER, GOLD, BISHOP, ROOK, KING,
  PROMOTED_PAWN, PROMOTED_LANCE, PROMOTED_KNIGHT, PROMOTED_SILVER, PROMOTED_BISHOP, PROMOTED_ROOK
} from '../game/engine';

const KANJI_MAP = {
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

const SvgPiece = ({ type, player }) => {
  const isPromoted = type.startsWith('+');
  const kanji = KANJI_MAP[type];

  // SVG path for a pentagonal Shogi piece (adjust as needed)
  // This is a simplified representation, actual dimensions might vary
  const piecePath = "M25 0 L45 15 L40 50 L10 50 L5 15 Z";

  const fillColor = player === 'player1' ? 'white' : '#e0e0e0';
  const strokeColor = '#333';
  const textColor = isPromoted ? '#b80000' : 'black';

  return (
    <svg width="60" height="60" viewBox="0 0 60 60">
      <path d={piecePath} fill={fillColor} stroke={strokeColor} strokeWidth="1" />
      <text
        x="25"
        y="32"
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize="28"
        fill={textColor}
        fontFamily="'Noto Sans JP', sans-serif"
      >
        {kanji}
      </text>
    </svg>
  );
};

export default SvgPiece;
