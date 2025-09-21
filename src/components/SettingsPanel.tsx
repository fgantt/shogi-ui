import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import PiecePreview from './PiecePreview';
import ThemeSelector from './ThemeSelector';
import '../styles/settings.css';

type Notation = 'western' | 'kifu' | 'usi' | 'csa';

interface SettingsPanelProps {
  pieceThemeType: string;
  onPieceThemeTypeChange: (type: string) => void;
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
  gameLayout: 'classic' | 'compact';
  onGameLayoutChange: (layout: 'classic' | 'compact') => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  pieceThemeType,
  onPieceThemeTypeChange,
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
  gameLayout,
  onGameLayoutChange,
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
          <h3>Piece Themes</h3>
          <ThemeSelector 
            selectedTheme={pieceThemeType} 
            onThemeChange={onPieceThemeTypeChange} 
          />
          <PiecePreview theme={pieceThemeType} />
        </section>

        <section>
          <h3>Move Log Notation</h3>
          <div className="setting-group">
            <label className="notation-option">
              <input
                type="radio"
                value="western"
                checked={notation === 'western'}
                onChange={() => onNotationChange('western')}
              />
              <div className="notation-label">
                <span className="notation-name">English</span>
                <span className="notation-example">P-7f, Rx2d</span>
              </div>
            </label>
            <label className="notation-option">
              <input
                type="radio"
                value="kifu"
                checked={notation === 'kifu'}
                onChange={() => onNotationChange('kifu')}
              />
              <div className="notation-label">
                <span className="notation-name">KIF</span>
                <span className="notation-example">７六歩, ２四飛</span>
              </div>
            </label>
            <label className="notation-option">
              <input
                type="radio"
                value="usi"
                checked={notation === 'usi'}
                onChange={() => onNotationChange('usi')}
              />
              <div className="notation-label">
                <span className="notation-name">USI</span>
                <span className="notation-example">7g7f, 2d2b</span>
              </div>
            </label>
            <label className="notation-option">
              <input
                type="radio"
                value="csa"
                checked={notation === 'csa'}
                onChange={() => onNotationChange('csa')}
              />
              <div className="notation-label">
                <span className="notation-name">CSA</span>
                <span className="notation-example">+7776FU, -2424HI</span>
              </div>
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

        <section>
          <h3>Game Layout</h3>
          <div className="setting-group">
            <label>
              <input
                type="radio"
                value="classic"
                checked={gameLayout === 'classic'}
                onChange={() => onGameLayoutChange('classic')}
              />
              Slim Shogi
            </label>
            <label>
              <input
                type="radio"
                value="compact"
                checked={gameLayout === 'compact'}
                onChange={() => onGameLayoutChange('compact')}
              />
              Classic Shogi
            </label>
          </div>
        </section>

        <button onClick={onClose}>Close</button>
      </div>
    </div>
  );
};

export default SettingsPanel;