import React, { useEffect, useState } from 'react';
import { Square, PieceType } from 'tsshogi';
import SvgPiece from './SvgPiece';
import { BoardRef } from './Board';

interface RecommendationOverlayProps {
  recommendation: { from: Square | null; to: Square | null; isDrop?: boolean; pieceType?: string; isPromotion?: boolean } | null;
  boardRef: React.RefObject<HTMLDivElement | null>;
  boardComponentRef: React.RefObject<BoardRef | null>; // Added
  pieceThemeType?: string;
  currentPlayer?: 'black' | 'white';
  onHighlightCapturedPiece?: (pieceType: string | null) => void;
}

const RecommendationOverlay: React.FC<RecommendationOverlayProps> = ({
  recommendation, 
  boardRef,
  boardComponentRef,
  pieceThemeType = 'kanji',
  currentPlayer = 'black',
  onHighlightCapturedPiece
}) => {
  const [boardSize, setBoardSize] = useState({ width: 630, height: 684 });
  const [dropOverlayStyle, setDropOverlayStyle] = useState({});
  const [arrowPositions, setArrowPositions] = useState<{ from: { x: number; y: number }; to: { x: number; y: number } } | null>(null);

  useEffect(() => {
    const updateBoardSize = () => {
      if (boardRef.current) {
        const boardElement = boardRef.current.querySelector('.board') as HTMLElement;
        if (boardElement) {
          const rect = boardElement.getBoundingClientRect();
          setBoardSize({ width: rect.width, height: rect.height });
        } else {
          const rect = boardRef.current.getBoundingClientRect();
          setBoardSize({ width: rect.width, height: rect.height });
        }
      }
    };

    updateBoardSize();
    window.addEventListener('resize', updateBoardSize);
    return () => {
      window.removeEventListener('resize', updateBoardSize);
    };
  }, [boardRef, recommendation]);

  useEffect(() => {
    if (recommendation?.isDrop && recommendation.to && boardComponentRef.current && boardRef.current) {
      const squareElement = boardComponentRef.current.getSquareRef(recommendation.to.usi);
      if (squareElement) {
        const squareRect = squareElement.getBoundingClientRect();
        const boardRect = boardRef.current.getBoundingClientRect();
        const squareSize = boardSize.width / 9;
        const pieceSize = squareRect.width * 0.8;

        setDropOverlayStyle({
          position: 'absolute',
          top: squareRect.top - boardRect.top,
          left: squareRect.left - boardRect.left,
          width: squareRect.width,
          height: squareRect.height,
          zIndex: 10,
          pointerEvents: 'none',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: 'rgba(34, 197, 94, 0.3)',
          borderRadius: '4px',
          border: '2px solid #22C55E',
        });
      }
    } else {
      setDropOverlayStyle({});
    }
  }, [recommendation, boardComponentRef, boardRef, boardSize]);

  useEffect(() => {
    if (!recommendation?.isDrop && recommendation?.from && recommendation?.to && boardComponentRef.current && boardRef.current) {
      const fromSquareElement = boardComponentRef.current.getSquareRef(recommendation.from.usi);
      const toSquareElement = boardComponentRef.current.getSquareRef(recommendation.to.usi);
      const boardRect = boardRef.current.getBoundingClientRect();

      if (fromSquareElement && toSquareElement) {
        const fromRect = fromSquareElement.getBoundingClientRect();
        const toRect = toSquareElement.getBoundingClientRect();

        setArrowPositions({
          from: {
            x: fromRect.left - boardRect.left + fromRect.width / 2,
            y: fromRect.top - boardRect.top + fromRect.height / 2,
          },
          to: {
            x: toRect.left - boardRect.left + toRect.width / 2,
            y: toRect.top - boardRect.top + toRect.height / 2,
          },
        });
      }
    } else {
      setArrowPositions(null);
    }
  }, [recommendation, boardComponentRef, boardRef]);

  useEffect(() => {
    if (recommendation?.isDrop && onHighlightCapturedPiece) {
      onHighlightCapturedPiece(recommendation.pieceType || null);
    } else if (onHighlightCapturedPiece) {
      onHighlightCapturedPiece(null);
    }
  }, [recommendation, onHighlightCapturedPiece]);

  if (!recommendation || !recommendation.to) {
    return null;
  }

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

  if (recommendation.isDrop) {
    const squareSize = boardSize.width / 9;
    const pieceSize = squareSize * 0.8;
    const pieceType = getPieceTypeFromString(recommendation.pieceType || '');
    const player = currentPlayer === 'black' ? 'player1' : 'player2';
    
    if (!pieceType) {
      return null;
    }
    
    return (
      <div
        className="recommendation-drop-overlay"
        style={dropOverlayStyle}
      >
        <div style={{ 
          width: pieceSize * 0.7, 
          height: pieceSize * 0.7,
          animation: 'breathing 2s ease-in-out infinite',
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
  } else if (arrowPositions) {
    return (
      <svg 
        className="recommendation-arrow-svg" 
        width={boardSize.width} 
        height={boardSize.height} 
        style={{ 
          position: 'absolute', 
          top: 0, 
          left: 0, 
          zIndex: 10,
          pointerEvents: 'none'
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
          x1={arrowPositions.from.x}
          y1={arrowPositions.from.y}
          x2={arrowPositions.to.x}
          y2={arrowPositions.to.y}
          stroke="#22C55E"
          strokeWidth="3"
          strokeOpacity="0.9"
          markerEnd="url(#arrowhead)"
        />
        {recommendation.isPromotion && (
          <g>
            <circle
              cx={arrowPositions.to.x + 20}
              cy={arrowPositions.to.y - 15}
              r="12"
              fill="#22C55E"
              stroke="#ffffff"
              strokeWidth="2"
            />
            <text
              x={arrowPositions.to.x + 20}
              y={arrowPositions.to.y - 10}
              fontSize="14"
              fontWeight="bold"
              fill="#ffffff"
              textAnchor="middle"
              style={{ textShadow: '1px 1px 2px rgba(0,0,0,0.8)' }}
            >
              +
            </text>
          </g>
        )}
      </svg>
    );
  }
  
  return null;
};

export default RecommendationOverlay;
