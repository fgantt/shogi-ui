import React from 'react';

const SettingsPanel = ({
  aiDifficulty,
  onDifficultyChange,
  pieceLabelType,
  onPieceLabelTypeChange,
  wallpaperList,
  onSelectWallpaper,
  boardBackgroundList,
  onSelectBoardBackground,
  onClose,
  currentWallpaper,
  currentBoardBackground,
  showAttackedPieces,
  onShowAttackedPiecesChange,
}) => {
  const getFileName = (path) => {
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
          <h3>Board Background</h3>
          <div className="setting-group setting-thumbnails">
            {boardBackgroundList.map((bg, index) => {
              console.log(`Board Thumbnail: ${bg}, Current Board: ${currentBoardBackground}, Match: ${bg === currentBoardBackground}`);
              return (
              <img
                key={index}
                src={bg}
                alt={`Board Background ${index + 1}`}
                className={`thumbnail ${bg === currentBoardBackground ? 'selected-thumbnail' : ''}`}
                onClick={() => onSelectBoardBackground(bg)}
                title={getFileName(bg)}
              />
            )})}
          </div>
        </section>

        <section>
          <h3>Wallpaper</h3>
          <div className="setting-group setting-thumbnails">
            {wallpaperList.map((wp, index) => {
              console.log(`Wallpaper Thumbnail: ${wp}, Current Wallpaper: ${currentWallpaper}, Match: ${wp === currentWallpaper}`);
              return (
              <img
                key={index}
                src={wp}
                alt={`Wallpaper ${index + 1}`}
                className={`thumbnail ${wp === currentWallpaper ? 'selected-thumbnail' : ''}`}
                onClick={() => onSelectWallpaper(wp)}
                title={getFileName(wp)}
              />
            )})}
          </div>
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

        <button onClick={onClose}>Close</button>
      </div>
    </div>
  );
};

export default SettingsPanel;
