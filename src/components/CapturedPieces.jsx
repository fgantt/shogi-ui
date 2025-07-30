import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const CapturedPieces = ({ pieces, player, onPieceClick, onPieceDragStart, pieceLabelType }) => {
  return (
    <div className={`captured-pieces ${player}`}>
      <h3>{player === 'player1' ? 'Player 1' : 'Player 2'} Captured</h3>
      <div className="pieces-list">
        {Object.entries(pieces.reduce((acc, piece) => {
          acc[piece.type] = (acc[piece.type] || 0) + 1;
          return acc;
        }, {})).map(([type, count]) => (
          <Piece
            key={type}
            type={type}
            player={player}
            onDragStart={() => onPieceDragStart(type)}
            onClick={() => onPieceClick(type)}
            pieceLabelType={pieceLabelType}
            count={count}
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;
