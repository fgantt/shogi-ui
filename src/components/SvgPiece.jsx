import React from "react";
import { KANJI_MAP, ENGLISH_MAP } from "../utils/pieceMaps";

const PIECE_PATHS = {
  // King 60x68
  K: "M35 4 L62 10 L65 72 L5 72 L8 10 Z",
  // Rook 58x66
  R: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  // Bishop 58x66
  B: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  // Gold General 56x64
  G: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  // Silver General 56x64
  S: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  // Knight 54x62
  N: "M35 7 L57 14 L62 69 L8 69 L13 14 Z",
  // Lance 50x60
  L: "M35 8 L56 15 L60 68 L10 68 L15 15 Z",
  // Pawn 48x58
  P: "M35 9 L55 16 L59 67 L11 67 L16 16 Z",

  // Promoted pieces (example, adjust as needed)
  "+R": "M35 5 L60 12 L64 71 L6 71 L10 12 Z", // Promoted Rook (Dragon King)
  "+B": "M35 5 L60 12 L64 71 L6 71 L10 12 Z", // Promoted Bishop (Dragon Horse)
  "+S": "M35 6 L58 13 L63 70 L7 70 L12 13 Z", // Promoted Silver
  "+N": "M35 7 L57 14 L62 69 L8 69 L13 14 Z", // Promoted Knight
  "+L": "M35 8 L56 15 L60 68 L10 68 L15 15 Z", // Promoted Lance
  "+P": "M35 9 L55 16 L59 67 L11 67 L16 16 Z", // Promoted Pawn
};

const SvgPiece = ({ type, player, pieceLabelType }) => {
  const isPromoted = type.startsWith("+");
  const label =
    pieceLabelType === "kanji" ? KANJI_MAP[type] : ENGLISH_MAP[type];

  const piecePath = PIECE_PATHS[type];

  const fillColor = "url(#wood-bambo-pattern)";
  const strokeColor = "#333";
  const textColor = isPromoted ? "#b80000" : "black";

  return (
    <svg width="70" height="76" viewBox="0 0 70 76">
      <defs>
        <pattern
          id="wood-bambo-pattern"
          patternUnits="objectBoundingBox"
          width="1"
          height="1"
        >
          <image
            href="/public/boards/wood-bambo.jpg"
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
      <text
        x="35"
        y="45"
        textAnchor="middle"
        dominantBaseline="middle"
        fontSize={label.length === 2 ? "24" : "36"}
        fill={textColor}
        fontFamily={
          pieceLabelType === "kanji"
            ? `'Noto Sans JP', sans-serif`
            : "sans-serif"
        }
      >
        {label}
      </text>
    </svg>
  );
};

export default SvgPiece;
