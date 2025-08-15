import React from "react";
import { KANJI_MAP, ENGLISH_MAP } from "../utils/pieceMaps";
import type { Piece, PieceType, Player } from '../types';

const PIECE_PATHS: { [key in PieceType]?: string } = {
  K: "M35 4 L62 10 L65 72 L5 72 L8 10 Z",
  R: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  B: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  G: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  S: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  N: "M35 7 L57 14 L62 69 L8 69 L13 14 Z",
  L: "M35 8 L56 15 L60 68 L10 68 L15 15 Z",
  P: "M35 9 L55 16 L59 67 L11 67 L16 16 Z",
  "+R": "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  "+B": "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  "+S": "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  "+N": "M35 7 L57 14 L62 69 L8 69 L13 14 Z",
  "+L": "M35 8 L56 15 L60 68 L10 68 L15 15 Z",
  "+P": "M35 9 L55 16 L59 67 L11 67 L16 16 Z",
};

interface SvgPieceProps {
  type?: PieceType;
  player?: Player;
  pieceLabelType?: string;
  piece?: Piece;
  size?: number;
  hideText?: boolean;
}

const SvgPiece: React.FC<SvgPieceProps> = ({ type, player, pieceLabelType, piece, size = 70, hideText = false }) => {
  const pieceType = type || (piece && piece.type);
  const piecePlayer = player || (piece && piece.player);
  const labelType = pieceLabelType || 'kanji';
  
  if (!pieceType) {
    console.warn('SvgPiece: type prop is required');
    return null;
  }

  const isPromoted = pieceType.startsWith("+");
  const label = labelType === "kanji" ? KANJI_MAP[pieceType] : ENGLISH_MAP[pieceType];

  const piecePath = PIECE_PATHS[pieceType];

  if (!piecePath) {
    console.warn(`SvgPiece: No path found for piece type: ${pieceType}`);
    return null;
  }

  const fillColor = "url(#wood-bambo-pattern)";
  const strokeColor = "#333";
  const textColor = isPromoted ? "#b80000" : "black";

  return (
    <svg
      width={size}
      height={size * 1.086}
      viewBox="0 0 70 76"
      className={piecePlayer === "player2" ? "rotate-180" : ""}
    >
      <defs>
        <pattern
          id="wood-bambo-pattern"
          patternUnits="objectBoundingBox"
          width="1"
          height="1"
        >
          <image
            href="/boards/wood-ginkgo-1.jpg"
            x="0"
            y="0"
            width="70"
            height="76"
            preserveAspectRatio="none"
          ></image>
        </pattern>
      </defs>
      <path
        d={piecePath}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth="1"
      />
      {!hideText && (
        <text
          x="35"
          y="45"
          textAnchor="middle"
          dominantBaseline="middle"
          fontSize={label.length === 2 ? "24" : "36"}
          fill={textColor}
          fontFamily={
            labelType === "kanji"
              ? `'Noto Sans JP', sans-serif`
              : "sans-serif"
          }
        >
          {label}
        </text>
      )}
    </svg>
  );
};

export default SvgPiece;
