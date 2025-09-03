import React from 'react';
import './LoadGameModal.css';

interface LoadGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onLoad: (name: string) => void;
  onDelete: (name: string) => void;
  savedGames: { [key: string]: string };
}

const LoadGameModal: React.FC<LoadGameModalProps> = ({ isOpen, onClose, onLoad, onDelete, savedGames }) => {
  if (!isOpen) return null;

  return (
    <div className="load-game-modal-overlay">
      <div className="load-game-modal">
        <div className="load-game-modal-header">
          <h2>Load Game</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>
        <div className="load-game-modal-content">
          {Object.keys(savedGames).length === 0 ? (
            <p>No saved games found.</p>
          ) : (
            <ul className="saved-games-list">
              {Object.keys(savedGames).map((name) => (
                <li key={name} className="saved-game-item">
                  <span>{name}</span>
                  <div className="buttons">
                    <button onClick={() => onLoad(name)}>Load</button>
                    <button onClick={() => onDelete(name)}>Delete</button>
                  </div>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="load-game-modal-footer">
          <button className="cancel-button" onClick={onClose}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
};

export default LoadGameModal;