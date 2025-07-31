import React from 'react';
import SvgPiece from './SvgPiece';
import '../styles/shogi.css';

const Piece = ({ type, player, onDragStart, onClick, pieceLabelType, count, isSelected, isAttacked }) => {
  return (
    <div
      className={`piece ${player} ${isSelected ? 'selected' : ''}`}
      draggable="true"
      onDragStart={onDragStart}
      onClick={onClick}
    >
      <SvgPiece type={type} player={player} pieceLabelType={pieceLabelType} />
      {count > 1 && <div className="badge-counter">{count}</div>}
      {isAttacked && <div className="badge-attacked">!</div>}
    </div>
  );
};

export default Piece;
