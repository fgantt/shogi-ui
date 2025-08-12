import React, { useState, useEffect } from 'react';
import { 
  getWasmAiMove, 
  isWasmEngineAvailable, 
  getEngineStats, 
  getEngineCapabilities,
  benchmarkEngines,
  getPerformanceMetrics,
  resetEngine
} from '../ai/wasmEngine.js';
import { 
  getEngineStatus, 
  compareEnginePerformance, 
  getEngineRecommendations 
} from '../ai/computerPlayer.js';
import './WebAssemblyDemo.css';

const WebAssemblyDemo = () => {
  const [engineStatus, setEngineStatus] = useState(null);
  const [testResults, setTestResults] = useState(null);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState(null);

  // Mock game state for testing
  const mockGameState = {
    board: [
      // Row 0 (top - player2)
      [
        { type: 'L', player: 'player2' }, // Lance
        { type: 'N', player: 'player2' }, // Knight
        { type: 'S', player: 'player2' }, // Silver
        { type: 'G', player: 'player2' }, // Gold
        { type: 'K', player: 'player2' }, // King
        { type: 'G', player: 'player2' }, // Gold
        { type: 'S', player: 'player2' }, // Silver
        { type: 'N', player: 'player2' }, // Knight
        { type: 'L', player: 'player2' }  // Lance
      ],
      // Row 1
      [
        null,
        { type: 'R', player: 'player2' }, // Rook
        null, null, null, null, null,
        { type: 'B', player: 'player2' }, // Bishop
        null
      ],
      // Row 2
      Array(9).fill(null).map(() => ({ type: 'P', player: 'player2' })), // Pawns
      // Row 3 (empty)
      Array(9).fill(null),
      // Row 4 (empty)
      Array(9).fill(null),
      // Row 5 (empty)
      Array(9).fill(null),
      // Row 6
      Array(9).fill(null).map(() => ({ type: 'P', player: 'player1' })), // Pawns
      // Row 7
      [
        null,
        { type: 'B', player: 'player1' }, // Bishop
        null, null, null, null, null,
        { type: 'R', player: 'player1' }, // Rook
        null
      ],
      // Row 8 (bottom - player1)
      [
        { type: 'L', player: 'player1' }, // Lance
        { type: 'N', player: 'player1' }, // Knight
        { type: 'S', player: 'player1' }, // Silver
        { type: 'G', player: 'player1' }, // Gold
        { type: 'K', player: 'player1' }, // King
        { type: 'G', player: 'player1' }, // Gold
        { type: 'S', player: 'player1' }, // Silver
        { type: 'N', player: 'player1' }, // Knight
        { type: 'L', player: 'player1' }  // Lance
      ]
    ],
    currentPlayer: 'player1',
    capturedPieces: {
      player1: [],
      player2: []
    },
    moveHistory: [],
    isCheck: false,
    isCheckmate: false
  };

  useEffect(() => {
    checkEngineStatus();
  }, []);

  const checkEngineStatus = async () => {
    try {
      const status = getEngineStatus();
      setEngineStatus(status);
    } catch (error) {
      setError('Failed to check engine status: ' + error.message);
    }
  };

  const initializeWebAssembly = async () => {
    try {
      setIsLoading(true);
      setError(null);
      
      console.log('Initializing WebAssembly engine...');
      
      // Try to get a move to trigger initialization
      const testMove = await getWasmAiMove(mockGameState, 'easy');
      
      console.log('WebAssembly initialization successful:', testMove);
      
      // Refresh engine status
      await checkEngineStatus();
      
      setTestResults({
        success: true,
        message: 'WebAssembly engine initialized successfully!',
        move: testMove
      });
      
    } catch (error) {
      console.error('WebAssembly initialization failed:', error);
      setError('WebAssembly initialization failed: ' + error.message);
      setTestResults({
        success: false,
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  };

  const validateBoardStructure = (board) => {
    if (!board || !Array.isArray(board)) {
      return { valid: false, error: 'Board is not an array' };
    }
    
    if (board.length !== 9) {
      return { valid: false, error: `Board has ${board.length} rows, expected 9. Found: ${board.length}` };
    }
    
    for (let row = 0; row < board.length; row++) {
      if (!Array.isArray(board[row])) {
        return { valid: false, error: `Row ${row} is not an array, found: ${typeof board[row]}` };
      }
      
      if (board[row].length !== 9) {
        return { valid: false, error: `Row ${row} has ${board[row].length} columns, expected 9` };
      }
      
      for (let col = 0; col < board[row].length; col++) {
        const cell = board[row][col];
        if (cell !== null && (typeof cell !== 'object' || !cell.type || !cell.player)) {
          return { 
            valid: false, 
            error: `Invalid cell at [${row}][${col}]: ${JSON.stringify(cell)}` 
          };
        }
      }
    }
    
    return { valid: true };
  };

  const runBasicTest = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      console.log('Running basic WebAssembly test...');
      console.log('Mock game state:', mockGameState);
      
      // Validate board structure first
      const boardValidation = validateBoardStructure(mockGameState.board);
      if (!boardValidation.valid) {
        throw new Error(`Board validation failed: ${boardValidation.error}`);
      }
      console.log('✅ Board structure validation passed');
      
      // Test basic move generation
      const move = await getWasmAiMove(mockGameState, 'easy');
      
      setTestResults({
        success: true,
        move,
        message: 'Basic test completed successfully!'
      });
      
    } catch (error) {
      console.error('Basic test error:', error);
      setError('Basic test failed: ' + error.message);
      setTestResults({
        success: false,
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  };

  const runPerformanceTest = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      console.log('Running performance test...');
      
      const results = await compareEnginePerformance(mockGameState, 'medium');
      
      setTestResults({
        success: true,
        performance: results,
        message: 'Performance test completed!'
      });
      
    } catch (error) {
      setError('Performance test failed: ' + error.message);
      setTestResults({
        success: false,
        error: error.message
      });
    } finally {
      setIsLoading(false);
    }
  };

  const runBenchmarkTest = async () => {
    setIsLoading(true);
    setError(null);
    
    try {
      console.log('Running benchmark test...');
      
      const results = await benchmarkEngines(mockGameState, 'hard');
      
      setTestResults({
        success: true,
        benchmark: results,
        message: 'Benchmark test completed!'
      });
      
    } catch (error) {
      setError('Benchmark test failed: ' + error.message);
      setError('Benchmark test failed: ' + error.message);
    } finally {
      setIsLoading(false);
    }
  };

  const getEngineStats = () => {
    try {
      return {
        stats: getEngineStats(),
        capabilities: getEngineCapabilities(),
        recommendations: getEngineRecommendations()
      };
    } catch (error) {
      return { error: error.message };
    }
  };

  const renderEngineStatus = () => {
    if (!engineStatus) return <div>Loading engine status...</div>;
    
    return (
      <div className="engine-status">
        <h3>Engine Status</h3>
        <div className="status-grid">
          <div className="status-item">
            <strong>WebAssembly Available:</strong> 
            <span className={engineStatus.wasmAvailable ? 'status-ok' : 'status-error'}>
              {engineStatus.wasmAvailable ? '✅ Yes' : '❌ No'}
            </span>
          </div>
          <div className="status-item">
            <strong>WebAssembly Enabled:</strong> 
            <span className={engineStatus.wasmEnabled ? 'status-ok' : 'status-warning'}>
              {engineStatus.wasmEnabled ? '✅ Yes' : '⚠️ No'}
            </span>
          </div>
          <div className="status-item">
            <strong>Current Engine:</strong> 
            <span className="status-info">{engineStatus.currentEngine}</span>
          </div>
        </div>
      </div>
    );
  };

  const renderTestResults = () => {
    if (!testResults) return null;
    
    return (
      <div className="test-results">
        <h3>Test Results</h3>
        {testResults.success ? (
          <div className="result-success">
            <p>✅ {testResults.message}</p>
            {testResults.move && (
              <div className="move-details">
                <strong>Generated Move:</strong>
                <pre>{JSON.stringify(testResults.move, null, 2)}</pre>
              </div>
            )}
            {testResults.performance && (
              <div className="performance-details">
                <strong>Performance Comparison:</strong>
                <pre>{JSON.stringify(testResults.performance, null, 2)}</pre>
              </div>
            )}
            {testResults.benchmark && (
              <div className="benchmark-details">
                <strong>Benchmark Results:</strong>
                <pre>{JSON.stringify(testResults.benchmark, null, 2)}</pre>
              </div>
            )}
          </div>
        ) : (
          <div className="result-error">
            <p>❌ Test failed: {testResults.error}</p>
          </div>
        )}
      </div>
    );
  };

  const renderEngineInfo = () => {
    try {
      const info = getEngineStats();
      if (info.error) return <div>Error loading engine info: {info.error}</div>;
      
      return (
        <div className="engine-info">
          <h3>Engine Information</h3>
          <div className="info-grid">
            <div className="info-section">
              <h4>Capabilities</h4>
              <ul>
                <li>Max Search Depth: {info.capabilities.maxSearchDepth}</li>
                <li>Bitboards: {info.capabilities.supportsBitboards ? '✅' : '❌'}</li>
                <li>Transposition Table: {info.capabilities.supportsTranspositionTable ? '✅' : '❌'}</li>
                <li>Killer Moves: {info.capabilities.supportsKillerMoves ? '✅' : '❌'}</li>
                <li>History Heuristic: {info.capabilities.supportsHistoryHeuristic ? '✅' : '❌'}</li>
              </ul>
            </div>
            <div className="info-section">
              <h4>Recommendations</h4>
              <ul>
                {Object.entries(info.recommendations).map(([difficulty, rec]) => (
                  <li key={difficulty}>
                    <strong>{difficulty}:</strong> {rec.recommended} - {rec.reason}
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>
      );
    } catch (error) {
      return <div>Error loading engine information: {error.message}</div>;
    }
  };

  return (
    <div className="wasm-demo">
      <h2>WebAssembly Engine Demo</h2>
      <p>This page demonstrates the WebAssembly integration with the Shogi AI engine.</p>
      
      {renderEngineStatus()}
      
      <div className="debug-info">
        <h3>Debug Information</h3>
        <div className="board-structure">
          <h4>Board Structure Validation</h4>
          <button 
            onClick={() => {
              const validation = validateBoardStructure(mockGameState.board);
              if (validation.valid) {
                alert('✅ Board structure is valid!');
              } else {
                alert(`❌ Board validation failed: ${validation.error}`);
              }
            }}
            className="btn btn-info"
          >
            Validate Board Structure
          </button>
          
          <h4>WebAssembly Engine Control</h4>
          <button 
            onClick={initializeWebAssembly}
            disabled={isLoading}
            className="btn btn-success"
            style={{ marginRight: '10px' }}
          >
            {isLoading ? 'Initializing...' : 'Initialize WebAssembly'}
          </button>
          
          <button 
            onClick={() => {
              resetEngine();
              checkEngineStatus();
              alert('WebAssembly engine reset. Click "Initialize WebAssembly" to reinitialize.');
            }}
            className="btn btn-warning"
          >
            Reset Engine
          </button>
        <div className="board-preview">
          <strong>Board Structure:</strong>
          <div>Rows: {mockGameState.board.length}</div>
          <div>Columns: {mockGameState.board[0] ? mockGameState.board[0].length : 'N/A'}</div>
          <button 
            onClick={() => {
              console.log('Full board structure:');
              mockGameState.board.forEach((row, index) => {
                console.log(`Row ${index}:`, row, `(length: ${row ? row.length : 'undefined'})`);
              });
            }}
            className="btn btn-secondary"
            style={{ marginTop: '10px', marginBottom: '10px' }}
          >
            Log Board Structure to Console
          </button>
          <strong>Board Preview (first 3 rows):</strong>
          <pre>{JSON.stringify(mockGameState.board.slice(0, 3), null, 2)}</pre>
        </div>
      </div>
      </div>
      
      <div className="demo-controls">
        <h3>Test Controls</h3>
        <div className="button-group">
          <button 
            onClick={runBasicTest} 
            disabled={isLoading}
            className="btn btn-primary"
          >
            {isLoading ? 'Running...' : 'Run Basic Test'}
          </button>
          
          <button 
            onClick={runPerformanceTest} 
            disabled={isLoading}
            className="btn btn-secondary"
          >
            {isLoading ? 'Running...' : 'Run Performance Test'}
          </button>
          
          <button 
            onClick={runBenchmarkTest} 
            disabled={isLoading}
            className="btn btn-success"
          >
            {isLoading ? 'Running...' : 'Run Benchmark Test'}
          </button>
        </div>
      </div>
      
      {error && (
        <div className="error-message">
          <strong>Error:</strong> {error}
        </div>
      )}
      
      {renderTestResults()}
      {renderEngineInfo()}
      
      <div className="demo-notes">
        <h3>Notes</h3>
        <ul>
          <li>The WebAssembly engine provides 5-10x performance improvement over JavaScript</li>
          <li>It supports advanced search algorithms and bitboard representation</li>
          <li>If WebAssembly fails, the system automatically falls back to JavaScript</li>
          <li>Check the browser console for detailed logs and performance metrics</li>
        </ul>
      </div>
    </div>
  );
};

export default WebAssemblyDemo;
