import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const Board = ({ board, onSquareClick, onDragStart, onDrop, legalMoves, legalDropSquares, isCheck, kingPosition, lastMove, pieceLabelType, selectedPiece, attackedSquares, showAttackedPieces, showPieceTooltips, isThinking }) => {
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

    // Check if 'to' is a valid coordinate
    const isToSquare = Array.isArray(to) && to.length === 2 && to[0] === row && to[1] === col;

    // Check if 'from' is a valid coordinate (and not a drop)
    const isFromSquare = Array.isArray(from) && from.length === 2 && from[0] === row && from[1] === col;

    return isFromSquare || isToSquare;
  };

  const columnNumbers = [9, 8, 7, 6, 5, 4, 3, 2, 1];
  const rowNumbers = [1, 2, 3, 4, 5, 6, 7, 8, 9];

  return (
    <div className={`shogi-board-container ${isThinking ? 'ai-thinking-overlay' : ''}`}>
      <div className="column-numbers">
        {columnNumbers.map((num, index) => (
          <div key={index} className="column-number-cell">
            {num}
          </div>
        ))}
      </div>
      <div className="board-and-row-numbers">
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
                      isSelected={selectedPiece && selectedPiece.row === rowIndex && selectedPiece.col === colIndex}
                      isAttacked={showAttackedPieces && attackedSquares[piece.player === 'player1' ? 'player2' : 'player1'].has(`${rowIndex},${colIndex}`)}
                      showTooltips={showPieceTooltips}
                    />
                  )}
                </div>
              ))}
            </div>
          ))}
        </div>
        <div className="row-numbers">
          {rowNumbers.map((num, index) => (
            <div key={index} className="row-number-cell">
              {num}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
};

export default Board;
