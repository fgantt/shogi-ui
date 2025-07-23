import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const Board = ({ board }) => {
  return (
    <div className="board">
      {board.map((row, rowIndex) => (
        <div key={rowIndex} className="board-row">
          {row.map((piece, colIndex) => (
            <div key={colIndex} className="board-square">
              {piece && <Piece type={piece.type} player={piece.player} />}
            </div>
          ))}
        </div>
      ))}
    </div>
  );
};

export default Board;
