import React from 'react';
import { PieceType as TsshogiPieceType } from 'tsshogi';
import PieceComponent from './Piece';
import '../styles/shogi.css';

// Mapping from tsshogi PieceType to our component's PieceType
const pieceTypeMapping: { [key in TsshogiPieceType]?: string } = {
  PAWN: 'pawn',
  LANCE: 'lance',
  KNIGHT: 'knight',
  SILVER: 'silver',
  GOLD: 'gold',
  BISHOP: 'bishop',
  ROOK: 'rook',
};

const PIECE_ORDER: { [key: string]: number } = {
  pawn: 1,
  lance: 2,
  knight: 3,
  silver: 4,
  gold: 5,
  bishop: 6,
  rook: 7,
};

interface CapturedPiecesProps {
  captured: { [key in TsshogiPieceType]?: number };
  player: 'player1' | 'player2';
  onPieceClick?: (type: TsshogiPieceType) => void;
}

const CapturedPieces: React.FC<CapturedPiecesProps> = ({ captured, player, onPieceClick }) => {
  const pieces = Object.entries(captured)
    .filter(([, count]) => count && count > 0)
    .map(([type, count]) => ({ type: type as TsshogiPieceType, count }));

  const sortedPieces = pieces.sort(([typeA], [typeB]) => {
    const mappedTypeA = pieceTypeMapping[typeA] ?? '';
    const mappedTypeB = pieceTypeMapping[typeB] ?? '';
    return (PIECE_ORDER[mappedTypeA] ?? 8) - (PIECE_ORDER[mappedTypeB] ?? 8);
  });

  return (
    <div className={`captured-pieces ${player}`}>
      <h3>{player === 'player1' ? 'Sente' : 'Gote'}</h3>
      <div className={`pieces-list ${player}`}>
        {sortedPieces.map(({ type, count }) => (
          <PieceComponent
            key={type}
            type={pieceTypeMapping[type] ?? ''}
            player={player}
            onClick={() => onPieceClick && onPieceClick(type)}
            pieceLabelType={'kanji'} // Hardcoded for now
            count={count}
          />
        ))}
      </div>
    </div>
  );
};

export default CapturedPieces;