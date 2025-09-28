import React from "react";
import { KANJI_MAP, ENGLISH_MAP } from "../utils/pieceMaps";
import { PieceType } from 'tsshogi';
import { getSvgPathForPiece, isSvgTheme } from "../utils/pieceThemes";

const PIECE_PATHS: { [key: string]: string } = {
  pawn: "M35 9 L55 16 L59 67 L11 67 L16 16 Z",
  lance: "M35 8 L56 15 L60 68 L10 68 L15 15 Z",
  knight: "M35 7 L57 14 L62 69 L8 69 L13 14 Z",
  silver: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  gold: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  bishop: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  rook: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  king: "M35 4 L62 10 L65 72 L5 72 L8 10 Z",
  promPawn: "M35 9 L55 16 L59 67 L11 67 L16 16 Z",
  promLance: "M35 8 L56 15 L60 68 L10 68 L15 15 Z",
  promKnight: "M35 7 L57 14 L62 69 L8 69 L13 14 Z",
  promSilver: "M35 6 L58 13 L63 70 L7 70 L12 13 Z",
  horse: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
  dragon: "M35 5 L60 12 L64 71 L6 71 L10 12 Z",
};

interface SvgPieceProps {
  type: string;
  player?: 'player1' | 'player2';
  pieceThemeType?: string;
  size?: number;
  hideText?: boolean;
  isSelected?: boolean;
}

const SvgPiece: React.FC<SvgPieceProps> = ({ type, player, pieceThemeType, size = 70, hideText = false, isSelected = false }) => {
  // Early return if type is undefined
  if (!type) {
    console.warn('SvgPiece: type prop is undefined');
    return null;
  }
  
  const pieceType: string = type;
  const piecePlayer = player;
  const themeType = pieceThemeType || 'kanji';
  
  // Handle SVG themes
  if (isSvgTheme(themeType) && piecePlayer) {
    const svgPath = getSvgPathForPiece(pieceType as PieceType, piecePlayer, themeType);
    
    return (
      <div
        style={{
          width: size,
          height: size * 1.086,
          position: 'relative',
          filter: isSelected ? 'drop-shadow(0 8px 4px rgba(0, 0, 0, 0.6))' : 'none'
        }}
      >
        <img
          src={svgPath}
          alt={`${pieceType} piece`}
          style={{
            width: '100%',
            height: '100%',
            objectFit: 'contain'
          }}
          onError={(e) => {
            console.warn(`Failed to load SVG for piece: ${svgPath}`);
            // Fallback to default piece rendering
            e.currentTarget.style.display = 'none';
          }}
        />
      </div>
    );
  }

  // Handle text-based themes (kanji/english)
  const isPromoted = pieceType && (pieceType.includes('prom') || pieceType === 'horse' || pieceType === 'dragon');
  const label = themeType === "kanji" ? KANJI_MAP[pieceType] : ENGLISH_MAP[pieceType];

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
        {isSelected && (
          <filter id="piece-shadow" x="-50%" y="-50%" width="200%" height="200%">
            <feDropShadow
              dx="0"
              dy="8"
              stdDeviation="4"
              floodColor="rgba(0, 0, 0, 0.6)"
              floodOpacity="0.8"
            />
          </filter>
        )}
      </defs>
      <path
        d={piecePath}
        fill={fillColor}
        stroke={strokeColor}
        strokeWidth="1"
        filter={isSelected ? "url(#piece-shadow)" : undefined}
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
            themeType === "kanji"
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