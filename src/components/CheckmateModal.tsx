import React from 'react';

interface CheckmateModalProps {
  winner: 'player1' | 'player2' | 'draw' | null;
  onDismiss: () => void;
  onNewGame: () => void;
}

const CheckmateModal: React.FC<CheckmateModalProps> = ({ winner, onDismiss, onNewGame }) => {
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
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>{title}</h2>
        <p>{message}</p>
        <p className="game-over-note">The game is now over. You can review the final position or start a new game.</p>
        <div className="checkmate-modal-buttons">
          <button onClick={onNewGame}>New Game</button>
          <button onClick={onDismiss}>Review Position</button>
        </div>
      </div>
    </div>
  );
};

export default CheckmateModal;