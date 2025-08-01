import React from 'react';

const CheckmateModal = ({ winner, onDismiss, onNewGame }) => {
  return (
    <div className="modal-overlay">
      <div className="promotion-modal">
        <h2>Checkmate!</h2>
        <p>{winner === 'player1' ? 'Player 1' : 'Player 2'} wins by checkmate!</p>
        <button onClick={onDismiss}>Dismiss</button>
        <button onClick={onNewGame}>New Game</button>
      </div>
    </div>
  );
};

export default CheckmateModal;