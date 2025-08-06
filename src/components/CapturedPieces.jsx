import React from 'react';
import Piece from './Piece';
import '../styles/shogi.css';

const PIECE_ORDER = {
  'P': 1, // Pawn
  'L': 2, // Lance
  'N': 3, // Knight
  'S': 4, // Silver
  'G': 5, // Gold
  'B': 6, // Bishop
  'R': 7, // Rook
};

const CapturedPieces = ({ pieces, player, onPieceClick, onPieceDragStart, pieceLabelType, selectedCapturedPiece, isThinking, boardBackground }) => {
  const groupedPieces = pieces.reduce((acc, piece) => {
    acc[piece.type] = (acc[piece.type] || 0) + 1;
    return acc;
  }, {});

  const sortedPieces = Object.entries(groupedPieces).sort(([typeA], [typeB]) => {
    return PIECE_ORDER[typeA] - PIECE_ORDER[typeB];
  });

  return (
    <div className={`captured-pieces ${player}`} style={{ backgroundImage: `url('${boardBackground}')` }}>
      <h3>
        {player === 'player1' ? '☗ ' : <><span style={{ color: "white" }}>☗</span>{' '}</>}
        {pieceLabelType === 'kanji'
          ? (player === 'player1' ? 'Sente' : 'Gote')
          : (player === 'player1' ? 'Black' : 'White')}
      </h3>
      <div className={`pieces-list ${player}`}>
        {sortedPieces.map(([type, count]) => (
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
