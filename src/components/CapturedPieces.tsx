import React from 'react';
import PieceComponent from './Piece';
import '../styles/shogi.css';
import type { Piece, Player, PieceType } from '../types';

const PIECE_ORDER: { [key in PieceType]?: number } = {
  'P': 1,
  'L': 2,
  'N': 3,
  'S': 4,
  'G': 5,
  'B': 6,
  'R': 7,
};

interface CapturedPiecesProps {
  pieces: Piece[];
  player: Player;
  onPieceClick: (type: PieceType) => void;
  onPieceDragStart: (type: PieceType) => void;
  pieceLabelType: string;
  selectedCapturedPiece: { type: PieceType } | null;
  isThinking: boolean;
  boardBackground: string;
  isGameOver: boolean;
}

const CapturedPieces: React.FC<CapturedPiecesProps> = ({ pieces, player, onPieceClick, onPieceDragStart, pieceLabelType, selectedCapturedPiece, isThinking, boardBackground, isGameOver }) => {
  const groupedPieces = pieces.reduce((acc, piece) => {
    acc[piece.type] = (acc[piece.type] || 0) + 1;
    return acc;
  }, {} as Record<PieceType, number>);

  const sortedPieces = Object.entries(groupedPieces).sort(([typeA], [typeB]) => {
    return (PIECE_ORDER[typeA as PieceType] ?? 8) - (PIECE_ORDER[typeB as PieceType] ?? 8);
  });

  return (
    <div className={`captured-pieces ${player}`} style={{ backgroundImage: `url(${boardBackground})` }}>
      <h3>
      {player === 'player1' ? '☗ ' : <><span style={{ color: "white" }}>☗</span>{' '}</>}
        {pieceLabelType === 'kanji'
          ? (player === 'player1' ? 'Sente' : 'Gote')
          : (player === 'player1' ? 'Black' : 'White')}      </h3>
      <div className={`pieces-list ${player}`}>
        {sortedPieces.map(([type, count]) => (
          <PieceComponent
            key={type}
            type={type as PieceType}
            player={player}
            onDragStart={isThinking || isGameOver ? undefined : () => onPieceDragStart(type as PieceType)}
            onClick={isThinking || isGameOver ? undefined : () => onPieceClick(type as PieceType)}
            pieceLabelType={pieceLabelType}
            count={count}
            isSelected={selectedCapturedPiece ? selectedCapturedPiece.type === type : false}
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;
