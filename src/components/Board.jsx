import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const Board = ({ board, onSquareClick, onDragStart, onDrop, legalMoves }) => {
  const isLegalMove = (row, col) => {
    return legalMoves.some(move => move[0] === row && move[1] === col);
  };

  return (
    <div className="board">
      {board.map((row, rowIndex) => (
        <div key={rowIndex} className="board-row">
          {row.map((piece, colIndex) => (
            <div
              key={colIndex}
              className={`board-square ${isLegalMove(rowIndex, colIndex) ? 'legal-move' : ''}`}
              onClick={() => onSquareClick(rowIndex, colIndex)}
              onDragOver={(e) => e.preventDefault()} // Allow drop
              onDrop={() => onDrop(rowIndex, colIndex)}
            >
              {piece && (
                <Piece
                  type={piece.type}
                  player={piece.player}
                  onDragStart={() => onDragStart(rowIndex, colIndex)}
                />
              )}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};

export default Board;
