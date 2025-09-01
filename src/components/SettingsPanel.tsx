import React, { useState } from 'react';
import { Link } from 'react-router-dom';

type Difficulty = 'easy' | 'medium' | 'hard';
type PieceLabelType = 'kanji' | 'english';
type Notation = 'western' | 'kifu';

interface SettingsPanelProps {
  aiDifficulty: Difficulty;
  onDifficultyChange: (difficulty: Difficulty) => void;
  pieceLabelType: PieceLabelType;
  onPieceLabelTypeChange: (type: PieceLabelType) => void;
  notation: Notation;
  onNotationChange: (notation: Notation) => void;
  wallpaperList: string[];
  onSelectWallpaper: (wallpaper: string) => void;
  boardBackgroundList: string[];
  onSelectBoardBackground: (background: string) => void;
  onClose: () => void;
  currentWallpaper: string;
  currentBoardBackground: string;
  showAttackedPieces: boolean;
  onShowAttackedPiecesChange: (show: boolean) => void;
  showPieceTooltips: boolean;
  onShowPieceTooltipsChange: (show: boolean) => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  aiDifficulty,
  onDifficultyChange,
  pieceLabelType,
  onPieceLabelTypeChange,
  notation,
  onNotationChange,
  wallpaperList,
  onSelectWallpaper,
  boardBackgroundList,
  onSelectBoardBackground,
  onClose,
  currentWallpaper,
  currentBoardBackground,
  showAttackedPieces,
  onShowAttackedPiecesChange,
  showPieceTooltips,
  onShowPieceTooltipsChange,
}) => {
  const [isBoardBackgroundCollapsed, setIsBoardBackgroundCollapsed] = useState(true);
  const [isWallpaperCollapsed, setIsWallpaperCollapsed] = useState(true);

  const toggleBoardBackgroundCollapse = () => {
    setIsBoardBackgroundCollapsed(!isBoardBackgroundCollapsed);
  };

  const toggleWallpaperCollapse = () => {
    setIsWallpaperCollapsed(!isWallpaperCollapsed);
  };
  const getFileName = (path: string): string => {
    const parts = path.split('/');
    const fileNameWithExtension = parts[parts.length - 1];
    const fileName = fileNameWithExtension.split('.')[0];
    return fileName;
  };

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>Settings</h2>

        <section>
          <h3>AI Difficulty</h3>
          <div className="setting-group">
            <label>
              <input
                type="radio"
                value="easy"
                checked={aiDifficulty === 'easy'}
                onChange={() => onDifficultyChange('easy')}
              />
              Easy
            </label>
            <label>
              <input
                type="radio"
                value="medium"
                checked={aiDifficulty === 'medium'}
                onChange={() => onDifficultyChange('medium')}
              />
              Medium
            </label>
            <label>
              <input
                type="radio"
                value="hard"
                checked={aiDifficulty === 'hard'}
                onChange={() => onDifficultyChange('hard')}
              />
              Hard
            </label>
          </div>
        </section>

        <section>
          <h3>Engine</h3>
          <div className="setting-group">
            <Link to="/settings/engine" className="button-link">
              Manage Engines
            </Link>
          </div>
        </section>

        <section>
          <h3>Piece Labels</h3>
          <div className="setting-group">
            <label>
              <input
                type="radio"
                value="kanji"
                checked={pieceLabelType === 'kanji'}
                onChange={() => onPieceLabelTypeChange('kanji')}
              />
              Kanji
            </label>
            <label>
              <input
                type="radio"
                value="english"
                checked={pieceLabelType === 'english'}
                onChange={() => onPieceLabelTypeChange('english')}
              />
              English
            </label>
          </div>
        </section>

        <section>
          <h3>Move Log Notation</h3>
          <div className="setting-group">
            <label>
              <input
                type="radio"
                value="western"
                checked={notation === 'western'}
                onChange={() => onNotationChange('western')}
              />
              Western
            </label>
            <label>
              <input
                type="radio"
                value="kifu"
                checked={notation === 'kifu'}
                onChange={() => onNotationChange('kifu')}
              />
              Kifu
            </label>
          </div>
        </section>

        <section>
          <h3 onClick={toggleBoardBackgroundCollapse} style={{ cursor: 'pointer' }}>
            Board Background
            <span className={`collapse-arrow ${isBoardBackgroundCollapsed ? 'collapsed' : ''}`}>&#9660;</span>
          </h3>
          {!isBoardBackgroundCollapsed && (
            <div className="setting-group setting-thumbnails">
              {boardBackgroundList.map((bg, index) => (
                <img
                  key={index}
                  src={bg}
                  alt={`Board Background ${index + 1}`}
                  className={`thumbnail ${bg === currentBoardBackground ? 'selected-thumbnail' : ''}`}
                  onClick={() => onSelectBoardBackground(bg)}
                  title={getFileName(bg)}
                />
              ))}
            </div>
          )}
        </section>

        <section>
          <h3 onClick={toggleWallpaperCollapse} style={{ cursor: 'pointer' }}>
            Wallpaper
            <span className={`collapse-arrow ${isWallpaperCollapsed ? 'collapsed' : ''}`}>&#9660;</span>
          </h3>
          {!isWallpaperCollapsed && (
            <div className="setting-group setting-thumbnails">
              {wallpaperList.map((wp, index) => (
                <img
                  key={index}
                  src={wp}
                  alt={`Wallpaper ${index + 1}`}
                  className={`thumbnail ${wp === currentWallpaper ? 'selected-thumbnail' : ''}`}
                  onClick={() => onSelectWallpaper(wp)}
                  title={getFileName(wp)}
                />
              ))}
            </div>
          )}
        </section>

        <section>
          <h3>Show Attacked Pieces</h3>
          <div className="setting-group">
            <label className="switch">
              <input
                type="checkbox"
                checked={showAttackedPieces}
                onChange={(e) => onShowAttackedPiecesChange(e.target.checked)}
              />
              <span className="slider round"></span>
            </label>
          </div>
        </section>

        <section>
          <h3>Show Piece Tooltips</h3>
          <div className="setting-group">
            <label className="switch">
              <input
                type="checkbox"
                checked={showPieceTooltips}
                onChange={(e) => onShowPieceTooltipsChange(e.target.checked)}
              />
              <span className="slider round"></span>
            </label>
          </div>
        </section>

        <button onClick={onClose}>Close</button>
      </div>
    </div>
  );
};

export default SettingsPanel;