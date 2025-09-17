import React, { useEffect, useState } from 'react';
import { Square, PieceType } from 'tsshogi';
import SvgPiece from './SvgPiece';

interface RecommendationOverlayProps {
  recommendation: { from: Square | null; to: Square | null; isDrop?: boolean; pieceType?: string; isPromotion?: boolean } | null;
  boardRef: React.RefObject<HTMLDivElement | null>;
  pieceThemeType?: string;
  currentPlayer?: 'black' | 'white';
  onHighlightCapturedPiece?: (pieceType: string | null) => void;
}

const RecommendationOverlay: React.FC<RecommendationOverlayProps> = ({ 
  recommendation, 
  boardRef,
  pieceThemeType = 'kanji',
  currentPlayer = 'black',
  onHighlightCapturedPiece
}) => {
  const [boardSize, setBoardSize] = useState({ width: 630, height: 684 });

  useEffect(() => {
    const updateBoardSize = () => {
      if (boardRef.current) {
        const boardElement = boardRef.current.querySelector('.board') as HTMLElement;
        if (boardElement) {
          const rect = boardElement.getBoundingClientRect();
          setBoardSize({ width: rect.width, height: rect.height });
          console.log('RecommendationOverlay: Updated board size:', { width: rect.width, height: rect.height });
        } else {
          // Fallback to container size if board element not found
          const rect = boardRef.current.getBoundingClientRect();
          setBoardSize({ width: rect.width, height: rect.height });
          console.log('RecommendationOverlay: Using container size as fallback:', { width: rect.width, height: rect.height });
        }
      }
    };

    // Update size on mount and when recommendation changes
    updateBoardSize();

    // Listen for window resize events
    window.addEventListener('resize', updateBoardSize);
    
    return () => {
      window.removeEventListener('resize', updateBoardSize);
    };
  }, [boardRef, recommendation]);

  // Handle captured piece highlighting for drop recommendations
  useEffect(() => {
    if (recommendation?.isDrop && onHighlightCapturedPiece) {
      onHighlightCapturedPiece(recommendation.pieceType || null);
    } else if (onHighlightCapturedPiece) {
      onHighlightCapturedPiece(null);
    }
  }, [recommendation, onHighlightCapturedPiece]);

  console.log('RecommendationOverlay received recommendation:', recommendation);
  
  if (!recommendation || !recommendation.to) {
    console.log('RecommendationOverlay: No valid recommendation to render');
    return null;
  }

  console.log('RecommendationOverlay: Rendering arrow for recommendation:', recommendation);

  // Convert piece type string to PieceType enum
  const getPieceTypeFromString = (pieceTypeStr: string): PieceType | null => {
    const pieceMap: { [key: string]: PieceType } = {
      'P': PieceType.PAWN,
      'L': PieceType.LANCE, 
      'N': PieceType.KNIGHT,
      'S': PieceType.SILVER,
      'G': PieceType.GOLD,
      'B': PieceType.BISHOP,
      'R': PieceType.ROOK,
      'K': PieceType.KING
    };
    return pieceMap[pieceTypeStr] || null;
  };

  // Convert square coordinates to pixel positions
  // Match Board.tsx calculation exactly but scaled to actual board size
  const squareToPixel = (square: Square) => {
    // Board.tsx uses fixed values: colIndex * 70 + 35, rowIndex * 76 + 38
    // We need to scale these to the actual board size
    const scaleX = boardSize.width / 630; // 630 is the standard board width
    const scaleY = boardSize.height / 684; // 684 is the standard board height
    
    // Use the same coordinate system as the Board component:
    // colIndex = 9 - file (file 1 -> col 8, file 9 -> col 0)
    // rowIndex = rank - 1 (rank 1 -> row 0, rank 9 -> row 8)
    const colIndex = 9 - square.file;
    const rowIndex = square.rank - 1;
    
    // Scale the Board.tsx calculation to match the actual board size
    const x = (colIndex * 70 + 35) * scaleX;
    const y = (rowIndex * 76 + 38) * scaleY;
    
    console.log(`Square ${square.usi} (file:${square.file}, rank:${square.rank}) -> col:${colIndex}, row:${rowIndex} -> x:${x}, y:${y}`);
    console.log(`Scale factors: x:${scaleX}, y:${scaleY}`);
    
    return { x, y };
  };

  const toPos = squareToPixel(recommendation.to);
  
  console.log('RecommendationOverlay: Arrow positions:', { to: toPos, isDrop: recommendation.isDrop });
  console.log('RecommendationOverlay: Board size:', boardSize);

  // Calculate the offset for column labels
  // The column labels are positioned above the board, so we need to offset down
  const columnLabelHeight = 35; // Approximate height of column labels
  
  // For drop moves, show a green-tinted piece image instead of a line
  if (recommendation.isDrop) {
    const squareSize = boardSize.width / 9;
    const pieceSize = squareSize * 0.8; // Make piece slightly smaller than square
    const scaleY = boardSize.height / 684; // Scale factor for vertical positioning
    const pieceType = getPieceTypeFromString(recommendation.pieceType || '');
    const player = currentPlayer === 'black' ? 'player1' : 'player2';
    
    if (!pieceType) {
      console.log('RecommendationOverlay: Unknown piece type:', recommendation.pieceType);
      return null;
    }
    
    return (
      <div
        className="recommendation-drop-overlay"
        style={{
          position: 'absolute',
          top: columnLabelHeight + toPos.y - pieceSize / scaleY,
          left: toPos.x - pieceSize / 2,
          width: pieceSize,
          height: pieceSize,
          zIndex: 10,
          pointerEvents: 'none',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: 'rgba(34, 197, 94, 0.3)', // Green tint
          borderRadius: '4px',
          border: '2px solid #22C55E',
          filter: 'hue-rotate(0deg) saturate(1.5) brightness(1.2)', // Green tint filter
        }}
      >
        <div style={{ 
          width: pieceSize * 0.7, 
          height: pieceSize * 0.7,
          filter: 'hue-rotate(120deg) saturate(1.5) brightness(1.2)', // Additional green tint
        }}>
          <SvgPiece 
            type={pieceType} 
            player={player} 
            pieceThemeType={pieceThemeType}
            size={pieceSize * 0.7}
          />
        </div>
      </div>
    );
  }
  
  // For regular moves, show an arrow from source to destination
  if (!recommendation.from) {
    console.log('RecommendationOverlay: Invalid recommendation - no from square and not a drop');
    return null;
  }
  
  const fromPos = squareToPixel(recommendation.from);
  
  return (
    <svg 
      className="recommendation-arrow-svg" 
      width={boardSize.width} 
      height={boardSize.height} 
      style={{ 
        position: 'absolute', 
        top: columnLabelHeight, 
        left: 0, 
        zIndex: 10,
        pointerEvents: 'none' // Allow clicks to pass through to the board
      }}
    >
      <defs>
        <marker
          id="arrowhead"
          markerWidth="8"
          markerHeight="6"
          refX="7"
          refY="3"
          orient="auto"
        >
          <path
            d="M 0 0 L 8 3 L 0 6"
            fill="none"
            stroke="#22C55E"
            strokeWidth="2"
          />
        </marker>
      </defs>
      <line
        x1={fromPos.x}
        y1={fromPos.y}
        x2={toPos.x}
        y2={toPos.y}
        stroke="#22C55E"
        strokeWidth="3"
        strokeOpacity="0.9"
        markerEnd="url(#arrowhead)"
      />
      {/* Add promotion indicator if this is a promotion move */}
      {recommendation.isPromotion && (
        <g>
          {/* Add a "+" symbol near the destination */}
          <text
            x={toPos.x + 15}
            y={toPos.y - 10}
            fontSize="16"
            fontWeight="bold"
            fill="#22C55E"
            textAnchor="middle"
            style={{ textShadow: '1px 1px 2px rgba(0,0,0,0.8)' }}
          >
            +
          </text>
          {/* Add a small circle around the destination to emphasize promotion */}
          <circle
            cx={toPos.x}
            cy={toPos.y}
            r="12"
            fill="none"
            stroke="#22C55E"
            strokeWidth="2"
            strokeDasharray="3,3"
          />
        </g>
      )}
    </svg>
  );
};

export default RecommendationOverlay;
