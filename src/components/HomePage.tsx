import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import SettingsPanel from './SettingsPanel';
import StartGameModal from './StartGameModal';
import { GameSettings } from '../types';
import { loadWallpaperImages, loadBoardImages, getFallbackWallpaperImages, getFallbackBoardImages } from '../utils/imageLoader';
import { setSoundsEnabled, setVolume } from '../utils/audio';
import './HomePage.css';

const HomePage: React.FC = () => {
  const navigate = useNavigate();
  const [isSettingsOpen, setIsSettingsOpen] = useState<boolean>(false);
  const [isStartGameModalOpen, setIsStartGameModalOpen] = useState<boolean>(false);
  const [pieceLabelType, setPieceLabelType] = useState<string>(localStorage.getItem('shogi-piece-label-type') || 'kanji');
  const [notation, setNotation] = useState<'western' | 'kifu' | 'usi' | 'csa'>((localStorage.getItem('shogi-notation') as any) || 'kifu');
  const [wallpaperList, setWallpaperList] = useState<string[]>([]);
  const [boardBackgroundList, setBoardBackgroundList] = useState<string[]>([]);
  const [currentWallpaper, setCurrentWallpaper] = useState<string>('');
  const [currentBoardBackground, setCurrentBoardBackground] = useState<string>('');
  const [showAttackedPieces, setShowAttackedPieces] = useState<boolean>(localStorage.getItem('shogi-show-attacked-pieces') === 'true' || true);
  const [showPieceTooltips, setShowPieceTooltips] = useState<boolean>(localStorage.getItem('shogi-show-piece-tooltips') === 'true' || false);
  const [soundsEnabled, setSoundsEnabledState] = useState<boolean>(localStorage.getItem('shogi-sounds-enabled') !== 'false');
  const [soundVolume, setSoundVolumeState] = useState<number>(() => {
    const stored = localStorage.getItem('shogi-sound-volume');
    return stored ? parseFloat(stored) : 0.9;
  });
  const [gameLayout, setGameLayoutState] = useState<'classic' | 'compact'>((localStorage.getItem('shogi-game-layout') as any) || 'compact');

  // Sync sound settings with audio manager on mount
  useEffect(() => {
    setSoundsEnabled(soundsEnabled);
    setVolume(soundVolume);
  }, [soundsEnabled, soundVolume]);

  useEffect(() => {
    const loadAssets = async () => {
      let finalWallpaperPaths: string[] = [];
      let finalBoardPaths: string[] = [];

      try {
        // Try to dynamically load images from directories
        const [wallpaperPaths, boardPaths] = await Promise.all([
          loadWallpaperImages(),
          loadBoardImages()
        ]);

        // If dynamic loading returns empty arrays, fall back to hardcoded lists
        finalWallpaperPaths = wallpaperPaths.length > 0 ? wallpaperPaths : getFallbackWallpaperImages();
        finalBoardPaths = boardPaths.length > 0 ? boardPaths : getFallbackBoardImages();

        setWallpaperList(finalWallpaperPaths);
        setBoardBackgroundList(finalBoardPaths);

        console.log('Loaded wallpapers:', finalWallpaperPaths.length, 'images');
        console.log('Loaded boards:', finalBoardPaths.length, 'images');
      } catch (error) {
        console.error('Error loading images dynamically, using fallback lists:', error);
        // Fall back to hardcoded lists if dynamic loading fails
        finalWallpaperPaths = getFallbackWallpaperImages();
        finalBoardPaths = getFallbackBoardImages();
        setWallpaperList(finalWallpaperPaths);
        setBoardBackgroundList(finalBoardPaths);
      }

      // Set current wallpaper to match the one set by App.jsx
      const currentBodyBackground = document.body.style.backgroundImage;
      if (currentBodyBackground && currentBodyBackground !== 'none') {
        // Extract the URL from the background-image style
        const urlMatch = currentBodyBackground.match(/url\(['"]?([^'"]+)['"]?\)/);
        if (urlMatch) {
          setCurrentWallpaper(urlMatch[1]);
        }
      }

      // Set random board background if not already set
      if (finalBoardPaths.length > 0 && !currentBoardBackground) {
        const initialBoardBackground = finalBoardPaths[Math.floor(Math.random() * finalBoardPaths.length)];
        setCurrentBoardBackground(initialBoardBackground);
      }
    };

    loadAssets();
  }, []);

  const handleStartGame = () => {
    setIsStartGameModalOpen(true);
  };

  const handleStartGameWithSettings = (settings: GameSettings) => {
    navigate('/game', { 
      state: { 
        aiDifficulty: 'medium', // Default difficulty, actual strength comes from engine options
        showAttackedPieces,
        showPieceTooltips,
        currentWallpaper,
        currentBoardBackground,
        player1Type: settings.player1Type,
        player2Type: settings.player2Type,
        player1EngineId: settings.player1EngineId,
        player2EngineId: settings.player2EngineId,
        player1TempOptions: settings.player1TempOptions,
        player2TempOptions: settings.player2TempOptions,
        minutesPerSide: settings.minutesPerSide,
        byoyomiInSeconds: settings.byoyomiInSeconds,
        initialSfen: settings.initialSfen
      } 
    });
    setIsStartGameModalOpen(false);
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

  const handlePieceThemeChange = (theme: string) => {
    setPieceLabelType(theme);
    localStorage.setItem('shogi-piece-label-type', theme);
    
    // Dispatch custom event for same-tab theme updates
    const event = new CustomEvent('themeChange', { detail: theme });
    window.dispatchEvent(event);
  };

  const handleNotationChange = (newNotation: 'western' | 'kifu' | 'usi' | 'csa') => {
    setNotation(newNotation);
    localStorage.setItem('shogi-notation', newNotation);
  };

  const handleShowAttackedPiecesChange = (show: boolean) => {
    setShowAttackedPieces(show);
    localStorage.setItem('shogi-show-attacked-pieces', show.toString());
  };

  const handleShowPieceTooltipsChange = (show: boolean) => {
    setShowPieceTooltips(show);
    localStorage.setItem('shogi-show-piece-tooltips', show.toString());
  };

  const handleSoundsEnabledChange = (enabled: boolean) => {
    setSoundsEnabledState(enabled);
    localStorage.setItem('shogi-sounds-enabled', enabled.toString());
    setSoundsEnabled(enabled);
  };

  const handleSoundVolumeChange = (volume: number) => {
    setSoundVolumeState(volume);
    localStorage.setItem('shogi-sound-volume', volume.toString());
    setVolume(volume);
  };

  const handleGameLayoutChange = (layout: 'classic' | 'compact') => {
    setGameLayoutState(layout);
    localStorage.setItem('shogi-game-layout', layout);
  };

  const handleBoardBackgroundChange = (background: string) => {
    setCurrentBoardBackground(background);
    localStorage.setItem('shogi-board-background', background);
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
            <span className="button-icon japanese-game">🎌</span>
            <span className="button-text">Start New Game</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/help')}
          >
            <span className="button-icon japanese-help">📜</span>
            <span className="button-text">Help</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/practice')}
          >
            <span className="button-icon japanese-practice">🏯</span>
            <span className="button-text">Practice</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={handleOpenSettings}
          >
            <span className="button-icon japanese-settings">⚙️</span>
            <span className="button-text">Settings</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/engines')}
          >
            <span className="button-icon japanese-engines">🤖</span>
            <span className="button-text">Engines</span>
          </button>
          
          <button 
            className="nav-button"
            onClick={() => navigate('/about')}
          >
            <span className="button-icon japanese-about">🎋</span>
            <span className="button-text">About</span>
          </button>
          

        </div>
        
        <div className="home-footer">
          <p>Master the art of Shogi through practice and play</p>
        </div>
      </div>

      {isSettingsOpen && (
        <SettingsPanel
          pieceThemeType={pieceLabelType}
          onPieceThemeTypeChange={handlePieceThemeChange}
          notation={notation}
          onNotationChange={handleNotationChange}
          wallpaperList={wallpaperList}
          onSelectWallpaper={handleSelectWallpaper}
          boardBackgroundList={boardBackgroundList}
          onSelectBoardBackground={handleBoardBackgroundChange}
          onClose={handleCloseSettings}
          currentWallpaper={currentWallpaper}
          currentBoardBackground={currentBoardBackground}
          showAttackedPieces={showAttackedPieces}
          onShowAttackedPiecesChange={handleShowAttackedPiecesChange}
          showPieceTooltips={showPieceTooltips}
          onShowPieceTooltipsChange={handleShowPieceTooltipsChange}
          gameLayout={gameLayout}
          onGameLayoutChange={handleGameLayoutChange}
          soundsEnabled={soundsEnabled}
          onSoundsEnabledChange={handleSoundsEnabledChange}
          soundVolume={soundVolume}
          onSoundVolumeChange={handleSoundVolumeChange}
        />
      )}
      
      <StartGameModal 
        isOpen={isStartGameModalOpen} 
        onClose={() => setIsStartGameModalOpen(false)} 
        onStartGame={handleStartGameWithSettings} 
      />
    </div>
  );
};

export default HomePage;
