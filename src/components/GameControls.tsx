import React from 'react';
import '../styles/shogi.css';

interface GameControlsProps {
  onNewGame: () => void;
  onOpenSettings: () => void;
  onOpenSaveModal: () => void;
  onOpenLoadModal: () => void;
  onCyclePieceTheme: () => void;
  onCycleBoardBackground: () => void;
}

const GameControls: React.FC<GameControlsProps> = ({ onNewGame, onOpenSettings, onOpenSaveModal, onOpenLoadModal, onCyclePieceTheme, onCycleBoardBackground }) => {
  return (
    <div className="game-controls">
      <div className="game-controls-left">
        <button onClick={onNewGame} className="new-game-btn">
          <span role="img" aria-label="New Game">🔄</span> New Game
        </button>
        <button onClick={onOpenSaveModal}>
          <span role="img" aria-label="Save Game">💾</span> Save Game
        </button>
        <button onClick={onOpenLoadModal}>
          <span role="img" aria-label="Load Game">📂</span> Load Game
        </button>
      </div>
      <div className="game-controls-right">
        <button onClick={onCyclePieceTheme}>
          <span role="img" aria-label="Cycle Piece Theme">♟️</span> Pieces
        </button>
        <button onClick={onCycleBoardBackground}>
          <span role="img" aria-label="Cycle Board Background">🎨</span> Board
        </button>
        <button onClick={onOpenSettings}>
          <span role="img" aria-label="Settings">⚙️</span> Settings
        </button>
      </div>
    </div>
  );
};

export default GameControls;
