import React, { useState } from 'react';
import './SaveGameModal.css';

interface SaveGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (name: string) => void;
}

const SaveGameModal: React.FC<SaveGameModalProps> = ({ isOpen, onClose, onSave }) => {
  const [name, setName] = useState('');

  if (!isOpen) return null;

  const handleSave = () => {
    if (name) {
      onSave(name);
    }
  };

  return (
    <div className="save-game-modal-overlay">
      <div className="save-game-modal">
        <div className="save-game-modal-header">
          <h2>Save Game</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="save-game-modal-content">
          <div className="filename-input">
            <label htmlFor="filename-input">Game Name:</label>
            <input
              id="filename-input"
              type="text"
              value={name}
              onChange={(e) => setName(e.target.value)}
              placeholder="Enter a name for your game"
            />
          </div>
        </div>
        <div className="save-game-modal-footer">
          <button className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          <button className="save-button" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
};

export default SaveGameModal;