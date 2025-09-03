import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';

interface Engine {
  name: string;
  path: string;
}

const EngineSettings = () => {
  const [engines, setEngines] = useState<Engine[]>([]);
  const [selectedEngine, setSelectedEngine] = useState<string>('');
  const [newEngineName, setNewEngineName] = useState('');
  const [newEnginePath, setNewEnginePath] = useState('');

  useEffect(() => {
    const storedEngines = localStorage.getItem('shogi-engines');
    const storedSelectedEngine = localStorage.getItem('shogi-selected-engine');

    if (storedEngines) {
      setEngines(JSON.parse(storedEngines));
    } else {
      const defaultEngines = [{ name: 'Built-in WASM', path: '../ai/ai.worker.ts' }];
      setEngines(defaultEngines);
      localStorage.setItem('shogi-engines', JSON.stringify(defaultEngines));
    }

    if (storedSelectedEngine) {
      setSelectedEngine(storedSelectedEngine);
    } else {
      setSelectedEngine('../ai/ai.worker.ts');
      localStorage.setItem('shogi-selected-engine', '../ai/ai.worker.ts');
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
