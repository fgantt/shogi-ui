/**
 * TauriGameDemo - Demonstration component showing how to integrate
 * the Tauri engine system with game logic
 * 
 * This serves as a reference implementation for tasks 4.8 and 4.9
 */

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { EngineSelector } from './EngineSelector';
import { TauriUsiMonitor } from './TauriUsiMonitor';
import { useTauriEvents } from '../hooks/useTauriEvents';
import { 
  sendUsiCommand, 
  requestEngineMove, 
  parseBestMove, 
  parseEngineInfo,
  getBuiltinEnginePath 
} from '../utils/tauriEngine';
import type { CommandResponse, EngineConfig } from '../types/engine';

export function TauriGameDemo() {
  // Engine selection
  const [selectedEngineId, setSelectedEngineId] = useState<string | null>(null);
  const [activeEngineId, setActiveEngineId] = useState<string | null>(null);
  const [engineName, setEngineName] = useState<string>('');
  
  // Game state
  const [position, setPosition] = useState<string>('startpos');
  const [moveHistory, setMoveHistory] = useState<string[]>([]);
  const [currentTurn, setCurrentTurn] = useState<'black' | 'white'>('black');
  const [isEngineThinking, setIsEngineThinking] = useState(false);
  
  // USI monitor
  const [showUsiMonitor, setShowUsiMonitor] = useState(true);
  const [lastMove, setLastMove] = useState<string | null>(null);
  const [searchInfo, setSearchInfo] = useState<string>('');

  // Listen to engine messages
  useTauriEvents(activeEngineId, {
    onUsiMessage: (engineId, message) => {
      console.log(`Engine ${engineId}: ${message}`);
      
      // Parse bestmove response
      if (message.startsWith('bestmove')) {
        const { move } = parseBestMove(message);
        if (move && move !== 'resign') {
          handleEngineMove(move);
        } else {
          console.log('Engine resigned or returned invalid move');
          setIsEngineThinking(false);
        }
      }
      
      // Parse info messages
      if (message.startsWith('info')) {
        const info = parseEngineInfo(message);
        if (info.depth !== undefined) {
          setSearchInfo(
            `Depth: ${info.depth}, Score: ${info.score || 'N/A'}, Nodes: ${info.nodes || 'N/A'}`
          );
        }
      }
      
      // Handle readyok
      if (message === 'readyok') {
        console.log('Engine is ready');
      }
    },
    onUsiError: (engineId, error) => {
      console.error(`Engine ${engineId} error: ${error}`);
      setIsEngineThinking(false);
    },
  });

  // Initialize engine when selected
  useEffect(() => {
    if (!selectedEngineId) return;

    const initializeEngine = async () => {
      try {
        // Get engine info
        const enginesResponse = await invoke<CommandResponse<EngineConfig[]>>('get_engines');
        if (enginesResponse.success && enginesResponse.data) {
          const engine = enginesResponse.data.find(e => e.id === selectedEngineId);
          if (!engine) {
            console.error('Engine not found');
            return;
          }

          // Spawn the engine
          const spawnResponse = await invoke<CommandResponse>('spawn_engine', {
            engineId: selectedEngineId,
            name: engine.name,
            path: engine.path,
            tempOptions: null,
          });

          if (!spawnResponse.success) {
            console.error('Failed to spawn engine:', spawnResponse.message);
            return;
          }

          setActiveEngineId(selectedEngineId);
          setEngineName(engine.name);

          // Initialize with USI handshake
          await sendUsiCommand(selectedEngineId, 'usinewgame');
          await sendUsiCommand(selectedEngineId, 'isready');
        }
      } catch (error) {
        console.error('Error initializing engine:', error);
      }
    };

    initializeEngine();

    // Cleanup on unmount or engine change
    return () => {
      if (activeEngineId) {
        invoke('stop_engine', { engineId: activeEngineId }).catch(console.error);
      }
    };
  }, [selectedEngineId]);

  const handleEngineMove = (move: string) => {
    console.log('Engine played move:', move);
    setLastMove(move);
    setMoveHistory(prev => [...prev, move]);
    setCurrentTurn(prev => prev === 'black' ? 'white' : 'black');
    setIsEngineThinking(false);
  };

  const handleRequestMove = async () => {
    if (!activeEngineId || isEngineThinking) return;

    setIsEngineThinking(true);
    setSearchInfo('');

    try {
      await requestEngineMove(
        activeEngineId,
        position,
        moveHistory,
        {
          btime: 30000, // 30 seconds for black
          wtime: 30000, // 30 seconds for white
          byoyomi: 5000, // 5 second byoyomi
        }
      );
    } catch (error) {
      console.error('Error requesting move:', error);
      setIsEngineThinking(false);
    }
  };

  const handleNewGame = async () => {
    if (activeEngineId) {
      await sendUsiCommand(activeEngineId, 'usinewgame');
    }
    setPosition('startpos');
    setMoveHistory([]);
    setCurrentTurn('black');
    setLastMove(null);
    setSearchInfo('');
  };

  const handleManualCommand = async (engineId: string, command: string) => {
    try {
      await sendUsiCommand(engineId, command);
    } catch (error) {
      console.error('Error sending manual command:', error);
    }
  };

  return (
    <div className="tauri-game-demo">
      <div className="demo-header">
        <h1>Tauri Engine Integration Demo</h1>
        <p>This demonstrates the integration of Tauri-based USI engine communication</p>
      </div>

      <div className="demo-content">
        {/* Engine Selection Section */}
        <section className="engine-section">
          <h2>Engine Setup</h2>
          <EngineSelector
            selectedEngineId={selectedEngineId}
            onEngineSelect={setSelectedEngineId}
            label="Choose AI Engine:"
          />
          {engineName && (
            <div className="engine-status">
              <strong>Active Engine:</strong> {engineName}
              {isEngineThinking && <span className="thinking-indicator"> (Thinking...)</span>}
            </div>
          )}
        </section>

        {/* Game Controls Section */}
        <section className="game-controls-section">
          <h2>Game Control</h2>
          <div className="game-info">
            <p><strong>Position:</strong> {position}</p>
            <p><strong>Moves:</strong> {moveHistory.length > 0 ? moveHistory.join(' ') : 'None'}</p>
            <p><strong>Current Turn:</strong> {currentTurn}</p>
            {lastMove && <p><strong>Last Move:</strong> {lastMove}</p>}
            {searchInfo && <p><strong>Search Info:</strong> {searchInfo}</p>}
          </div>

          <div className="action-buttons">
            <button 
              onClick={handleRequestMove}
              disabled={!activeEngineId || isEngineThinking}
              className="primary-button"
            >
              {isEngineThinking ? 'Engine Thinking...' : 'Request Engine Move'}
            </button>
            <button onClick={handleNewGame} disabled={isEngineThinking}>
              New Game
            </button>
          </div>
        </section>

        {/* USI Monitor Section */}
        <section className="monitor-section">
          <TauriUsiMonitor
            engineIds={activeEngineId ? [activeEngineId] : []}
            isVisible={showUsiMonitor}
            onToggle={() => setShowUsiMonitor(!showUsiMonitor)}
            onSendCommand={handleManualCommand}
          />
        </section>

        {/* Integration Notes */}
        <section className="integration-notes">
          <h2>Integration Notes</h2>
          <div className="notes-content">
            <h3>Task 4.8 - Engine Selector</h3>
            <ul>
              <li>✅ EngineSelector component created and integrated</li>
              <li>✅ Loads available engines from backend</li>
              <li>✅ Auto-selects built-in engine by default</li>
              <li>✅ Provides refresh functionality</li>
            </ul>

            <h3>Task 4.9 - Tauri Engine Communication</h3>
            <ul>
              <li>✅ Uses invoke() for all backend commands</li>
              <li>✅ spawn_engine, send_usi_command, stop_engine</li>
              <li>✅ Event listeners via useTauriEvents hook</li>
              <li>✅ Parses bestmove and info responses</li>
              <li>✅ Handles engine lifecycle (spawn → initialize → use → stop)</li>
            </ul>

            <h3>Integration Pattern for GamePage</h3>
            <pre className="code-block">{`
// 1. Add EngineSelector to game setup
<EngineSelector
  selectedEngineId={selectedEngineId}
  onEngineSelect={setSelectedEngineId}
/>

// 2. Initialize engine on selection
useEffect(() => {
  if (selectedEngineId) {
    initializeEngine(selectedEngineId);
  }
}, [selectedEngineId]);

// 3. Use Tauri events for engine responses
useTauriEvents(activeEngineId, {
  onUsiMessage: handleEngineMessage,
  onUsiError: handleEngineError,
});

// 4. Request moves via Tauri
await requestEngineMove(
  engineId,
  position,
  moveHistory,
  timeControl
);
            `}</pre>
          </div>
        </section>
      </div>

      <style>{`
        .tauri-game-demo {
          padding: 2rem;
          max-width: 1000px;
          margin: 0 auto;
        }
        .demo-header {
          text-align: center;
          margin-bottom: 2rem;
        }
        .demo-header h1 {
          margin-bottom: 0.5rem;
          color: #333;
        }
        .demo-header p {
          color: #666;
        }
        .demo-content {
          display: flex;
          flex-direction: column;
          gap: 2rem;
        }
        .engine-section,
        .game-controls-section,
        .monitor-section,
        .integration-notes {
          background: white;
          padding: 1.5rem;
          border-radius: 8px;
          box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }
        .engine-section h2,
        .game-controls-section h2,
        .integration-notes h2 {
          margin-top: 0;
          margin-bottom: 1rem;
          color: #444;
        }
        .engine-status {
          margin-top: 1rem;
          padding: 0.8rem;
          background: #e8f5e9;
          border-radius: 4px;
          color: #2e7d32;
        }
        .thinking-indicator {
          font-style: italic;
          color: #1976d2;
        }
        .game-info {
          background: #f9f9f9;
          padding: 1rem;
          border-radius: 4px;
          margin-bottom: 1rem;
        }
        .game-info p {
          margin: 0.5rem 0;
          font-family: 'Courier New', monospace;
          font-size: 13px;
        }
        .action-buttons {
          display: flex;
          gap: 0.5rem;
        }
        .action-buttons button {
          padding: 0.7rem 1.5rem;
          border-radius: 4px;
          border: 1px solid #ddd;
          background: white;
          cursor: pointer;
          transition: all 0.2s;
        }
        .action-buttons .primary-button {
          background: #4a90e2;
          color: white;
          border-color: #4a90e2;
        }
        .action-buttons .primary-button:hover:not(:disabled) {
          background: #357abd;
        }
        .action-buttons button:disabled {
          opacity: 0.6;
          cursor: not-allowed;
        }
        .notes-content {
          color: #555;
        }
        .notes-content h3 {
          color: #333;
          margin-top: 1.5rem;
          margin-bottom: 0.5rem;
        }
        .notes-content ul {
          margin: 0.5rem 0;
          padding-left: 1.5rem;
        }
        .notes-content li {
          margin: 0.3rem 0;
        }
        .code-block {
          background: #f5f5f5;
          padding: 1rem;
          border-radius: 4px;
          overflow-x: auto;
          font-size: 12px;
          line-height: 1.5;
        }
      `}</style>
    </div>
  );
}

