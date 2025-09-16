import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { WasmUsiHandler } from '../../pkg-bundler/shogi_engine.js';

interface Engine {
  name: string;
  path: string;
}

const EngineSettings = () => {
  const [engines, setEngines] = useState<Engine[]>([]);
  const [selectedEngine, setSelectedEngine] = useState<string>('');
  const [newEngineName, setNewEngineName] = useState('');
  const [newEnginePath, setNewEnginePath] = useState('');
  const [debugEnabled, setDebugEnabled] = useState<boolean>(false);

  useEffect(() => {
    const storedEngines = localStorage.getItem('shogi-engines');
    const storedSelectedEngine = localStorage.getItem('shogi-selected-engine');
    const storedDebugEnabled = localStorage.getItem('shogi-debug-enabled');

    if (storedEngines) {
      setEngines(JSON.parse(storedEngines));
    } else {
      const defaultEngines = [{ name: 'Built-in WASM USI Engine', path: 'wasm-usi' }];
      setEngines(defaultEngines);
      localStorage.setItem('shogi-engines', JSON.stringify(defaultEngines));
    }

    if (storedSelectedEngine) {
      setSelectedEngine(storedSelectedEngine);
    } else {
      setSelectedEngine('wasm-usi');
      localStorage.setItem('shogi-selected-engine', 'wasm-usi');
    }

    if (storedDebugEnabled) {
      setDebugEnabled(storedDebugEnabled === 'true');
    }
  }, []);

  const handleAddEngine = () => {
    if (newEngineName && newEnginePath) {
      const newEngine = { name: newEngineName, path: newEnginePath };
      const updatedEngines = [...engines, newEngine];
      setEngines(updatedEngines);
      localStorage.setItem('shogi-engines', JSON.stringify(updatedEngines));
      setNewEngineName('');
      setNewEnginePath('');
    }
  };

  const handleSelectEngine = (path: string) => {
    setSelectedEngine(path);
    localStorage.setItem('shogi-selected-engine', path);
  };

  const handleDebugToggle = (enabled: boolean) => {
    setDebugEnabled(enabled);
    localStorage.setItem('shogi-debug-enabled', enabled.toString());
    
    // Update the WASM debug setting
    try {
      WasmUsiHandler.set_debug_enabled(enabled);
    } catch (error) {
      console.error('Failed to set debug enabled:', error);
    }
  };

  return (
    <div className="settings-overlay">
      <div className="settings-panel">
        <h2>Engine Settings</h2>

        <section>
          <h3>Available Engines</h3>
          <div className="setting-group">
            {engines.map((engine) => (
              <label key={engine.path}>
                <input
                  type="radio"
                  name="engine"
                  value={engine.path}
                  checked={selectedEngine === engine.path}
                  onChange={() => handleSelectEngine(engine.path)}
                />
                {engine.name} ({engine.path})
              </label>
            ))}
          </div>
        </section>

        <section>
          <h3>Debug Settings</h3>
          <div className="setting-group">
            <label>
              <input
                type="checkbox"
                checked={debugEnabled}
                onChange={(e) => handleDebugToggle(e.target.checked)}
              />
              Enable Debug Logging
            </label>
            <p className="setting-description">
              Enable this to see detailed debug information in the browser console. 
              This can help with troubleshooting but will make the console very verbose.
            </p>
          </div>
        </section>

        <section>
          <h3>Add New Engine</h3>
          <div className="setting-group">
            <input
              type="text"
              placeholder="Engine Name"
              value={newEngineName}
              onChange={(e) => setNewEngineName(e.target.value)}
            />
            <input
              type="text"
              placeholder="Engine Path"
              value={newEnginePath}
              onChange={(e) => setNewEnginePath(e.target.value)}
            />
            <button onClick={handleAddEngine}>Add Engine</button>
          </div>
        </section>

        <Link to="/game" className="button-link">
          Back to Game
        </Link>
      </div>
    </div>
  );
};

export default EngineSettings;
