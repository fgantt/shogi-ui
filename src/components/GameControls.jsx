import React from 'react';
import '../styles/shogi.css';

const GameControls = ({ onNewGame, onUndoMove, onDifficultyChange, onPieceLabelTypeChange, pieceLabelType }) => {
  return (
    <div className="game-controls">
      <button onClick={onNewGame}>New Game</button>
      <button onClick={onUndoMove}>Undo Move</button>
      <select onChange={(e) => onDifficultyChange(e.target.value)}>
        <option value="easy">Easy</option>
        <option value="medium">Medium</option>
        <option value="hard">Hard</option>
      </select>
      <select onChange={(e) => onPieceLabelTypeChange(e.target.value)} value={pieceLabelType}>
        <option value="kanji">Kanji Labels</option>
        <option value="english">English Labels</option>
      </select>
    </div>
  );
};

export default GameControls;
