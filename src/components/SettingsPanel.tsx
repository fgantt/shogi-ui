import React, { useState, useRef, useEffect } from 'react';
import PiecePreview from './PiecePreview';
import ThemeSelector from './ThemeSelector';
import { useTheme, getThemeDisplayName, getThemeDescription, type Theme } from '../hooks/useTheme';
import { playPreviewSound } from '../utils/audio';
import { EngineSelector } from './EngineSelector';
import { EngineOptionsModal } from './EngineOptionsModal';
import type { EngineConfig } from '../types/engine';
import { invoke } from '@tauri-apps/api/core';
import '../styles/settings.css';

type Notation = 'western' | 'kifu' | 'usi' | 'csa';
type SettingsTab = 'appearance' | 'display' | 'notation' | 'backgrounds' | 'engines';

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
  showEngineThinking: boolean;
  onShowEngineThinkingChange: (show: boolean) => void;
  gameLayout: 'classic' | 'compact';
  onGameLayoutChange: (layout: 'classic' | 'compact') => void;
  soundsEnabled: boolean;
  onSoundsEnabledChange: (enabled: boolean) => void;
  soundVolume: number;
  onSoundVolumeChange: (volume: number) => void;
  // Recommendation engine settings
  recommendationEngineId: string | null;
  onRecommendationEngineChange: (engineId: string | null) => void;
  recommendationEngineOptions: {[key: string]: string} | null;
  onRecommendationEngineOptionsChange: (options: {[key: string]: string} | null) => void;
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
  showEngineThinking,
  onShowEngineThinkingChange,
  gameLayout,
  onGameLayoutChange,
  soundsEnabled,
  onSoundsEnabledChange,
  soundVolume,
  onSoundVolumeChange,
  // Recommendation engine settings
  recommendationEngineId,
  onRecommendationEngineChange,
  recommendationEngineOptions,
  onRecommendationEngineOptionsChange,
}) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>('appearance');
  const [isBoardBackgroundCollapsed, setIsBoardBackgroundCollapsed] = useState(false);
  const [isWallpaperCollapsed, setIsWallpaperCollapsed] = useState(false);
  const [isColorThemeCollapsed, setIsColorThemeCollapsed] = useState(false);
  const [isPieceThemesCollapsed, setIsPieceThemesCollapsed] = useState(false);
  const { theme, setTheme } = useTheme();
  const volumePreviewTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  
  // Recommendation engine options modal state
  const [optionsModalOpen, setOptionsModalOpen] = useState(false);
  const [selectedEngine, setSelectedEngine] = useState<EngineConfig | null>(null);
  const [engines, setEngines] = useState<EngineConfig[]>([]);

  // Load engines when component mounts
  useEffect(() => {
    loadEngines();
  }, []);

  // Cleanup timeout on unmount
  useEffect(() => {
    return () => {
      if (volumePreviewTimeoutRef.current) {
        clearTimeout(volumePreviewTimeoutRef.current);
      }
    };
  }, []);

  const loadEngines = async () => {
    try {
      const response = await invoke<any>('get_engines');
      if (response.success && response.data) {
        setEngines(response.data);
        
        // Auto-select favorite engine if no recommendation engine is selected
        if (!recommendationEngineId && response.data.length > 0) {
          const favoriteEngine = response.data.find(e => e.is_favorite);
          const defaultEngine = favoriteEngine || response.data.find(e => e.is_builtin) || response.data[0];
          if (defaultEngine) {
            onRecommendationEngineChange(defaultEngine.id);
          }
        }
      }
    } catch (error) {
      console.error('Error loading engines:', error);
    }
  };

  const handleOpenRecommendationEngineOptions = () => {
    if (!recommendationEngineId) return;
    
    const engine = engines.find(e => e.id === recommendationEngineId);
    if (engine) {
      setSelectedEngine(engine);
      setOptionsModalOpen(true);
    }
  };

  const handleCloseRecommendationEngineOptions = () => {
    setOptionsModalOpen(false);
    setSelectedEngine(null);
  };

  const handleSaveRecommendationEngineOptions = (options: {[key: string]: string}) => {
    onRecommendationEngineOptionsChange(options);
    setOptionsModalOpen(false);
    setSelectedEngine(null);
  };

  const toggleBoardBackgroundCollapse = () => {
    setIsBoardBackgroundCollapsed(!isBoardBackgroundCollapsed);
  };

  const toggleWallpaperCollapse = () => {
    setIsWallpaperCollapsed(!isWallpaperCollapsed);
  };

  const toggleColorThemeCollapse = () => {
    setIsColorThemeCollapsed(!isColorThemeCollapsed);
  };

  const togglePieceThemesCollapse = () => {
    setIsPieceThemesCollapsed(!isPieceThemesCollapsed);
  };

  const handleVolumeChange = (volume: number) => {
    // Update the volume immediately
    onSoundVolumeChange(volume);
    
    // Clear any existing timeout
    if (volumePreviewTimeoutRef.current) {
      clearTimeout(volumePreviewTimeoutRef.current);
    }
    
    // Set a new timeout to play the preview sound after user stops dragging
    volumePreviewTimeoutRef.current = setTimeout(() => {
      playPreviewSound();
    }, 150); // 150ms delay after user stops moving the slider
  };
  
  const getFileName = (path: string): string => {
    const parts = path.split('/');
    const fileNameWithExtension = parts[parts.length - 1];
    const fileName = fileNameWithExtension.split('.')[0];
    return fileName;
  };

  const themes: Theme[] = ['light', 'dark', 'traditional', 'ocean', 'forest', 'midnight', 'sunset', 'cyberpunk', 'cherry', 'monochrome', 'sepia'];

  const renderTabContent = () => {
    switch (activeTab) {
      case 'appearance':
        return (
          <>
            <section>
              <h3 onClick={toggleColorThemeCollapse} style={{ cursor: 'pointer' }}>
                Color Theme
                <span className={`collapse-arrow ${isColorThemeCollapsed ? 'collapsed' : ''}`}>&#9660;</span>
              </h3>
              {!isColorThemeCollapsed && (
                <div className="setting-group" style={{ flexDirection: 'column', alignItems: 'stretch' }}>
                  {themes.map((themeOption) => (
                    <label key={themeOption} className="notation-option" style={{ cursor: 'pointer' }}>
                      <input
                        type="radio"
                        value={themeOption}
                        checked={theme === themeOption}
                        onChange={() => setTheme(themeOption)}
                      />
                      <div className="notation-label">
                        <span className="notation-name">{getThemeDisplayName(themeOption)}</span>
                        <span className="notation-example">{getThemeDescription(themeOption)}</span>
                      </div>
                    </label>
                  ))}
                </div>
              )}
            </section>

            <section>
              <h3 onClick={togglePieceThemesCollapse} style={{ cursor: 'pointer' }}>
                Piece Themes
                <span className={`collapse-arrow ${isPieceThemesCollapsed ? 'collapsed' : ''}`}>&#9660;</span>
              </h3>
              {!isPieceThemesCollapsed && (
                <>
                  <ThemeSelector 
                    selectedTheme={pieceThemeType} 
                    onThemeChange={onPieceThemeTypeChange} 
                  />
                  <PiecePreview theme={pieceThemeType} />
                </>
              )}
            </section>
          </>
        );

      case 'display':
        return (
          <>
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
              <h3>Show Engine Thinking</h3>
              <div className="setting-group">
                <label className="switch">
                  <input
                    type="checkbox"
                    checked={showEngineThinking}
                    onChange={(e) => onShowEngineThinkingChange(e.target.checked)}
                  />
                  <span className="slider round"></span>
                </label>
              </div>
              <p className="setting-description">Display an arrow showing the AI's current thinking move</p>
            </section>

            <section>
              <h3>Piece Movement Sounds</h3>
              <div className="setting-group">
                <label className="switch">
                  <input
                    type="checkbox"
                    checked={soundsEnabled}
                    onChange={(e) => onSoundsEnabledChange(e.target.checked)}
                  />
                  <span className="slider round"></span>
                </label>
                <span style={{ marginLeft: '10px', fontSize: '14px', color: 'var(--color-text-secondary)' }}>
                  Play clacking sound when pieces are moved
                </span>
              </div>
              {soundsEnabled && (
                <div className="setting-group" style={{ marginTop: '10px' }}>
                  <label style={{ display: 'flex', alignItems: 'center', gap: '10px', width: '100%' }}>
                    <span style={{ fontSize: '14px', color: 'var(--color-text-secondary)', minWidth: '60px' }}>
                      Volume:
                    </span>
                    <input
                      type="range"
                      min="0"
                      max="100"
                      value={Math.round(soundVolume * 100)}
                      onChange={(e) => handleVolumeChange(Number(e.target.value) / 100)}
                      style={{ flex: 1 }}
                    />
                    <span style={{ fontSize: '14px', color: 'var(--color-text-secondary)', minWidth: '40px', textAlign: 'right' }}>
                      {Math.round(soundVolume * 100)}%
                    </span>
                  </label>
                </div>
              )}
            </section>
          </>
        );

      case 'notation':
        return (
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
        );

      case 'backgrounds':
        return (
          <>
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
          </>
        );

        case 'engines':
          return (
            <>
              <section>
                <h3>Recommendation Engine</h3>
                <p className="setting-description">
                  Select the engine to use for move recommendations when hints are enabled.
                  This engine will analyze the position and suggest moves for human players.
                </p>
                <div className="setting-group">
                  <EngineSelector
                    selectedEngineId={recommendationEngineId}
                  onEngineSelect={(engineId) => {
                    console.log('[SettingsPanel] Engine selection changed:', engineId);
                    onRecommendationEngineChange(engineId);
                  }}
                    label="Recommendation Engine"
                    autoSelect={false}
                  />
                  {recommendationEngineId && (
                    <button
                      className="engine-options-btn"
                      onClick={handleOpenRecommendationEngineOptions}
                      style={{
                        marginTop: '10px',
                        padding: '8px 16px',
                        backgroundColor: 'var(--color-primary)',
                        color: 'white',
                        border: 'none',
                        borderRadius: '4px',
                        cursor: 'pointer',
                        fontSize: '14px'
                      }}
                    >
                      Engine Options
                    </button>
                  )}
                </div>
              </section>
            </>
          );

      default:
        return null;
    }
  };

  return (
    <div className="settings-overlay">
      <div className="settings-panel settings-panel-tabbed">
        <div className="settings-header">
          <h2>Settings</h2>
        </div>
        <button className="settings-close-btn" onClick={onClose}>×</button>
        
        <div className="settings-content">
          <div className="settings-tabs">
            <button 
              className={`settings-tab ${activeTab === 'appearance' ? 'active' : ''}`}
              onClick={() => setActiveTab('appearance')}
            >
              <span className="tab-icon">🎨</span>
              <span className="tab-label">Appearance</span>
            </button>
            <button 
              className={`settings-tab ${activeTab === 'display' ? 'active' : ''}`}
              onClick={() => setActiveTab('display')}
            >
              <span className="tab-icon">⚙️</span>
              <span className="tab-label">Display</span>
            </button>
            <button 
              className={`settings-tab ${activeTab === 'notation' ? 'active' : ''}`}
              onClick={() => setActiveTab('notation')}
            >
              <span className="tab-icon">📝</span>
              <span className="tab-label">Notation</span>
            </button>
            <button 
              className={`settings-tab ${activeTab === 'backgrounds' ? 'active' : ''}`}
              onClick={() => setActiveTab('backgrounds')}
            >
              <span className="tab-icon">🖼️</span>
              <span className="tab-label">Backgrounds</span>
            </button>
            <button 
              className={`settings-tab ${activeTab === 'engines' ? 'active' : ''}`}
              onClick={() => setActiveTab('engines')}
            >
              <span className="tab-icon">🤖</span>
              <span className="tab-label">Engines</span>
            </button>
          </div>
          
          <div className="settings-tab-content">
            {renderTabContent()}
          </div>
        </div>
      </div>
      
      {/* Engine Options Modal */}
      {optionsModalOpen && selectedEngine && (
        <EngineOptionsModal
          isOpen={optionsModalOpen}
          engine={selectedEngine}
          onClose={handleCloseRecommendationEngineOptions}
          onSave={handleSaveRecommendationEngineOptions}
          tempOptions={recommendationEngineOptions}
        />
      )}
    </div>
  );
};

export default SettingsPanel;