import React, { useState } from 'react';
import SvgPiece from './SvgPiece';
import { getOppositeLabel, getEnglishName } from '../utils/pieceMaps';
import '../styles/shogi.css';

const Piece = ({ type, player, onDragStart, onClick, pieceLabelType, count, isSelected, isAttacked, showTooltips }) => {
  const [showTooltip, setShowTooltip] = useState(false);

  return (
    <div
      className={`piece ${player} ${isSelected ? 'selected' : ''}`}
      draggable="true"
      onDragStart={onDragStart}
      onClick={onClick}
      onMouseEnter={() => showTooltips && setShowTooltip(true)}
      onMouseLeave={() => showTooltips && setShowTooltip(false)}
    >
      <SvgPiece type={type} player={player} pieceLabelType={pieceLabelType} />
      {count > 1 && <div className="badge-counter">{count}</div>}
      {isAttacked && <div className={`badge-attacked badge-attacked-${player}`}>!</div>}
      {showTooltips && showTooltip && (
        <div className="piece-tooltip">
          {getOppositeLabel(type, pieceLabelType)} - {getEnglishName(type)}
        </div>
      )}
    </div>
  );
};

export default Piece;