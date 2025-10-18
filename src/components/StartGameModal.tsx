import React, { useState } from 'react';
import { GameSettings } from '../types';
import { Record } from 'tsshogi';
import { EngineSelector } from './EngineSelector';
import { EngineOptionsModal } from './EngineOptionsModal';
import type { EngineConfig } from '../types/engine';
import { invoke } from '@tauri-apps/api/core';

interface StartGameModalProps {
  isOpen: boolean;
  onClose: () => void;
  onStartGame: (settings: GameSettings) => void;
}

// Canned starting positions
const CANNED_POSITIONS = [
  { name: 'Standard', sfen: 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1' },
  { name: 'Lance', sfen: 'lnsgkgsn1/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Right Lance', sfen: '1nsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Bishop', sfen: 'lnsgkgsnl/1r7/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Rook', sfen: 'lnsgkgsnl/7b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Rook-Lance', sfen: 'lnsgkgsn1/7b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: '2-piece', sfen: 'lnsgkgsnl/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: '4-piece', sfen: '1nsgkgsn1/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: '6-piece', sfen: '2sgkgs2/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: '8-piece', sfen: '3gkg3/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: '10-piece', sfen: '4k4/9/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Dragonfly', sfen: '4k4/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Dragonfly + L', sfen: 'l3k3l/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Dragonfly + NL', sfen: 'ln2k2nl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w - 1' },
  { name: 'Lance Gained', sfen: 'lnsgkgsn1/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w L 1' },
  { name: 'Bishop Gained', sfen: 'lnsgkgsnl/1r7/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w B 1' },
  { name: 'Rook Gained', sfen: 'lnsgkgsnl/7b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w R 1' },
  { name: 'Rook-Lance Gained', sfen: 'lnsgkgsn1/7b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL w RL 1' }
];

const StartGameModal: React.FC<StartGameModalProps> = ({ isOpen, onClose, onStartGame }) => {
  const [player1Type, setPlayer1Type] = useState<'human' | 'ai'>('human');
  const [player2Type, setPlayer2Type] = useState<'human' | 'ai'>('ai');
  const [player1EngineId, setPlayer1EngineId] = useState<string | null>(null);
  const [player2EngineId, setPlayer2EngineId] = useState<string | null>(null);
  const [initialSfen, setInitialSfen] = useState<string>('');
  const [sfenError, setSfenError] = useState<string>('');
  const [selectedCannedPosition, setSelectedCannedPosition] = useState<string>('Standard');
  const [isInitialPositionCollapsed, setIsInitialPositionCollapsed] = useState(true);
  
  // Temporary options for this game only
  const [player1TempOptions, setPlayer1TempOptions] = useState<{[key: string]: string} | null>(null);
  const [player2TempOptions, setPlayer2TempOptions] = useState<{[key: string]: string} | null>(null);
  
  // Options modal state
  const [optionsModalOpen, setOptionsModalOpen] = useState(false);
  const [optionsModalPlayer, setOptionsModalPlayer] = useState<1 | 2 | null>(null);
  const [selectedEngine, setSelectedEngine] = useState<EngineConfig | null>(null);
  const [engines, setEngines] = useState<EngineConfig[]>([]);

  // Load engines when modal opens
  React.useEffect(() => {
    if (isOpen) {
      loadEngines();
    }
  }, [isOpen]);

  // Reset form when modal opens
  React.useEffect(() => {
    if (isOpen) {
      setPlayer1Type('human');
      setPlayer2Type('ai');
      setPlayer1EngineId(null);
      setPlayer2EngineId(null);
      setPlayer1TempOptions(null);
      setPlayer2TempOptions(null);
      setInitialSfen('');
      setSfenError('');
      setSelectedCannedPosition('Standard');
      setIsInitialPositionCollapsed(true);
    }
  }, [isOpen]);

  const loadEngines = async () => {
    try {
      const response = await invoke<any>('get_engines');
      if (response.success && response.data) {
        setEngines(response.data);
      }
    } catch (error) {
      console.error('Error loading engines:', error);
    }
  };

  const handleOpenOptions = (player: 1 | 2, engineId: string | null) => {
    if (!engineId) return;
    
    const engine = engines.find(e => e.id === engineId);
    if (engine) {
      setSelectedEngine(engine);
      setOptionsModalPlayer(player);
      setOptionsModalOpen(true);
    }
  };

  const handleCloseOptions = () => {
    setOptionsModalOpen(false);
    setOptionsModalPlayer(null);
    setSelectedEngine(null);
  };

  const handleSaveOptions = (options: {[key: string]: string}) => {
    if (optionsModalPlayer === 1) {
      setPlayer1TempOptions(options);
    } else if (optionsModalPlayer === 2) {
      setPlayer2TempOptions(options);
    }
    handleCloseOptions();
  };

  const toggleInitialPositionCollapse = () => {
    setIsInitialPositionCollapsed(!isInitialPositionCollapsed);
  };

  if (!isOpen) return null;

  const validateSfen = (sfen: string): string => {
    if (!sfen.trim()) {
      return ''; // Empty SFEN is valid (will use default startpos)
    }
    
    try {
      const recordResult = Record.newByUSI(`sfen ${sfen}`);
      if (recordResult instanceof Error) {
        return recordResult.message;
      }
      return '';
    } catch (error) {
      return 'Invalid SFEN format';
    }
  };

  const handleSfenChange = (value: string) => {
    setInitialSfen(value);
    const error = validateSfen(value);
    setSfenError(error);
    
    // Reset canned position selection when user manually edits SFEN
    if (selectedCannedPosition !== 'Custom') {
      setSelectedCannedPosition('Custom');
    }
  };

  const handleCannedPositionChange = (positionName: string) => {
    setSelectedCannedPosition(positionName);
    
    if (positionName === 'Standard') {
      // Clear SFEN for standard position
      setInitialSfen('');
      setSfenError('');
    } else {
      // Set SFEN for canned position
      const cannedPosition = CANNED_POSITIONS.find(pos => pos.name === positionName);
      if (cannedPosition) {
        setInitialSfen(cannedPosition.sfen);
        const error = validateSfen(cannedPosition.sfen);
        setSfenError(error);
      }
    }
  };

  const handleSubmit = (event: React.FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    
    // Validate SFEN before submitting
    const error = validateSfen(initialSfen);
    if (error) {
      setSfenError(error);
      return;
    }

    const formData = new FormData(event.currentTarget);
    const settings: GameSettings = {
      player1Type: formData.get('player1Type') as GameSettings['player1Type'],
      player2Type: formData.get('player2Type') as GameSettings['player2Type'],
      minutesPerSide: parseInt(formData.get('minutesPerSide') as string, 10) || 30,
      byoyomiInSeconds: parseInt(formData.get('byoyomiInSeconds') as string, 10) || 10,
      initialSfen: (selectedCannedPosition === 'Standard' || !initialSfen.trim()) ? undefined : initialSfen.trim(),
      player1EngineId,
      player2EngineId,
      player1TempOptions: player1TempOptions || undefined,
      player2TempOptions: player2TempOptions || undefined,
      useTauriEngine: !!(player1EngineId || player2EngineId),
    };
    onStartGame(settings);
    onClose();
  };

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>New Game</h2>
        <button className="settings-close-btn" onClick={onClose}>×</button>
        <form onSubmit={handleSubmit}>
          <section>
            <h3>Player 1 (Black)</h3>
            <div className="setting-group">
              <select id="player1Type" name="player1Type" defaultValue="human" onChange={(e) => setPlayer1Type(e.target.value as 'human' | 'ai')}>
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
              {player1Type === 'ai' && (
                <>
                  <EngineSelector
                    selectedEngineId={player1EngineId}
                    onEngineSelect={setPlayer1EngineId}
                    label="AI Engine:"
                  />
                  {player1EngineId && (
                    <button
                      type="button"
                      onClick={() => handleOpenOptions(1, player1EngineId)}
                      className="options-button-inline"
                      style={{ marginTop: '0.5rem' }}
                    >
                      ⚙️ Engine Options {player1TempOptions ? '(Modified)' : ''}
                    </button>
                  )}
                </>
              )}
            </div>
          </section>
          <section>
            <h3>Player 2 (White)</h3>
            <div className="setting-group">
              <select id="player2Type" name="player2Type" defaultValue="ai" onChange={(e) => setPlayer2Type(e.target.value as 'human' | 'ai')}>
                <option value="human">Human</option>
                <option value="ai">AI</option>
              </select>
              {player2Type === 'ai' && (
                <>
                  <EngineSelector
                    selectedEngineId={player2EngineId}
                    onEngineSelect={setPlayer2EngineId}
                    label="AI Engine:"
                  />
                  {player2EngineId && (
                    <button
                      type="button"
                      onClick={() => handleOpenOptions(2, player2EngineId)}
                      className="options-button-inline"
                      style={{ marginTop: '0.5rem' }}
                    >
                      ⚙️ Engine Options {player2TempOptions ? '(Modified)' : ''}
                    </button>
                  )}
                </>
              )}
            </div>
          </section>
          <section>
            <h3>Time Controls</h3>
            <div className="setting-group">
              <label htmlFor="minutesPerSide">Minutes per side</label>
              <input id="minutesPerSide" name="minutesPerSide" type="number" min="1" defaultValue="30" />
            </div>
            <div className="setting-group">
              <label htmlFor="byoyomiInSeconds">Byoyomi in seconds</label>
              <input id="byoyomiInSeconds" name="byoyomiInSeconds" type="number" min="0" defaultValue="10" />
            </div>
          </section>
          <section>
            <h3 onClick={toggleInitialPositionCollapse} style={{ cursor: 'pointer' }}>
              Initial Position (Optional)
              <span className={`collapse-arrow ${isInitialPositionCollapsed ? 'collapsed' : ''}`}>&#9660;</span>
            </h3>
            {!isInitialPositionCollapsed && (
              <>
                <div className="setting-group">
                  <label htmlFor="cannedPosition">Starting Position</label>
                  <select 
                    id="cannedPosition"
                    value={selectedCannedPosition}
                    onChange={(e) => handleCannedPositionChange(e.target.value)}
                  >
                    {CANNED_POSITIONS.map((position) => (
                      <option key={position.name} value={position.name}>
                        {position.name}
                      </option>
                    ))}
                    <option value="Custom">Custom SFEN</option>
                  </select>
                </div>
                <div className="setting-group">
                  <label htmlFor="initialSfen">SFEN String</label>
                  <input 
                    id="initialSfen" 
                    name="initialSfen" 
                    type="text" 
                    value={initialSfen}
                    onChange={(e) => handleSfenChange(e.target.value)}
                    placeholder={selectedCannedPosition === 'Standard' ? 'Leave empty for standard starting position' : 'Enter or edit SFEN string'}
                    className={sfenError ? 'error' : ''}
                  />
                  {sfenError && <div className="error-message">{sfenError}</div>}
                  <div className="help-text">
                    {selectedCannedPosition === 'Standard' 
                      ? 'Leave empty to use the standard starting position, or enter a custom SFEN string.'
                      : selectedCannedPosition === 'Custom'
                        ? 'Enter a SFEN string to start from a custom position.'
                        : 'You can edit this SFEN string or select a different starting position.'
                    }
                  </div>
                </div>
              </>
            )}
          </section>
          <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
            <button type="button" onClick={onClose}>Cancel</button>
            <button type="submit">Start Game</button>
          </div>
        </form>
      </div>
      
      {/* Engine Options Modal - reused from EngineManagementPage */}
      <EngineOptionsModal
        isOpen={optionsModalOpen}
        engine={selectedEngine}
        onClose={handleCloseOptions}
        onSave={handleSaveOptions}
        tempOptions={optionsModalPlayer === 1 ? player1TempOptions : player2TempOptions}
      />
    </div>
  );
};

export default StartGameModal;
