import React from 'react';
import '../styles/shogi.css';

const Piece = ({ type, player }) => {
  return (
    <div
      className={`piece ${player}`}
      draggable="true"
    >
      {type}
    </div>
  );
};

export default Piece;
