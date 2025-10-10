import React from 'react';

interface CheckmateModalProps {
  winner: 'player1' | 'player2' | 'draw' | null;
  endgameType?: 'checkmate' | 'resignation' | 'repetition' | 'stalemate' | 'illegal' | 'no_moves';
  details?: string;
  onDismiss: () => void;
  onNewGame: () => void;
}

const CheckmateModal: React.FC<CheckmateModalProps> = ({ 
  winner, 
  endgameType = 'checkmate',
  details,
  onDismiss, 
  onNewGame 
}) => {
  let title, message, emoji;

  if (winner === 'draw') {
    title = "Draw";
    emoji = "🤝";
    
    switch (endgameType) {
      case 'repetition':
        message = "The game is a draw by four-fold repetition (Sennichite / 千日手).";
        break;
      default:
        message = "The game is a draw.";
    }
  } else if (winner) {
    const winnerName = winner === 'player1' ? 'Sente (Player 1)' : 'Gote (Player 2)';
    const loserName = winner === 'player1' ? 'Gote (Player 2)' : 'Sente (Player 1)';
    
    switch (endgameType) {
      case 'checkmate':
        title = "Checkmate!";
        emoji = "👑";
        message = `${winnerName} wins by checkmate (Tsumi / 詰み)!`;
        break;
      case 'resignation':
        title = "Resignation";
        emoji = "🏳️";
        message = `${loserName} has resigned. ${winnerName} wins!`;
        break;
      case 'stalemate':
      case 'no_moves':
        title = "No Legal Moves";
        emoji = "🚫";
        message = `${loserName} has no legal moves. In Shogi, this counts as a loss. ${winnerName} wins!`;
        break;
      case 'illegal':
        title = "Illegal Move";
        emoji = "⚠️";
        message = `${loserName} made an illegal move. ${winnerName} wins!`;
        break;
      default:
        title = "Game Over";
        emoji = "🎌";
        message = `${winnerName} wins!`;
    }
  } else {
    return null; // Should not happen
  }

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <div style={{ fontSize: '48px', textAlign: 'center', marginBottom: '16px' }}>{emoji}</div>
        <h2 style={{ textAlign: 'center' }}>{title}</h2>
        <p style={{ textAlign: 'center', fontSize: '16px', margin: '16px 0' }}>{message}</p>
        {details && <p style={{ textAlign: 'center', fontSize: '14px', color: '#666', margin: '8px 0' }}>{details}</p>}
        <p className="game-over-note" style={{ textAlign: 'center', marginTop: '16px' }}>
          The game is now over. You can review the final position or start a new game.
        </p>
        <div className="checkmate-modal-buttons" style={{ display: 'flex', gap: '12px', justifyContent: 'center', marginTop: '20px' }}>
          <button onClick={onNewGame}>New Game</button>
          <button onClick={onDismiss}>Review Position</button>
        </div>
      </div>
    </div>
  );
};

export default CheckmateModal;