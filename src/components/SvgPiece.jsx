import React from 'react';
import { KANJI_MAP, ENGLISH_MAP } from '../utils/pieceMaps';

const SvgPiece = ({ type, player, pieceLabelType }) => {
  // console.log("SvgPiece received type:", type, "pieceLabelType:", pieceLabelType);
  const isPromoted = type.startsWith('+');
  const label = pieceLabelType === 'kanji' ? KANJI_MAP[type] : ENGLISH_MAP[type];

  // SVG path for a pentagonal Shogi piece (adjust as needed)
  // This is a simplified representation, actual dimensions might vary
  const piecePath = "M25 0 L45 15 L40 50 L10 50 L5 15 Z";

  const fillColor = player === 'player1' ? 'white' : '#e0e0e0';
  const strokeColor = '#333';
  const textColor = isPromoted ? '#b80000' : 'black';

  return (
    <svg width="60" height="60" viewBox="-5 -5 60 60">
      <path d={piecePath} fill={fillColor} stroke={strokeColor} strokeWidth="1" />
      <text
        x="25"
        y="32"
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize="28"
        fill={textColor}
        fontFamily={pieceLabelType === 'kanji' ? `'Noto Sans JP', sans-serif` : 'sans-serif'}
      >
        {label}
      </text>
    </svg>
  );
};

export default SvgPiece;
