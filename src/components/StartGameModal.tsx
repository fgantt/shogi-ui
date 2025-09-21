import React, { useState } from 'react';
import { GameSettings } from '../types';

interface StartGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartGame: (settings: GameSettings) => void;
}

const StartGameModal: React.FC<StartGameModalProps> = ({ isOpen, onClose, onStartGame }) => {
  const [player1Type, setPlayer1Type] = useState<'human' | 'ai'>('human');
  const [player2Type, setPlayer2Type] = useState<'human' | 'ai'>('ai');

  if (!isOpen) return null;

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const formData = new FormData(event.currentTarget);
    const settings: GameSettings = {
      player1Type: formData.get('player1Type') as GameSettings['player1Type'],
      player2Type: formData.get('player2Type') as GameSettings['player2Type'],
      player1Level: parseInt(formData.get('player1Level') as string, 10) || 5,
      player2Level: parseInt(formData.get('player2Level') as string, 10) || 5,
      minutesPerSide: parseInt(formData.get('minutesPerSide') as string, 10) || 30,
      byoyomiInSeconds: parseInt(formData.get('byoyomiInSeconds') as string, 10) || 10,
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
            <h3>Player 1 (Black)</h3>
            <div className="setting-group">
              <select id="player1Type" name="player1Type" defaultValue="human" onChange={(e) => setPlayer1Type(e.target.value as 'human' | 'ai')}>
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
              {player1Type === 'ai' && (
                <div className="setting-group">
                  <label htmlFor="player1Level">Level (1-8)</label>
                  <input id="player1Level" name="player1Level" type="number" min="1" max="8" defaultValue="5" />
                </div>
              )}
            </div>
          </section>
          <section>
            <h3>Player 2 (White)</h3>
            <div className="setting-group">
              <select id="player2Type" name="player2Type" defaultValue="ai" onChange={(e) => setPlayer2Type(e.target.value as 'human' | 'ai')}>
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
              {player2Type === 'ai' && (
                <div className="setting-group">
                  <label htmlFor="player2Level">Level (1-8)</label>
                  <input id="player2Level" name="player2Level" type="number" min="1" max="8" defaultValue="5" />
                </div>
              )}
            </div>
          </section>
          <section>
            <h3>Time Controls</h3>
            <div className="setting-group">
              <label htmlFor="minutesPerSide">Minutes per side</label>
              <input id="minutesPerSide" name="minutesPerSide" type="number" min="1" defaultValue="30" />
            </div>
            <div className="setting-group">
              <label htmlFor="byoyomiInSeconds">Byoyomi in seconds</label>
              <input id="byoyomiInSeconds" name="byoyomiInSeconds" type="number" min="0" defaultValue="10" />
            </div>
          </section>
          <button type="submit">Start Game</button>
        </form>
      </div>
    </div>
  );
};

export default StartGameModal;
