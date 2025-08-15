import React from 'react';
import '../styles/shogi.css';

interface GameControlsProps {
  onNewGame: () => void;
  onUndoMove: () => void;
  onOpenSettings: () => void;
  aiDifficulty: string | null;
  isThinking: boolean;
  isGameOver: boolean;
}

const GameControls: React.FC<GameControlsProps> = ({ onNewGame, onUndoMove, onOpenSettings, aiDifficulty, isThinking, isGameOver }) => {
  return (
    <div className="game-controls">
      <div className="game-controls-left">
        <button onClick={onNewGame} className="new-game-btn">
          <span role="img" aria-label="New Game">🔄</span> New Game
        </button>
        <button onClick={onUndoMove} className={isThinking ? 'thinking' : ''} disabled={isGameOver}>
          <span role="img" aria-label="Undo Move">↩️</span> Undo Move
        </button>
      </div>
      <div className="difficulty-display">
        {aiDifficulty && `Level: ${aiDifficulty.charAt(0).toUpperCase() + aiDifficulty.slice(1)}`}
      </div>
      <div className="game-controls-right">
        <button onClick={onOpenSettings} className={isThinking ? 'thinking' : ''}>
          <span role="img" aria-label="Settings">⚙️</span> Settings
        </button>
      </div>
    </div>
  );
};

export default GameControls;
