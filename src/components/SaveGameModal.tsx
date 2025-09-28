import React, { useState, useEffect } from 'react';
import './SaveGameModal.css';
import { GameFormat, GameData, generateGame, detectFormat } from '../utils/gameFormats';

interface SaveGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSave: (name: string, format: GameFormat) => void;
  gameData: GameData | null;
}

const SaveGameModal: React.FC<SaveGameModalProps> = ({ isOpen, onClose, onSave, gameData }) => {
  const [name, setName] = useState('');
  const [format, setFormat] = useState<GameFormat>('sfen');
  const [exportText, setExportText] = useState('');
  const [activeTab, setActiveTab] = useState<'save' | 'export'>('save');

  useEffect(() => {
    if (gameData && isOpen) {
      try {
        const generated = generateGame(gameData, format);
        setExportText(generated);
      } catch (error) {
        setExportText(`Error generating ${format.toUpperCase()} format: ${error}`);
      }
    }
  }, [gameData, format, isOpen]);

  const handleSave = () => {
    if (name) {
      onSave(name, format);
    }
  };

  const handleCopyToClipboard = async () => {
    try {
      await navigator.clipboard.writeText(exportText);
      // Could add a toast notification here
      alert('Copied to clipboard!');
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  };

  const handleFormatChange = (newFormat: GameFormat) => {
    setFormat(newFormat);
    if (gameData) {
      try {
        const generated = generateGame(gameData, newFormat);
        setExportText(generated);
      } catch (error) {
        setExportText(`Error generating ${newFormat.toUpperCase()} format: ${error}`);
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div className="save-game-modal-overlay">
      <div className="save-game-modal">
        <div className="save-game-modal-header">
          <h2>Save Game</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>
        
        <div className="save-game-modal-tabs">
          <button 
            className={`tab-button ${activeTab === 'save' ? 'active' : ''}`}
            onClick={() => setActiveTab('save')}
          >
            Save to Storage
          </button>
          <button 
            className={`tab-button ${activeTab === 'export' ? 'active' : ''}`}
            onClick={() => setActiveTab('export')}
          >
            Export/Copy
          </button>
        </div>

        <div className="save-game-modal-content">
          {activeTab === 'save' && (
            <>
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
              <div className="format-selector">
                <label htmlFor="format-select">Save Format:</label>
                <select
                  id="format-select"
                  value={format}
                  onChange={(e) => setFormat(e.target.value as GameFormat)}
                >
                  <option value="sfen">SFEN (Position Only)</option>
                  <option value="csa">CSA (Computer Shogi Association)</option>
                  <option value="kif">KIF (Japanese Format)</option>
                  <option value="json">JSON (Full Game Data)</option>
                </select>
              </div>
            </>
          )}

          {activeTab === 'export' && (
            <>
              <div className="format-selector">
                <label htmlFor="export-format-select">Export Format:</label>
                <select
                  id="export-format-select"
                  value={format}
                  onChange={(e) => handleFormatChange(e.target.value as GameFormat)}
                >
                  <option value="sfen">SFEN (Position Only)</option>
                  <option value="csa">CSA (Computer Shogi Association)</option>
                  <option value="kif">KIF (Japanese Format)</option>
                  <option value="json">JSON (Full Game Data)</option>
                </select>
              </div>
              <div className="export-textarea">
                <label htmlFor="export-text">Export Text:</label>
                <textarea
                  id="export-text"
                  value={exportText}
                  readOnly
                  rows={15}
                  placeholder="Game data will appear here..."
                />
              </div>
            </>
          )}
        </div>

        <div className="save-game-modal-footer">
          <button className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          {activeTab === 'save' && (
            <button className="save-button" onClick={handleSave}>
              Save
            </button>
          )}
          {activeTab === 'export' && (
            <button className="copy-button" onClick={handleCopyToClipboard}>
              Copy to Clipboard
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default SaveGameModal;