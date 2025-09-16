import React from 'react';
import { GameSettings } from '../types';

interface StartGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartGame: (settings: GameSettings) => void;
}

const StartGameModal: React.FC<StartGameModalProps> = ({ isOpen, onClose, onStartGame }) => {
  if (!isOpen) return null;

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const settings: GameSettings = {
      difficulty: formData.get('difficulty') as GameSettings['difficulty'],
      player1Type: formData.get('player1Type') as GameSettings['player1Type'],
      player2Type: formData.get('player2Type') as GameSettings['player2Type'],
    };
    onStartGame(settings);
    onClose();
  };

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>New Game</h2>
        <form onSubmit={handleSubmit}>
          <section>
            <h3>Difficulty</h3>
            <div className="setting-group">
              <select id="difficulty" name="difficulty" defaultValue="medium">
                <option value="easy">Easy</option>
                <option value="medium">Medium</option>
                <option value="hard">Hard</option>
              </select>
            </div>
          </section>
          <section>
            <h3>Player 1 (Black)</h3>
            <div className="setting-group">
              <select id="player1Type" name="player1Type" defaultValue="human">
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
            </div>
          </section>
          <section>
            <h3>Player 2 (White)</h3>
            <div className="setting-group">
              <select id="player2Type" name="player2Type" defaultValue="ai">
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
            </div>
          </section>
          <button type="submit">Start Game</button>
        </form>
      </div>
    </div>
  );
};

export default StartGameModal;
