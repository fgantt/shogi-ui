import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { EngineSelector } from './EngineSelector';
import type { CommandResponse } from '../types/engine';
import './EngineVsEnginePage.css';

interface EngineVsEngineState {
  move_number: number;
  current_player: string;
  position_sfen: string;
  last_move: string | null;
  move_history: string[];
  game_over: boolean;
  winner: string | null;
  game_result: string | null;
}

export function EngineVsEnginePage() {
  const [engine1Id, setEngine1Id] = useState<string | null>(null);
  const [engine2Id, setEngine2Id] = useState<string | null>(null);
  const [timePerMove, setTimePerMove] = useState(5);
  const [maxMoves, setMaxMoves] = useState(200);
  const [gameState, setGameState] = useState<EngineVsEngineState | null>(null);
  const [isRunning, setIsRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let unlistenUpdate: UnlistenFn | null = null;
    let unlistenMove: UnlistenFn | null = null;

    const setupListeners = async () => {
      // Listen for game state updates
      unlistenUpdate = await listen<EngineVsEngineState>('engine-vs-engine-update', (event) => {
        console.log('Game state update:', event.payload);
        setGameState(event.payload);
        
        if (event.payload.game_over) {
          setIsRunning(false);
        }
      });

      // Listen for individual moves
      unlistenMove = await listen<any>('engine-vs-engine-move', (event) => {
        console.log('Move played:', event.payload);
      });
    };

    setupListeners();

    return () => {
      if (unlistenUpdate) unlistenUpdate();
      if (unlistenMove) unlistenMove();
    };
  }, []);

  const handleStartMatch = async () => {
    if (!engine1Id || !engine2Id) {
      setError('Please select both engines');
      return;
    }

    if (engine1Id === engine2Id) {
      setError('Please select different engines for each player');
      return;
    }

    try {
      setError(null);
      setGameState(null);
      setIsRunning(true);

      const response = await invoke<CommandResponse>('start_engine_vs_engine', {
        engine1Id,
        engine2Id,
        initialSfen: null,
        timePerMoveMs: timePerMove * 1000,
        maxMoves,
      });

      if (!response.success) {
        setError(response.message || 'Failed to start match');
        setIsRunning(false);
      }
    } catch (err) {
      setError(`Error starting match: ${err}`);
      setIsRunning(false);
    }
  };

  const getResultDisplay = () => {
    if (!gameState?.game_over) return null;

    const { winner, game_result } = gameState;
    let resultText = '';
    let resultClass = '';

    if (winner === 'black') {
      resultText = '⚫ Black Wins!';
      resultClass = 'result-black-wins';
    } else if (winner === 'white') {
      resultText = '⚪ White Wins!';
      resultClass = 'result-white-wins';
    } else {
      resultText = '🤝 Draw';
      resultClass = 'result-draw';
    }

    return (
      <div className={`game-result ${resultClass}`}>
        <h2>{resultText}</h2>
        {game_result && <p>{game_result}</p>}
      </div>
    );
  };

  return (
    <div className="engine-vs-engine-page">
      <h1>Engine vs Engine Spectator Mode</h1>
      <p className="page-description">
        Watch two engines battle it out in an automated match!
      </p>

      {error && (
        <div className="error-banner">
          {error}
          <button onClick={() => setError(null)}>✕</button>
        </div>
      )}

      <div className="setup-panel">
        <h2>Match Setup</h2>

        <div className="engine-selection">
          <div className="engine-select-group">
            <h3>⚫ Black (Engine 1)</h3>
            <EngineSelector
              selectedEngineId={engine1Id}
              onEngineSelect={setEngine1Id}
              label="Select Engine:"
            />
          </div>

          <div className="versus-separator">
            <span>VS</span>
          </div>

          <div className="engine-select-group">
            <h3>⚪ White (Engine 2)</h3>
            <EngineSelector
              selectedEngineId={engine2Id}
              onEngineSelect={setEngine2Id}
              label="Select Engine:"
            />
          </div>
        </div>

        <div className="match-settings">
          <div className="setting-group">
            <label>Time per move (seconds):</label>
            <input
              type="number"
              min="1"
              max="60"
              value={timePerMove}
              onChange={(e) => setTimePerMove(parseInt(e.target.value) || 5)}
              disabled={isRunning}
            />
          </div>

          <div className="setting-group">
            <label>Max moves:</label>
            <input
              type="number"
              min="50"
              max="500"
              value={maxMoves}
              onChange={(e) => setMaxMoves(parseInt(e.target.value) || 200)}
              disabled={isRunning}
            />
          </div>
        </div>

        <button
          onClick={handleStartMatch}
          disabled={isRunning || !engine1Id || !engine2Id}
          className="start-match-button"
        >
          {isRunning ? 'Match in Progress...' : 'Start Match'}
        </button>
      </div>

      {gameState && (
        <div className="game-state-panel">
          <h2>Match Status</h2>

          {getResultDisplay()}

          <div className="state-info">
            <div className="info-row">
              <span className="info-label">Move Number:</span>
              <span className="info-value">{gameState.move_number}</span>
            </div>
            <div className="info-row">
              <span className="info-label">Current Player:</span>
              <span className="info-value">
                {gameState.current_player === 'black' ? '⚫ Black' : '⚪ White'}
              </span>
            </div>
            {gameState.last_move && (
              <div className="info-row">
                <span className="info-label">Last Move:</span>
                <span className="info-value move-text">{gameState.last_move}</span>
              </div>
            )}
          </div>

          {gameState.move_history.length > 0 && (
            <div className="move-history">
              <h3>Move History ({gameState.move_history.length} moves)</h3>
              <div className="move-list">
                {gameState.move_history.map((move, index) => (
                  <div key={index} className="move-item">
                    <span className="move-number">{index + 1}.</span>
                    <span className="move-player">
                      {index % 2 === 0 ? '⚫' : '⚪'}
                    </span>
                    <span className="move-text">{move}</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          <div className="position-sfen">
            <h3>Current Position (SFEN)</h3>
            <code>{gameState.position_sfen}</code>
          </div>
        </div>
      )}

      {!gameState && !isRunning && (
        <div className="waiting-state">
          <p>Configure the match settings above and click "Start Match" to begin.</p>
        </div>
      )}
    </div>
  );
}

