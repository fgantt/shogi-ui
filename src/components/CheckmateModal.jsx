import React from 'react';

const CheckmateModal = ({ winner, onDismiss, onNewGame }) => {
  let title, message;

  if (winner === 'draw') {
    title = "Draw";
    message = "The game is a draw by repetition (Sennichite).";
  } else if (winner) {
    title = "Checkmate!";
    message = `${winner === 'player1' ? 'Player 1' : 'Player 2'} wins by checkmate!`;
  } else {
    return null; // Should not happen
  }

  return (
    <div className="modal-overlay">
      <div className="promotion-modal">
        <h2>{title}</h2>
        <p>{message}</p>
        <button onClick={onDismiss}>Dismiss</button>
        <button onClick={onNewGame}>New Game</button>
      </div>
    </div>
  );
};

export default CheckmateModal;