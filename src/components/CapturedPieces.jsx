import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const CapturedPieces = ({ pieces, player, onPieceClick, onPieceDragStart, pieceLabelType, selectedCapturedPiece }) => {
  return (
    <div className={`captured-pieces ${player} ${isThinking ? 'thinking' : ''}`}>
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
            onDragStart={isThinking ? null : () => onPieceDragStart(type)}
            onClick={isThinking ? null : () => onPieceClick(type)}
            pieceLabelType={pieceLabelType}
            count={count}
            isSelected={selectedCapturedPiece && selectedCapturedPiece.type === type}
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;
