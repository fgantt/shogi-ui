import React from 'react';
import { ImmutablePosition, Square } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

interface BoardProps {
  position: ImmutablePosition;
  onSquareClick: (row: number, col: number) => void;
  selectedSquare: Square | null;
  legalMoves: Square[];
  lastMove: { from: Square | null; to: Square | null } | null;
  isSquareAttacked?: (square: Square) => boolean;
}



// Helper to map tsshogi color to our Player type
function toOurPlayer(color: string): 'player1' | 'player2' {
    return color === 'black' ? 'player1' : 'player2';
}

const Board: React.FC<BoardProps> = ({ position, onSquareClick, selectedSquare, legalMoves, lastMove, isSquareAttacked }) => {
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
        <div className="board">
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
                ].filter(Boolean).join(' ');

                return (
                  <div
                    key={colIndex}
                    data-testid={`square-${rowIndex}-${colIndex}`}
                    className={classNames}
                    onClick={() => onSquareClick(rowIndex, colIndex)}>
                    {piece && (
                      <PieceComponent
                        type={piece.type}
                        player={toOurPlayer(piece.color)}
                        pieceLabelType={'kanji'} // Hardcoded for now
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
                      />
                    )}
                  </div>
                );
              })}
            </div>
          ))}
        </div>
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
