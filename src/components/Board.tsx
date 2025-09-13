import React from 'react';
import { ImmutablePosition, Square } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

interface BoardProps {
  position: ImmutablePosition;
  onSquareClick: (row: number, col: number) => void;
  onDragStart?: (row: number, col: number) => void;
  onDragEnd?: (row: number, col: number) => void;
  onDragOver?: (row: number, col: number) => void;
  selectedSquare: Square | null;
  legalMoves: Square[];
  lastMove: { from: Square | null; to: Square | null } | null;
  isSquareAttacked?: (square: Square) => boolean;
  isInCheck?: boolean;
  kingInCheckSquare?: Square | null;
  attackingPieces?: Square[];
  boardBackground?: string;
  pieceThemeType?: string;
}



// Helper to map tsshogi color to our Player type
function toOurPlayer(color: string): 'player1' | 'player2' {
    return color === 'black' ? 'player1' : 'player2';
}

const Board: React.FC<BoardProps> = ({ position, onSquareClick, onDragStart, onDragEnd, onDragOver, selectedSquare, legalMoves, lastMove, isSquareAttacked, isInCheck, kingInCheckSquare, attackingPieces, boardBackground, pieceThemeType }) => {
  const columnNumbers = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const kifuRowLabels = ['一', '二', '三', '四', '五', '六', '七', '八', '九'];

  const isSelected = (row: number, col: number): boolean => {
    if (!selectedSquare) return false;
    // Convert row/col to tsshogi coordinates for comparison
    const square = Square.newByXY(col, row);
    return square ? selectedSquare.equals(square) : false;
  };

  const isLegalMove = (row: number, col: number): boolean => {
    const square = Square.newByXY(col, row);
    if (!square) return false;
    return legalMoves.some(move => move.equals(square));
  };

  const isLastMove = (row: number, col: number): boolean => {
    if (!lastMove) return false;
    const square = Square.newByXY(col, row);
    if (!square) return false;
    
    return (lastMove.from ? lastMove.from.equals(square) : false) || 
           (lastMove.to ? lastMove.to.equals(square) : false);
  };

  const isInCheckSquare = (row: number, col: number): boolean => {
    if (!isInCheck || !kingInCheckSquare) return false;
    const square = Square.newByXY(col, row);
    if (!square) return false;
    return square.equals(kingInCheckSquare);
  };

  // Helper function to convert square coordinates to pixel coordinates
  const squareToPixel = (square: Square) => {
    // tsshogi coordinate system:
    // file: 0-8 (where 0 = file 9, 8 = file 1) - right to left
    // rank: 0-8 (where 0 = rank 1, 8 = rank 9) - top to bottom
    // Our board rendering uses colIndex (0-8) and rowIndex (0-8)
    // where colIndex goes left to right (0 = leftmost column = file 9)
    // and rowIndex goes top to bottom (0 = top row = rank 1)
    
    // Convert tsshogi coordinates to our board coordinates
    // Note: tsshogi file 0 = traditional file 9 (rightmost), file 8 = traditional file 1 (leftmost)
    const colIndex = square.file; // file 0 (9筋) -> col 0, file 8 (1筋) -> col 8
    const rowIndex = square.rank; // rank 0 (1段) -> row 0, rank 8 (9段) -> row 8
    
    const x = colIndex * 70 + 35 + 30; // 70px per square, center at 35px, move right by 30px
    const y = rowIndex * 76 + 38 + 171; // 76px per square, center at 38px, move down by 2.25 square heights
    return { x, y };
  };

  return (
    <div className={`shogi-board-container`}>
      <div className="column-numbers">
        {columnNumbers.map((num) => (
          <div key={num} className="column-number-cell">
            {num}
          </div>
        ))}
      </div>
      <div className="board-and-row-numbers">
        <div 
          className="board"
          style={boardBackground ? {
            backgroundImage: `url('${boardBackground}')`,
            backgroundSize: 'cover',
            backgroundRepeat: 'no-repeat',
            backgroundPosition: 'center center'
          } : undefined}
        >
          {Array.from({ length: 9 }, (_, rowIndex) => (
            <div key={rowIndex} className="board-row">
              {Array.from({ length: 9 }, (_, colIndex) => {
                // Convert row/col to tsshogi Square
                const square = Square.newByXY(colIndex, rowIndex); // tsshogi uses 0-8 coordinates
                const piece = square ? position.board.at(square) : null;
                
                const classNames = [
                  'board-square',
                  isSelected(rowIndex, colIndex) ? 'selected' : '',
                  isLegalMove(rowIndex, colIndex) ? 'legal-move' : '',
                  isLastMove(rowIndex, colIndex) ? 'last-move' : '',
                  isInCheckSquare(rowIndex, colIndex) ? 'in-check' : '',
                ].filter(Boolean).join(' ');

                return (
                  <div
                    key={colIndex}
                    data-testid={`square-${rowIndex}-${colIndex}`}
                    className={classNames}
                    onClick={() => onSquareClick(rowIndex, colIndex)}
                    onDragOver={(e) => {
                      e.preventDefault();
                      onDragOver?.(rowIndex, colIndex);
                    }}
                    onDrop={(e) => {
                      e.preventDefault();
                      onDragEnd?.(rowIndex, colIndex);
                    }}>
                    {piece && (
                      <PieceComponent
                        type={piece.type}
                        player={toOurPlayer(piece.color)}
                        pieceThemeType={pieceThemeType || 'kanji'}
                        isSelected={isSelected(rowIndex, colIndex)}
                        isAttacked={(() => {
                          const attacked = isSquareAttacked ? isSquareAttacked(square) : false;
                          if (attacked) {
                            console.log(`Piece at ${square.usi} is attacked`);
                          }
                          return attacked;
                        })()}
                        onClick={() => {
                          onSquareClick(rowIndex, colIndex)
                        }}
                        onDragStart={() => {
                          onDragStart?.(rowIndex, colIndex);
                        }}
                      />
                    )}
                    {/* Promotion zone intersection dots */}
                    {((rowIndex === 2 && (colIndex === 2 || colIndex === 5)) || 
                      (rowIndex === 5 && (colIndex === 2 || colIndex === 5))) && (
                      <div className="intersection-dot"></div>
                    )}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
        {/* Red lines for check indicators */}
        {isInCheck && kingInCheckSquare && attackingPieces && attackingPieces.length > 0 && (
          <svg className="check-line-svg" width="630" height="684">
            {attackingPieces.map((attackerSquare, index) => {
              const attackerPos = squareToPixel(attackerSquare);
              const kingPos = squareToPixel(kingInCheckSquare);
              
              
              return (
                <g key={index}>
                  <line
                    x1={attackerPos.x}
                    y1={attackerPos.y}
                    x2={kingPos.x}
                    y2={kingPos.y}
                    stroke="red"
                    strokeWidth="3"
                    strokeOpacity="0.8"
                  />
                  {/* Debug circles to show exact positions */}
                  <circle cx={attackerPos.x} cy={attackerPos.y} r="8" fill="blue" opacity="0.7" />
                  <circle cx={kingPos.x} cy={kingPos.y} r="8" fill="green" opacity="0.7" />
                </g>
              );
            })}
          </svg>
        )}
        <div className="row-numbers">
          {kifuRowLabels.map((label) => (
            <div key={label} className="row-number-cell">
              {label}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Board;
