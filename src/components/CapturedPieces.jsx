import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const CapturedPieces = ({ pieces, player, onPieceClick, onPieceDragStart }) => {
  return (
    <div className={`captured-pieces ${player}`}>
      <h3>{player === 'player1' ? 'Player 1' : 'Player 2'} Captured</h3>
      <div className="pieces-list">
        {pieces.map((piece, index) => (
          <Piece
            key={index}
            type={piece.type}
            player={piece.player}
            onDragStart={() => onPieceDragStart(piece.type)} // Pass piece type for drag
            onClick={() => onPieceClick(piece.type)} // Pass piece type for click
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;
