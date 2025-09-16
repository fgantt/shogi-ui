import React, { useEffect, useState } from 'react';
import { Square } from 'tsshogi';

interface RecommendationOverlayProps {
  recommendation: { from: Square | null; to: Square | null } | null;
  boardRef: React.RefObject<HTMLDivElement | null>;
}

const RecommendationOverlay: React.FC<RecommendationOverlayProps> = ({ 
  recommendation, 
  boardRef
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

  console.log('RecommendationOverlay received recommendation:', recommendation);
  
  if (!recommendation || !recommendation.from || !recommendation.to) {
    console.log('RecommendationOverlay: No valid recommendation to render');
    return null;
  }

  console.log('RecommendationOverlay: Rendering arrow for recommendation:', recommendation);

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
    const squareWidth = 70 * scaleX;
    const squareHeight = 76 * scaleY;

    const x = (colIndex * squareWidth + squareWidth / 2);
    const y = (rowIndex * squareHeight + squareHeight / 2);
    
    console.log(`Square ${square.usi} (file:${square.file}, rank:${square.rank}) -> col:${colIndex}, row:${rowIndex} -> x:${x}, y:${y}`);
    console.log(`Scale factors: x:${scaleX}, y:${scaleY}`);
    
    return { x, y };
  };

  const fromPos = squareToPixel(recommendation.from);
  const toPos = squareToPixel(recommendation.to);
  
  console.log('RecommendationOverlay: Arrow positions:', { from: fromPos, to: toPos });
  console.log('RecommendationOverlay: Board size:', boardSize);

  // Calculate the offset for column labels
  // The column labels are positioned above the board, so we need to offset down
  const columnLabelHeight = 35; // Approximate height of column labels
  
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
    </svg>
  );
};

export default RecommendationOverlay;
