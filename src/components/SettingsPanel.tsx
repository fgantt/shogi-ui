import React, { useState } from 'react';
import { Link } from 'react-router-dom';
import PiecePreview from './PiecePreview';

type Difficulty = 'easy' | 'medium' | 'hard';
type PieceThemeType = 'kanji' | 'english' | '13xforever-1-kanji' | '13xforever-2-kanji' | '2-kanji_red_wood' | 'doubutsu' | 'Hari-Seldon-1-kanji' | 'Hari-Seldon-2-kanji' | 'international' | 'kanji_light_3D_OTB' | 'kanji_red_wood' | 'Kinki-1-kanji' | 'Kinki-2-kanji' | 'Minase-1-kanji' | 'Minase-2-kanji' | 'Ryoko-1-kanji' | 'Ryoko-2-kanji';
type Notation = 'western' | 'kifu' | 'usi' | 'csa';

interface SettingsPanelProps {
  aiDifficulty: Difficulty;
  onDifficultyChange: (difficulty: Difficulty) => void;
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
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  aiDifficulty,
  onDifficultyChange,
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
          <h3>Piece Themes</h3>
          <div className="setting-group">
            <label>
              <input
                type="radio"
                value="kanji"
                checked={pieceThemeType === 'kanji'}
                onChange={() => onPieceThemeTypeChange('kanji')}
              />
              Kanji
            </label>
            <label>
              <input
                type="radio"
                value="english"
                checked={pieceThemeType === 'english'}
                onChange={() => onPieceThemeTypeChange('english')}
              />
              English
            </label>
            <label>
              <input
                type="radio"
                value="13xforever-1-kanji"
                checked={pieceThemeType === '13xforever-1-kanji'}
                onChange={() => onPieceThemeTypeChange('13xforever-1-kanji')}
              />
              13xforever 1
            </label>
            <label>
              <input
                type="radio"
                value="13xforever-2-kanji"
                checked={pieceThemeType === '13xforever-2-kanji'}
                onChange={() => onPieceThemeTypeChange('13xforever-2-kanji')}
              />
              13xforever 2
            </label>
            <label>
              <input
                type="radio"
                value="2-kanji_red_wood"
                checked={pieceThemeType === '2-kanji_red_wood'}
                onChange={() => onPieceThemeTypeChange('2-kanji_red_wood')}
              />
              2 Kanji Red Wood
            </label>
            <label>
              <input
                type="radio"
                value="doubutsu"
                checked={pieceThemeType === 'doubutsu'}
                onChange={() => onPieceThemeTypeChange('doubutsu')}
              />
              Doubutsu
            </label>
            <label>
              <input
                type="radio"
                value="Hari-Seldon-1-kanji"
                checked={pieceThemeType === 'Hari-Seldon-1-kanji'}
                onChange={() => onPieceThemeTypeChange('Hari-Seldon-1-kanji')}
              />
              Hari-Seldon 1
            </label>
            <label>
              <input
                type="radio"
                value="Hari-Seldon-2-kanji"
                checked={pieceThemeType === 'Hari-Seldon-2-kanji'}
                onChange={() => onPieceThemeTypeChange('Hari-Seldon-2-kanji')}
              />
              Hari-Seldon 2
            </label>
            <label>
              <input
                type="radio"
                value="international"
                checked={pieceThemeType === 'international'}
                onChange={() => onPieceThemeTypeChange('international')}
              />
              International
            </label>
            <label>
              <input
                type="radio"
                value="kanji_light_3D_OTB"
                checked={pieceThemeType === 'kanji_light_3D_OTB'}
                onChange={() => onPieceThemeTypeChange('kanji_light_3D_OTB')}
              />
              Kanji Light 3D OTB
            </label>
            <label>
              <input
                type="radio"
                value="kanji_red_wood"
                checked={pieceThemeType === 'kanji_red_wood'}
                onChange={() => onPieceThemeTypeChange('kanji_red_wood')}
              />
              Kanji Red Wood
            </label>
            <label>
              <input
                type="radio"
                value="Kinki-1-kanji"
                checked={pieceThemeType === 'Kinki-1-kanji'}
                onChange={() => onPieceThemeTypeChange('Kinki-1-kanji')}
              />
              Kinki 1
            </label>
            <label>
              <input
                type="radio"
                value="Kinki-2-kanji"
                checked={pieceThemeType === 'Kinki-2-kanji'}
                onChange={() => onPieceThemeTypeChange('Kinki-2-kanji')}
              />
              Kinki 2
            </label>
            <label>
              <input
                type="radio"
                value="Minase-1-kanji"
                checked={pieceThemeType === 'Minase-1-kanji'}
                onChange={() => onPieceThemeTypeChange('Minase-1-kanji')}
              />
              Minase 1
            </label>
            <label>
              <input
                type="radio"
                value="Minase-2-kanji"
                checked={pieceThemeType === 'Minase-2-kanji'}
                onChange={() => onPieceThemeTypeChange('Minase-2-kanji')}
              />
              Minase 2
            </label>
            <label>
              <input
                type="radio"
                value="Ryoko-1-kanji"
                checked={pieceThemeType === 'Ryoko-1-kanji'}
                onChange={() => onPieceThemeTypeChange('Ryoko-1-kanji')}
              />
              Ryoko 1
            </label>
            <label>
              <input
                type="radio"
                value="Ryoko-2-kanji"
                checked={pieceThemeType === 'Ryoko-2-kanji'}
                onChange={() => onPieceThemeTypeChange('Ryoko-2-kanji')}
              />
              Ryoko 2
            </label>
          </div>
          <PiecePreview theme={pieceThemeType} />
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
              English
            </label>
            <label>
              <input
                type="radio"
                value="kifu"
                checked={notation === 'kifu'}
                onChange={() => onNotationChange('kifu')}
              />
              KIF
            </label>
            <label>
              <input
                type="radio"
                value="usi"
                checked={notation === 'usi'}
                onChange={() => onNotationChange('usi')}
              />
              USI
            </label>
            <label>
              <input
                type="radio"
                value="csa"
                checked={notation === 'csa'}
                onChange={() => onNotationChange('csa')}
              />
              CSA
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