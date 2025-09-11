import React from 'react';
import { PieceType as TsshogiPieceType, Hand } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

// Order for sorting pieces (lower number = higher priority)
const PIECE_ORDER: { [key in TsshogiPieceType]: number } = {
  [TsshogiPieceType.PAWN]: 0,
  [TsshogiPieceType.LANCE]: 1,
  [TsshogiPieceType.KNIGHT]: 2,
  [TsshogiPieceType.SILVER]: 3,
  [TsshogiPieceType.GOLD]: 4,
  [TsshogiPieceType.BISHOP]: 5,
  [TsshogiPieceType.ROOK]: 6,
  [TsshogiPieceType.KING]: 7,
  [TsshogiPieceType.PROM_PAWN]: 8,
  [TsshogiPieceType.PROM_LANCE]: 9,
  [TsshogiPieceType.PROM_KNIGHT]: 10,
  [TsshogiPieceType.PROM_SILVER]: 11,
  [TsshogiPieceType.HORSE]: 12,
  [TsshogiPieceType.DRAGON]: 13,
};

interface CapturedPiecesProps {
  captured: Hand;
  player: 'player1' | 'player2';
  onPieceClick?: (type: TsshogiPieceType) => void;
  selectedCapturedPiece?: TsshogiPieceType | null;
  isAttacked?: boolean;
  boardBackground?: string;
  pieceLabelType?: 'kanji' | 'english';
}

const CapturedPieces: React.FC<CapturedPiecesProps> = ({ captured, player, onPieceClick, selectedCapturedPiece, isAttacked, boardBackground, pieceLabelType }) => {
  const pieces = captured.counts
    .filter(({ count }) => count > 0)
    .map(({ type, count }) => ({ type, count }));

  const sortedPieces = pieces.sort(({ type: typeA }, { type: typeB }) => {
    return PIECE_ORDER[typeA] - PIECE_ORDER[typeB];
  });

  return (
    <div 
      className={`captured-pieces ${player}`}
      style={boardBackground ? {
        backgroundImage: `url('${boardBackground}')`,
        backgroundSize: 'cover',
        backgroundRepeat: 'no-repeat',
        backgroundPosition: 'center center'
      } : undefined}
    >
      <h3>
        <span style={{ color: player === 'player1' ? 'black' : 'white' }}>☗ </span>
        {player === 'player1' ? 'Sente' : 'Gote'}
      </h3>
      <div className={`pieces-list ${player}`}>
        {sortedPieces.map(({ type, count }) => (
          <PieceComponent
            key={type}
            type={type}
            player={player}
            onClick={() => onPieceClick && onPieceClick(type)}
            pieceLabelType={pieceLabelType || 'kanji'}
            count={count}
            isSelected={selectedCapturedPiece === type}
            isAttacked={isAttacked || false}
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;