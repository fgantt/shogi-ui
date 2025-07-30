import React from 'react';
import '../styles/shogi.css';

const GameControls = ({ onNewGame, onUndoMove, onDifficultyChange, onPieceLabelTypeChange, pieceLabelType, onOpenSettings }) => {
  return (
    <div className="game-controls">
      <button onClick={onNewGame}>New Game</button>
      <button onClick={onUndoMove}>Undo Move</button>
      
      <button onClick={onOpenSettings}>Settings</button>
    </div>
  );
};

export default GameControls;
