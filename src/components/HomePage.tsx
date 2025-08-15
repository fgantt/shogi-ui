import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import SettingsPanel from './SettingsPanel';
import './HomePage.css';

const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [aiDifficulty, setAiDifficulty] = useState<'easy' | 'medium' | 'hard'>('medium');
  const [pieceLabelType, setPieceLabelType] = useState<'kanji' | 'english'>('kanji');
  const [wallpaperList, setWallpaperList] = useState<string[]>([]);
  const [boardBackgroundList, setBoardBackgroundList] = useState<string[]>([]);
  const [currentWallpaper, setCurrentWallpaper] = useState<string>('');
  const [currentBoardBackground, setCurrentBoardBackground] = useState<string>('');
  const [showAttackedPieces, setShowAttackedPieces] = useState<boolean>(true);
  const [showPieceTooltips, setShowPieceTooltips] = useState<boolean>(false);

  useEffect(() => {
    const importWallpapers = async () => {
      const modules = import.meta.glob('/public/wallpapers/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      setWallpaperList(paths);
      // Set current wallpaper to match the one set by App.jsx
      const currentBodyBackground = document.body.style.backgroundImage;
      if (currentBodyBackground && currentBodyBackground !== 'none') {
        // Extract the URL from the background-image style
        const urlMatch = currentBodyBackground.match(/url\(['"]?([^'"]+)['"]?\)/);
        if (urlMatch) {
          setCurrentWallpaper(urlMatch[1]);
        }
      }
    };
    const importBoardBackgrounds = async () => {
      const modules = import.meta.glob('/public/boards/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      setBoardBackgroundList(paths);
      if (paths.length > 0) {
        const initialBoardBackground = paths[Math.floor(Math.random() * paths.length)];
        setCurrentBoardBackground(initialBoardBackground);
      }
    };
    importWallpapers();
    importBoardBackgrounds();
  }, []);

  const handleStartGame = () => {
    navigate('/game', { 
      state: { 
        aiDifficulty,
        pieceLabelType,
        showAttackedPieces,
        showPieceTooltips,
        currentWallpaper,
        currentBoardBackground
      } 
    });
  };

  const handleOpenSettings = () => {
    setIsSettingsOpen(true);
  };

  const handleCloseSettings = () => {
    setIsSettingsOpen(false);
  };

  const handleSelectWallpaper = (wallpaper) => {
    setCurrentWallpaper(wallpaper);
    document.body.style.backgroundImage = `url('${wallpaper}')`;
    document.body.style.backgroundSize = 'cover';
    document.body.style.backgroundRepeat = 'no-repeat';
    document.body.style.backgroundPosition = 'center center';
    document.body.style.backgroundAttachment = 'fixed';
  };

  const handleSelectBoardBackground = (boardBackground) => {
    setCurrentBoardBackground(boardBackground);
  };

  return (
    <div className="home-page">
      <div className="home-content">
        <h1 className="home-title">Shogi Vibe</h1>
        <p className="home-subtitle">Experience the ancient art of Japanese chess</p>
        
        <div className="navigation-grid">
          <button 
            className="nav-button primary"
            onClick={handleStartGame}
          >
            <span className="button-icon">♔</span>
            <span className="button-text">Start New Game</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/help')}
          >
            <span className="button-icon">📖</span>
            <span className="button-text">Help</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/practice')}
          >
            <span className="button-icon">🎯</span>
            <span className="button-text">Practice</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={handleOpenSettings}
          >
            <span className="button-icon">⚙️</span>
            <span className="button-text">Settings</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/about')}
          >
            <span className="button-icon">ℹ️</span>
            <span className="button-text">About</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/wasm-demo')}
          >
            <span className="button-icon">🚀</span>
            <span className="button-text">WebAssembly Demo</span>
          </button>
        </div>
        
        <div className="home-footer">
          <p>Master the art of Shogi through practice and play</p>
        </div>
      </div>

      {isSettingsOpen && (
        <SettingsPanel
          aiDifficulty={aiDifficulty}
          onDifficultyChange={setAiDifficulty}
          pieceLabelType={pieceLabelType}
          onPieceLabelTypeChange={setPieceLabelType}
          wallpaperList={wallpaperList}
          onSelectWallpaper={handleSelectWallpaper}
          boardBackgroundList={boardBackgroundList}
          onSelectBoardBackground={handleSelectBoardBackground}
          onClose={handleCloseSettings}
          currentWallpaper={currentWallpaper}
          currentBoardBackground={currentBoardBackground}
          showAttackedPieces={showAttackedPieces}
          onShowAttackedPiecesChange={setShowAttackedPieces}
          showPieceTooltips={showPieceTooltips}
          onShowPieceTooltipsChange={setShowPieceTooltips}
        />
      )}
    </div>
  );
};

export default HomePage;
