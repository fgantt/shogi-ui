import React, { useState } from 'react';
import SvgPiece from './SvgPiece';
import { getOppositeLabel, getEnglishName } from '../utils/pieceMaps';
import '../styles/shogi.css';
import type { PieceType } from 'tsshogi';

interface PieceProps {
  type: PieceType;
  player: 'player1' | 'player2';
  onDragStart?: () => void;
  onClick?: () => void;
  pieceThemeType: string;
  count?: number;
  isSelected?: boolean;
  isAttacked?: boolean;
  showTooltips?: boolean;
}

const Piece: React.FC<PieceProps> = ({ type, player, onDragStart, onClick, pieceThemeType, count, isSelected, isAttacked, showTooltips }) => {
  const [showTooltip, setShowTooltip] = useState(false);

  return (
    <div
      className={`piece ${isSelected ? 'selected' : ''}`}
      draggable="true"
      onDragStart={onDragStart}
      onClick={onClick}
      onMouseEnter={() => showTooltips && setShowTooltip(true)}
      onMouseLeave={() => showTooltips && setShowTooltip(false)}
    >
      <div className="piece-inner">
        <SvgPiece type={type} player={player} pieceThemeType={pieceThemeType} isSelected={isSelected} />
      </div>
      {count && count > 1 && <div className={`badge-counter ${player === 'player2' ? 'badge-counter-gote' : ''}`}>{count}</div>}
      {isAttacked && <div className={`badge-attacked badge-attacked-${player} ${player === 'player2' ? 'badge-attacked-gote' : ''}`}>!</div>}
      {showTooltips && showTooltip && (
        <div className="piece-tooltip">
          {getOppositeLabel(type, pieceThemeType)} - {getEnglishName(type)}
        </div>
      )}
    </div>
  );
};

export default Piece;
