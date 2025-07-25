import React from 'react';
import SvgPiece from './SvgPiece';
import '../styles/shogi.css';

const Piece = ({ type, player, onDragStart, onClick, pieceLabelType }) => {
  return (
    <div
      className={`piece ${player}`}
      draggable="true"
      onDragStart={onDragStart}
      onClick={onClick}
    >
      <SvgPiece type={type} player={player} pieceLabelType={pieceLabelType} />
    </div>
  );
};

export default Piece;
