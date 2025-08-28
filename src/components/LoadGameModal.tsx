import React, { useState, useRef } from 'react';
import { parseShogiFile, detectFormatFromExtension, SUPPORTED_FORMATS, SupportedFormat } from '../game/shogi';
import { GameState } from '../types';
import './LoadGameModal.css';

interface LoadGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onGameLoaded: (gameState: GameState) => void;
}

const LoadGameModal: React.FC<LoadGameModalProps> = ({ isOpen, onClose, onGameLoaded }) => {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [detectedFormat, setDetectedFormat] = useState<SupportedFormat | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  if (!isOpen) return null;

  const handleFileSelect = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      setSelectedFile(file);
      setError(null);
      
      // Try to detect format from extension
      const format = detectFormatFromExtension(file.name);
      setDetectedFormat(format);
      
      if (!format) {
        setError('Could not determine file format from extension. Please ensure the file has a valid extension (.kif, .csa, .jkf, .sfen, etc.)');
      }
    }
  };

  const handleLoad = async () => {
    if (!selectedFile) return;

    setIsLoading(true);
    setError(null);

    try {
      const fileContent = await readFileAsText(selectedFile);
      const format = detectedFormat || 'kif'; // Default to KIF if we can't detect
      
      const gameState = parseShogiFile(fileContent, format);
      onGameLoaded(gameState);
      onClose();
      
      // Reset state
      setSelectedFile(null);
      setDetectedFormat(null);
      if (fileInputRef.current) {
        fileInputRef.current.value = '';
      }
    } catch (error) {
      console.error('Error loading game:', error);
      setError(`Failed to load game: ${error instanceof Error ? error.message : 'Unknown error'}`);
    } finally {
      setIsLoading(false);
    }
  };

  const handleClose = () => {
    setSelectedFile(null);
    setDetectedFormat(null);
    setError(null);
    if (fileInputRef.current) {
      fileInputRef.current.value = '';
    }
    onClose();
  };

  const handleDragOver = (event: React.DragEvent) => {
    event.preventDefault();
  };

  const handleDrop = (event: React.DragEvent) => {
    event.preventDefault();
    const files = event.dataTransfer.files;
    if (files.length > 0) {
      const file = files[0];
      setSelectedFile(file);
      setError(null);
      
      const format = detectFormatFromExtension(file.name);
      setDetectedFormat(format);
      
      if (!format) {
        setError('Could not determine file format from extension. Please ensure the file has a valid extension (.kif, .csa, .jkf, .sfen, etc.)');
      }
    }
  };

  const getSupportedExtensions = (): string => {
    return Object.values(SUPPORTED_FORMATS)
      .map(format => format.extension)
      .join(', ');
  };

  return (
    <div className="load-game-modal-overlay">
      <div className="load-game-modal">
        <div className="load-game-modal-header">
          <h2>Load Game</h2>
          <button className="close-button" onClick={handleClose}>
            ×
          </button>
        </div>
        
        <div className="load-game-modal-content">
          <div className="file-drop-zone"
               onDragOver={handleDragOver}
               onDrop={handleDrop}>
            <div className="file-drop-content">
              <div className="file-icon">📁</div>
              <p>Drag and drop a Shogi file here, or click to browse</p>
              <button 
                className="browse-button"
                onClick={() => fileInputRef.current?.click()}
                disabled={isLoading}
              >
                Browse Files
              </button>
              <input
                ref={fileInputRef}
                type="file"
                accept={getSupportedExtensions()}
                onChange={handleFileSelect}
                style={{ display: 'none' }}
              />
            </div>
          </div>
          
          {selectedFile && (
            <div className="file-info">
              <h3>Selected File:</h3>
              <p><strong>Name:</strong> {selectedFile.name}</p>
              <p><strong>Size:</strong> {(selectedFile.size / 1024).toFixed(1)} KB</p>
              <p><strong>Type:</strong> {selectedFile.type || 'Unknown'}</p>
              {detectedFormat && (
                <p><strong>Detected Format:</strong> {SUPPORTED_FORMATS[detectedFormat].name}</p>
              )}
            </div>
          )}
          
          {error && (
            <div className="error-message">
              <p>⚠️ {error}</p>
            </div>
          )}
          
          <div className="supported-formats">
            <h3>Supported Formats:</h3>
            <ul>
              {Object.entries(SUPPORTED_FORMATS).map(([key, format]) => (
                <li key={key}>
                  <strong>{format.name}</strong> ({format.extension}) - {getFormatDescription(key as SupportedFormat)}
                </li>
              ))}
            </ul>
          </div>
        </div>
        
        <div className="load-game-modal-footer">
          <button className="cancel-button" onClick={handleClose}>
            Cancel
          </button>
          <button 
            className="load-button" 
            onClick={handleLoad}
            disabled={!selectedFile || isLoading}
          >
            {isLoading ? 'Loading...' : 'Load Game'}
          </button>
        </div>
      </div>
    </div>
  );
};

function getFormatDescription(format: SupportedFormat): string {
  switch (format) {
    case 'kif':
      return 'Traditional Japanese KIF format';
    case 'ki2':
      return 'Compact Japanese KI2 format';
    case 'csa':
      return 'CSA format used in Japanese servers';
    case 'jkf':
      return 'JSON-based format';
    case 'sfen':
      return 'SFEN notation';
    default:
      return 'Unknown format';
  }
}

function readFileAsText(file: File): Promise<string> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = (event) => {
      const result = event.target?.result;
      if (typeof result === 'string') {
        resolve(result);
      } else {
        reject(new Error('Failed to read file as text'));
      }
    };
    reader.onerror = () => reject(new Error('Failed to read file'));
    reader.readAsText(file);
  });
}

export default LoadGameModal;
