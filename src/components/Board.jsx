import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const Board = ({ board, onSquareClick, onDragStart, onDrop, legalMoves, legalDropSquares, isCheck, kingPosition, lastMove, pieceLabelType }) => {
  console.log("Board received legalMoves:", legalMoves);
  console.log("Board received legalDropSquares:", legalDropSquares);
  const isLegalMove = (row, col) => {
    return legalMoves.some(move => move[0] === row && move[1] === col);
  };

  const isLegalDropSquare = (row, col) => {
    return legalDropSquares.some(square => square[0] === row && square[1] === col);
  };

  const isKingSquare = (row, col) => {
    return kingPosition && kingPosition[0] === row && kingPosition[1] === col;
  };

  const isLastMoveSquare = (row, col) => {
    if (!lastMove) return false;
    const { from, to } = lastMove;
    return (from[0] === row && from[1] === col) || (to[0] === row && to[1] === col);
  };

  const columnNumbers = [1, 2, 3, 4, 5, 6, 7, 8, 9];
  const rowNumbers = [1, 2, 3, 4, 5, 6, 7, 8, 9];

  return (
    <div className="shogi-board-container">
      <div className="column-numbers">
        {columnNumbers.map((num, index) => (
          <div key={index} className="column-number-cell">
            {num}
          </div>
        ))}
      </div>
      <div className="board-and-row-numbers">
        <div className="row-numbers">
          {rowNumbers.map((num, index) => (
            <div key={index} className="row-number-cell">
              {num}
            </div>
          ))}
        </div>
        <div className="board">
          {board.map((row, rowIndex) => (
            <div key={rowIndex} className="board-row">
              {row.map((piece, colIndex) => (
                <div
                  key={colIndex}
                  className={`board-square ${isLegalMove(rowIndex, colIndex) ? 'legal-move' : ''} ${isLegalDropSquare(rowIndex, colIndex) ? 'legal-move' : ''} ${isKingSquare(rowIndex, colIndex) && isCheck ? 'in-check' : ''} ${isLastMoveSquare(rowIndex, colIndex) ? 'last-move' : ''}`}
                  onClick={() => onSquareClick(rowIndex, colIndex)}
                  onDragOver={(e) => e.preventDefault()} // Allow drop
                  onDrop={() => onDrop(rowIndex, colIndex)}
                >
                  {piece && (
                    <Piece
                      type={piece.type}
                      player={piece.player}
                      onDragStart={() => onDragStart(rowIndex, colIndex)}
                      pieceLabelType={pieceLabelType}
                    />
                  )}
                </div>
              ))}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Board;
