import React, { useState } from 'react';
import './LoadGameModal.css';
import { GameFormat, parseGame, detectFormat, GameData, testKifParsing } from '../utils/gameFormats';

interface LoadGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onLoad: (name: string, format: GameFormat) => void;
  onLoadFromText: (gameData: GameData) => void;
  onDelete: (name: string) => void;
  savedGames: { [key: string]: string };
}

const LoadGameModal: React.FC<LoadGameModalProps> = ({ 
  isOpen, 
  onClose, 
  onLoad, 
  onLoadFromText, 
  onDelete, 
  savedGames 
}) => {
  const [activeTab, setActiveTab] = useState<'storage' | 'import'>('storage');
  const [importText, setImportText] = useState('');
  const [importFormat, setImportFormat] = useState<GameFormat>('sfen');
  const [autoDetect, setAutoDetect] = useState(true);
  const [parseError, setParseError] = useState<string | null>(null);

  const handleImport = () => {
    if (!importText.trim()) {
      setParseError('Please enter game data');
      return;
    }

    try {
      const detectedFormat = autoDetect ? detectFormat(importText) : importFormat;
      if (!detectedFormat) {
        setParseError('Could not detect format. Please select format manually.');
        return;
      }

      const result = parseGame(importText, detectedFormat);
      console.log('Parse result:', result);
      if (result.success && result.data) {
        console.log('Game data:', result.data);
        onLoadFromText(result.data);
        setImportText('');
        setParseError(null);
        onClose();
      } else {
        setParseError(result.error || 'Failed to parse game data');
      }
    } catch (error) {
      setParseError(`Error parsing game: ${error}`);
    }
  };

  const handleAutoDetectToggle = () => {
    setAutoDetect(!autoDetect);
    if (importText.trim()) {
      const detected = detectFormat(importText);
      if (detected) {
        setImportFormat(detected);
      }
    }
  };

  const handleTextChange = (text: string) => {
    setImportText(text);
    setParseError(null);
    
    if (autoDetect && text.trim()) {
      const detected = detectFormat(text);
      if (detected) {
        setImportFormat(detected);
      }
    }
  };

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
        
        <div className="load-game-modal-tabs">
          <button 
            className={`tab-button ${activeTab === 'storage' ? 'active' : ''}`}
            onClick={() => setActiveTab('storage')}
          >
            From Storage
          </button>
          <button 
            className={`tab-button ${activeTab === 'import' ? 'active' : ''}`}
            onClick={() => setActiveTab('import')}
          >
            Import/Paste
          </button>
        </div>

        <div className="load-game-modal-content">
          {activeTab === 'storage' && (
            <>
              {Object.keys(savedGames).length === 0 ? (
                <p>No saved games found.</p>
              ) : (
                <ul className="saved-games-list">
                  {Object.keys(savedGames).map((name) => (
                    <li key={name} className="saved-game-item">
                      <span>{name}</span>
                      <div className="buttons">
                        <button onClick={() => onLoad(name, 'sfen')}>Load</button>
                        <button onClick={() => onDelete(name)}>Delete</button>
                      </div>
                    </li>
                  ))}
                </ul>
              )}
            </>
          )}

          {activeTab === 'import' && (
            <>
              <div className="import-controls">
                <div className="auto-detect-toggle">
                  <label>
                    <input
                      type="checkbox"
                      checked={autoDetect}
                      onChange={handleAutoDetectToggle}
                    />
                    Auto-detect format
                  </label>
                </div>
                
                {!autoDetect && (
                  <div className="format-selector">
                    <label htmlFor="import-format-select">Format:</label>
                    <select
                      id="import-format-select"
                      value={importFormat}
                      onChange={(e) => setImportFormat(e.target.value as GameFormat)}
                    >
                      <option value="sfen">SFEN (Position Only)</option>
                      <option value="csa">CSA (Computer Shogi Association)</option>
                      <option value="kif">KIF (Japanese Format)</option>
                      <option value="json">JSON (Full Game Data)</option>
                    </select>
                  </div>
                )}
                
                {autoDetect && importFormat && (
                  <div className="detected-format">
                    <span>Detected format: <strong>{importFormat.toUpperCase()}</strong></span>
                  </div>
                )}
                
                <div className="debug-controls">
                  <button 
                    type="button"
                    onClick={() => {
                      const result = testKifParsing();
                      console.log('Test result:', result);
                    }}
                    style={{ fontSize: '12px', padding: '4px 8px' }}
                  >
                    Test KIF Parser
                  </button>
                </div>
              </div>

              <div className="import-textarea">
                <label htmlFor="import-text">Game Data:</label>
                <textarea
                  id="import-text"
                  value={importText}
                  onChange={(e) => handleTextChange(e.target.value)}
                  placeholder="Paste game data here (SFEN, CSA, KIF, or JSON format)..."
                  rows={15}
                />
              </div>

              {parseError && (
                <div className="parse-error">
                  <strong>Error:</strong> {parseError}
                </div>
              )}
            </>
          )}
        </div>

        <div className="load-game-modal-footer">
          <button className="cancel-button" onClick={onClose}>
            Cancel
          </button>
          {activeTab === 'import' && (
            <button className="import-button" onClick={handleImport}>
              Import Game
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

export default LoadGameModal;