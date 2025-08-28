import React, { useState } from 'react';
import { SUPPORTED_FORMATS, SupportedFormat, generateShogiFile } from '../game/shogi';
import { GameState } from '../types';
import './SaveGameModal.css';

interface SaveGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  gameState: GameState;
}

const SaveGameModal: React.FC<SaveGameModalProps> = ({ isOpen, onClose, gameState }) => {
  const [selectedFormat, setSelectedFormat] = useState<SupportedFormat>('kif');
  const [filename, setFilename] = useState('');

  if (!isOpen) return null;

  const handleSave = () => {
    try {
      const content = generateShogiFile(gameState, selectedFormat);
      const format = SUPPORTED_FORMATS[selectedFormat];
      
      // Create blob and download
      const blob = new Blob([content], { type: format.mimeType });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = filename || `shogi-game${format.extension}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);
      
      onClose();
    } catch (error) {
      console.error('Error saving game:', error);
      alert('Failed to save game. Please try again.');
    }
  };

  const handleClose = () => {
    setSelectedFormat('kif');
    setFilename('');
    onClose();
  };

  return (
    <div className="save-game-modal-overlay">
      <div className="save-game-modal">
        <div className="save-game-modal-header">
          <h2>Save Game</h2>
          <button className="close-button" onClick={handleClose}>
            ×
          </button>
        </div>
        
        <div className="save-game-modal-content">
          <div className="format-selection">
            <label htmlFor="format-select">File Format:</label>
            <select
              id="format-select"
              value={selectedFormat}
              onChange={(e) => setSelectedFormat(e.target.value as SupportedFormat)}
            >
              {Object.entries(SUPPORTED_FORMATS).map(([key, format]) => (
                <option key={key} value={key}>
                  {format.name} ({format.extension})
                </option>
              ))}
            </select>
          </div>
          
          <div className="filename-input">
            <label htmlFor="filename-input">Filename (optional):</label>
            <input
              id="filename-input"
              type="text"
              value={filename}
              onChange={(e) => setFilename(e.target.value)}
              placeholder={`shogi-game${SUPPORTED_FORMATS[selectedFormat].extension}`}
            />
          </div>
          
          <div className="format-info">
            <p>
              <strong>Selected format:</strong> {SUPPORTED_FORMATS[selectedFormat].name}
            </p>
            <p>
              <strong>File extension:</strong> {SUPPORTED_FORMATS[selectedFormat].extension}
            </p>
            <p>
              <strong>Description:</strong> {getFormatDescription(selectedFormat)}
            </p>
          </div>
        </div>
        
        <div className="save-game-modal-footer">
          <button className="cancel-button" onClick={handleClose}>
            Cancel
          </button>
          <button className="save-button" onClick={handleSave}>
            Save Game
          </button>
        </div>
      </div>
    </div>
  );
};

function getFormatDescription(format: SupportedFormat): string {
  switch (format) {
    case 'kif':
      return 'Traditional Japanese KIF format, widely supported by Shogi software';
    case 'ki2':
      return 'Compact Japanese KI2 format, good for sharing and analysis';
    case 'csa':
      return 'CSA format, commonly used in Japanese Shogi servers';
    case 'jkf':
      return 'JSON-based format, good for programmatic access';
    case 'sfen':
      return 'SFEN notation, compact representation of board position';
    default:
      return 'Unknown format';
  }
}

export default SaveGameModal;
