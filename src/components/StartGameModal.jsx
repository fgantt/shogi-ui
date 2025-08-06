import React from 'react';

const StartGameModal = ({ isOpen, onClose, onStartGame }) => {
  if (!isOpen) return null;

  const handleSubmit = (event) => {
    event.preventDefault();
    const formData = new FormData(event.target);
    const settings = {
      difficulty: formData.get('difficulty'),
      player1Type: formData.get('player1Type'),
      player2Type: formData.get('player2Type'),
      pieceSet: formData.get('pieceSet'),
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
            <h3>Player 1</h3>
            <div className="setting-group">
              <select id="player1Type" name="player1Type" defaultValue="human">
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
            </div>
          </section>
          <section>
            <h3>Player 2</h3>
            <div className="setting-group">
              <select id="player2Type" name="player2Type" defaultValue="ai">
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
            </div>
          </section>
          <section>
            <h3>Piece Set</h3>
            <div className="setting-group">
              <select id="pieceSet" name="pieceSet" defaultValue="kanji">
                <option value="kanji">Kanji</option>
                <option value="international">International</option>
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
