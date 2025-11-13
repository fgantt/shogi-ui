import React, { useState, useEffect, useCallback } from 'react';
import { useMultipleEngineEvents } from '../hooks/useTauriEvents';
import { parseEngineInfo } from '../utils/tauriEngine';
import type { EngineConfig } from '../types/engine';
import './UsiMonitor.css';
import { loadUsiMonitorState, saveUsiMonitorState } from '../utils/persistence';

interface UsiMessage {
  id: string;
  timestamp: Date;
  direction: 'sent' | 'received';
  message: string;
  engineId: string;
}

interface SearchInfo {
  id: string;
  timestamp: Date;
  elapsed: number; // time in seconds
  rank: number; // multipv rank
  depth: number;
  seldepth?: number;
  nodes: number;
  score: number;
  pv: string; // space-separated move list
}

type TabType = 'engine' | 'search';

interface TauriUsiMonitorProps {
  engineIds: string[];
  engines?: EngineConfig[];  // Optional: for displaying engine names
  engineNames?: Map<string, string>;  // Map from runtime ID to engine display name
  isVisible: boolean;
  onToggle: () => void;
  onSendCommand?: (engineId: string, command: string) => void;
  player1EngineId?: string | null;  // Runtime ID of player 1's engine (if any)
  player2EngineId?: string | null;  // Runtime ID of player 2's engine (if any)
  player1Type?: 'human' | 'ai';  // Type of player 1
  player2Type?: 'human' | 'ai';  // Type of player 2
}

export const TauriUsiMonitor: React.FC<TauriUsiMonitorProps> = ({
  engineIds,
  engines,
  engineNames,
  isVisible,
  onToggle,
  onSendCommand,
  player1EngineId,
  player2EngineId,
  player1Type,
  player2Type,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>(() => loadUsiMonitorState().activeTab);
  const [communicationHistory, setCommunicationHistory] = useState<UsiMessage[]>([]);
  const [searchInfoByEngine, setSearchInfoByEngine] = useState<Map<string, SearchInfo[]>>(new Map());
  const [manualCommands, setManualCommands] = useState<Map<string, string>>(new Map());
  const [npsPerEngine, setNpsPerEngine] = useState<Map<string, number>>(new Map());
  const setActiveTabWithPersistence = (tab: TabType) => {
    setActiveTab(tab);
    saveUsiMonitorState({ activeTab: tab });
  };


  // Helper to get engine display name
  const getEngineDisplayName = (engineId: string): string => {
    // First try the runtime ID mapping
    if (engineNames) {
      const name = engineNames.get(engineId);
      if (name) return name;
    }
    // Fall back to config ID lookup
    if (!engines) return engineId.substring(0, 8) + '...';
    const engine = engines.find(e => e.id === engineId);
    return engine?.display_name || engineId.substring(0, 8) + '...';
  };

  // Helper to get player name from engine ID
  // Memoized to avoid recalculation on every render
  const getPlayerFromEngineId = useCallback((engineId: string): string => {
    // Check if this engine is player 1's engine (Sente)
    if (player1EngineId === engineId) {
      return 'Sente';
    }
    // Check if this engine is player 2's engine (Gote)
    if (player2EngineId === engineId) {
      return 'Gote';
    }
    
    // Fallback: infer from the order in engineIds (for backwards compatibility)
    const index = engineIds.indexOf(engineId);
    if (index === -1) return 'Unknown';
    return index === 0 ? 'Sente' : 'Gote';
  }, [player1EngineId, player2EngineId, engineIds]);

  // Listen to sent command events
  useEffect(() => {
    const handleCommandSent = (engineId: string) => (event: Event) => {
      const customEvent = event as CustomEvent<{ command: string }>;
      const command = customEvent.detail.command;
      
      const newMessage: UsiMessage = {
        id: `sent-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'sent',
        message: command,
        engineId,
      };
      setCommunicationHistory(prev => [...prev, newMessage]);

      // Do not clear search info on new "go"; keep last results visible until replaced by new info
    };

    // Register listeners for each engine
    const listeners: Array<{ engineId: string; handler: (event: Event) => void }> = [];
    engineIds.forEach(engineId => {
      const handler = handleCommandSent(engineId);
      window.addEventListener(`usi-command-sent::${engineId}`, handler);
      listeners.push({ engineId, handler });
    });

    // Cleanup
    return () => {
      listeners.forEach(({ engineId, handler }) => {
        window.removeEventListener(`usi-command-sent::${engineId}`, handler);
      });
    };
  }, [engineIds]);

  // Memoize callbacks to prevent unnecessary re-renders
  const handleUsiMessage = useCallback((engineId: string, message: string) => {
    const newMessage: UsiMessage = {
      id: `received-${Date.now()}-${Math.random()}`,
      timestamp: new Date(),
      direction: 'received',
      message,
      engineId,
    };
    setCommunicationHistory(prev => [...prev, newMessage]);

    // Parse info messages for search tab
    if (message.startsWith('info ')) {
      const info = parseEngineInfo(message);
      
      // Update NPS if available
      if (info.nps !== undefined) {
        setNpsPerEngine(prev => {
          const newMap = new Map(prev);
          newMap.set(engineId, info.nps!);
          return newMap;
        });
      } else if (info.nodes !== undefined && info.time !== undefined && info.time > 0) {
        // Calculate NPS if not provided directly
        const calculatedNps = Math.round((info.nodes * 1000) / info.time);
        setNpsPerEngine(prev => {
          const newMap = new Map(prev);
          newMap.set(engineId, calculatedNps);
          return newMap;
        });
      }
      
      // Only add to search info if it has the key fields
      if (info.depth !== undefined && info.score !== undefined && info.nodes !== undefined) {
        const searchInfo: SearchInfo = {
          id: `search-${Date.now()}-${Math.random()}`,
          timestamp: new Date(),
          elapsed: info.time !== undefined ? info.time / 1000 : 0, // Convert ms to seconds
          rank: info.multipv || 1,
          depth: info.depth,
          seldepth: info.seldepth,
          nodes: info.nodes,
          score: info.score,
          pv: info.pv ? info.pv.join(' ') : '',
        };

        setSearchInfoByEngine(prev => {
          const newMap = new Map(prev);
          const engineInfo = newMap.get(engineId) || [];
          // Add to the beginning (newest first)
          newMap.set(engineId, [searchInfo, ...engineInfo]);
          return newMap;
        });
      }
    }

    // Check for bestmove to clear search info for new move
    if (message.startsWith('bestmove')) {
      // Don't clear immediately - keep the last move's info visible
      // We'll clear when a new "go" command is sent
    }
  }, []);

  const handleUsiError = useCallback((engineId: string, error: string) => {
    const errorMessage: UsiMessage = {
      id: `error-${Date.now()}-${Math.random()}`,
      timestamp: new Date(),
      direction: 'received',
      message: `ERROR: ${error}`,
      engineId,
    };
    setCommunicationHistory(prev => [...prev, errorMessage]);
  }, []);

  // Listen to all engine messages
  useMultipleEngineEvents(engineIds, {
    onUsiMessage: handleUsiMessage,
    onUsiError: handleUsiError,
  });

  const handleSendCommand = (engineId: string) => {
    const command = manualCommands.get(engineId);
    if (!command?.trim() || !onSendCommand) return;
    
    // The onSendCommand callback (sendUsiCommand) will emit an event that we listen to
    onSendCommand(engineId, command.trim());
    
    // Clear the input for this engine
    setManualCommands(prev => {
      const newMap = new Map(prev);
      newMap.set(engineId, '');
      return newMap;
    });
  };

  const updateManualCommand = (engineId: string, value: string) => {
    setManualCommands(prev => {
      const newMap = new Map(prev);
      newMap.set(engineId, value);
      return newMap;
    });
  };

  // Get last sent and received for each engine
  const getLastSent = (engineId: string): { timestamp: Date | null; message: string } => {
    const messages = communicationHistory.filter(
      entry => entry.engineId === engineId && entry.direction === 'sent'
    );
    const last = messages[messages.length - 1];
    return last 
      ? { timestamp: last.timestamp, message: last.message }
      : { timestamp: null, message: 'None' };
  };

  const getLastReceived = (engineId: string): { timestamp: Date | null; message: string } => {
    const messages = communicationHistory.filter(
      entry => entry.engineId === engineId && entry.direction === 'received'
    );
    const last = messages[messages.length - 1];
    return last 
      ? { timestamp: last.timestamp, message: last.message }
      : { timestamp: null, message: 'None' };
  };

  const formatTimestamp = (timestamp: Date | null): string => {
    if (!timestamp) return 'N/A';
    return timestamp.toLocaleTimeString();
  };

  const formatNumber = (num: number): string => {
    return num.toLocaleString();
  };

  // Format PV with player symbols
  const formatPV = (pv: string, engineId: string): JSX.Element[] => {
    if (!pv) return [];
    
    const moves = pv.split(' ').filter(m => m.trim());
    const playerRole = getPlayerFromEngineId(engineId);
    const isSenteEngine = playerRole === 'Sente';
    
    return moves.map((move, index) => {
      // Determine which player makes this move
      // For Sente engine: first move is Sente (☗), then Gote (☖), etc.
      // For Gote engine: first move is Gote (☖), then Sente (☗), etc.
      const isSenteMove = isSenteEngine ? index % 2 === 0 : index % 2 === 1;
      const symbol = isSenteMove ? '☗' : '☖';
      
      return (
        <span key={index} className="pv-move">
          <span className={isSenteMove ? 'sente-symbol' : 'gote-symbol'}>{symbol}</span>
          {move}
        </span>
      );
    });
  };

  if (!isVisible) {
    return (
      <div className="usi-monitor-toggle">
        <button onClick={onToggle} className="toggle-button">
          USI Monitor
        </button>
      </div>
    );
  }

  return (
    <div className="usi-monitor">
      <div className="usi-monitor-header">
        <h3>USI Communication Monitor</h3>
        <button onClick={onToggle} className="close-button">
          ×
        </button>
      </div>
      
      <div className="usi-monitor-tabs">
        <button
          className={`tab-button ${activeTab === 'engine' ? 'active' : ''}`}
          onClick={() => setActiveTabWithPersistence('engine')}
        >
          Engine Monitor
        </button>
        <button
          className={`tab-button ${activeTab === 'search' ? 'active' : ''}`}
          onClick={() => setActiveTabWithPersistence('search')}
        >
          Search
        </button>
      </div>

      <div className="usi-monitor-content">
        {activeTab === 'engine' && (
          <div className="engine-monitor-tab">
            {engineIds.map((engineId, index) => {
              const lastSent = getLastSent(engineId);
              const lastReceived = getLastReceived(engineId);
              const player = getPlayerFromEngineId(engineId);
              
              return (
                <div key={engineId} className="engine-section">
                  {index > 0 && <div className="engine-divider" />}
                  
                  <div className="engine-header">
                    <strong>{player}:</strong> {getEngineDisplayName(engineId)}
                  </div>

                  <div className="last-command-item">
                    <label>Last Sent: {formatTimestamp(lastSent.timestamp)}</label>
                    <div className="command-text sent-command">
                      {lastSent.message}
                    </div>
                  </div>

                  <div className="last-command-item">
                    <label>Last Received: {formatTimestamp(lastReceived.timestamp)}</label>
                    <div className="command-text received-command">
                      {lastReceived.message}
                    </div>
                  </div>

                  {onSendCommand && (
                    <div className="manual-command">
                      <input
                        type="text"
                        placeholder="Enter USI command..."
                        value={manualCommands.get(engineId) || ''}
                        onChange={(e) => updateManualCommand(engineId, e.target.value)}
                        onKeyPress={(e) => e.key === 'Enter' && handleSendCommand(engineId)}
                      />
                      <button 
                        onClick={() => handleSendCommand(engineId)} 
                        disabled={!manualCommands.get(engineId)?.trim()}
                      >
                        SEND
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}

        {activeTab === 'search' && (
          <div className="search-tab">
            {engineIds.map((engineId, index) => {
              const searchInfo = searchInfoByEngine.get(engineId) || [];
              const player = getPlayerFromEngineId(engineId);
              const nps = npsPerEngine.get(engineId);
              
              return (
                <div key={engineId} className="engine-section">
                  {index > 0 && <div className="engine-divider" />}
                  
                  <div className="engine-header">
                    <div>
                      <strong>{player}:</strong> {getEngineDisplayName(engineId)}
                    </div>
                    {nps !== undefined && (
                      <div className="engine-nps">
                        NPS: {formatNumber(nps)}
                      </div>
                    )}
                  </div>

                  {searchInfo.length === 0 ? (
                    <div className="no-search-info">No search information available</div>
                  ) : (
                    <div className="search-table-container">
                      <table className="search-table">
                        <thead>
                          <tr>
                            <th>Elapsed</th>
                            <th>Rank</th>
                            <th>Depth</th>
                            <th>Nodes</th>
                            <th>Score</th>
                            <th>PV</th>
                          </tr>
                        </thead>
                        <tbody>
                          {searchInfo.map((info) => (
                            <tr key={info.id}>
                              <td>{info.elapsed.toFixed(1)}s</td>
                              <td>{info.rank}</td>
                              <td>
                                {info.depth}
                                {info.seldepth !== undefined && `/${info.seldepth}`}
                              </td>
                              <td>{formatNumber(info.nodes)}</td>
                              <td className={info.score > 0 ? 'score-positive' : info.score < 0 ? 'score-negative' : ''}>
                                {info.score > 0 ? '+' : ''}{info.score}
                              </td>
                              <td className="pv-cell">{formatPV(info.pv, engineId)}</td>
                            </tr>
                          ))}
                        </tbody>
                      </table>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>
    </div>
  );
};

