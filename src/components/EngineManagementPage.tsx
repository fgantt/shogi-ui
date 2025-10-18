import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import type { EngineConfig, CommandResponse, EngineMetadata, EngineHealthResult } from '../types/engine';
import { EngineOptionsModal } from './EngineOptionsModal';
import './EngineManagementPage.css';

export function EngineManagementPage() {
  const [engines, setEngines] = useState<EngineConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [addingEngine, setAddingEngine] = useState(false);
  const [validating, setValidating] = useState(false);
  const [healthCheckResults, setHealthCheckResults] = useState<Map<string, EngineHealthResult>>(new Map());

  // Form state for adding new engine
  const [newEngineName, setNewEngineName] = useState('');
  const [newEnginePath, setNewEnginePath] = useState('');
  const [validationResult, setValidationResult] = useState<EngineMetadata | null>(null);

  // Engine options modal state
  const [optionsModalOpen, setOptionsModalOpen] = useState(false);
  const [selectedEngineForOptions, setSelectedEngineForOptions] = useState<EngineConfig | null>(null);

  // Load engines on mount
  useEffect(() => {
    loadEngines();
    registerBuiltinEngine();
  }, []);

  const loadEngines = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await invoke<CommandResponse<EngineConfig[]>>('get_engines');
      
      if (response.success && response.data) {
        setEngines(response.data);
      } else {
        setError(response.message || 'Failed to load engines');
      }
    } catch (err) {
      setError(`Error loading engines: ${err}`);
      console.error('Error loading engines:', err);
    } finally {
      setLoading(false);
    }
  };

  const registerBuiltinEngine = async () => {
    try {
      const response = await invoke<CommandResponse>('register_builtin_engine');
      if (response.success) {
        console.log('Built-in engine registered');
        // Reload engines to show the built-in engine
        setTimeout(loadEngines, 500);
      }
    } catch (err) {
      console.warn('Failed to register built-in engine:', err);
    }
  };

  const handleBrowseEngine = async () => {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        // No filters - allow all files to be selected
      });

      if (selected && typeof selected === 'string') {
        setNewEnginePath(selected);
        // Auto-validate when path is selected
        validateEnginePath(selected);
      }
    } catch (err) {
      console.error('Error browsing for engine:', err);
      setError(`Failed to browse for engine: ${err}`);
    }
  };

  const validateEnginePath = async (path: string) => {
    if (!path) return;

    try {
      setValidating(true);
      setValidationResult(null);
      setError(null);

      const response = await invoke<CommandResponse<EngineMetadata>>('validate_engine_path', {
        path,
      });

      if (response.success && response.data) {
        setValidationResult(response.data);
        // Auto-fill name if empty
        if (!newEngineName && response.data.name) {
          setNewEngineName(response.data.name);
        }
      } else {
        setError(response.message || 'Engine validation failed');
        setValidationResult(null);
      }
    } catch (err) {
      setError(`Validation error: ${err}`);
      setValidationResult(null);
    } finally {
      setValidating(false);
    }
  };

  const handleAddEngine = async () => {
    if (!newEngineName.trim() || !newEnginePath.trim()) {
      setError('Please provide both name and path');
      return;
    }

    try {
      setAddingEngine(true);
      setError(null);

      const response = await invoke<CommandResponse<EngineConfig>>('add_engine', {
        name: newEngineName.trim(),
        path: newEnginePath.trim(),
      });

      if (response.success) {
        // Clear form
        setNewEngineName('');
        setNewEnginePath('');
        setValidationResult(null);
        // Reload engines
        await loadEngines();
      } else {
        setError(response.message || 'Failed to add engine');
      }
    } catch (err) {
      setError(`Error adding engine: ${err}`);
    } finally {
      setAddingEngine(false);
    }
  };

  const handleRemoveEngine = async (engineId: string, engineName: string) => {
    if (!confirm(`Are you sure you want to remove "${engineName}"?`)) {
      return;
    }

    try {
      setError(null);
      const response = await invoke<CommandResponse>('remove_engine', {
        engineId,
      });

      if (response.success) {
        await loadEngines();
      } else {
        setError(response.message || 'Failed to remove engine');
      }
    } catch (err) {
      setError(`Error removing engine: ${err}`);
    }
  };

  const handleOpenOptions = (engine: EngineConfig) => {
    setSelectedEngineForOptions(engine);
    setOptionsModalOpen(true);
  };

  const handleCloseOptions = () => {
    setOptionsModalOpen(false);
    setSelectedEngineForOptions(null);
  };

  const handleCloneEngine = async (engine: EngineConfig) => {
    const newDisplayName = `${engine.display_name} (copy)`;
    
    try {
      setError(null);
      const response = await invoke<CommandResponse<{ new_engine_id: string }>>('clone_engine', {
        engineId: engine.id,
        newDisplayName: newDisplayName,
      });

      if (response.success) {
        await loadEngines(); // Reload the engines list
        console.log('Engine cloned successfully:', response.data?.new_engine_id);
      } else {
        setError(response.message || 'Failed to clone engine');
      }
    } catch (err) {
      setError(`Error cloning engine: ${err}`);
      console.error('Error cloning engine:', err);
    }
  };

  const handleHealthCheck = async () => {
    try {
      setError(null);
      const response = await invoke<CommandResponse<{ results: EngineHealthResult[] }>>('health_check_engines');

      if (response.success && response.data) {
        const resultsMap = new Map<string, EngineHealthResult>();
        response.data.results.forEach(result => {
          resultsMap.set(result.id, result);
        });
        setHealthCheckResults(resultsMap);
      } else {
        setError(response.message || 'Health check failed');
      }
    } catch (err) {
      setError(`Error during health check: ${err}`);
    }
  };

  const handleToggleFavorite = async (engineId: string) => {
    try {
      setError(null);
      const response = await invoke<CommandResponse>('set_favorite_engine', {
        engineId,
      });

      if (response.success) {
        await loadEngines(); // Reload to show updated favorite status
      } else {
        setError(response.message || 'Failed to set favorite engine');
      }
    } catch (err) {
      setError(`Error setting favorite engine: ${err}`);
      console.error('Error setting favorite engine:', err);
    }
  };

  const getEngineStatusBadge = (engine: EngineConfig) => {
    const healthResult = healthCheckResults.get(engine.id);
    
    if (healthResult) {
      switch (healthResult.status) {
        case 'healthy':
          return <span className="status-badge status-healthy">Healthy</span>;
        case 'unhealthy':
          return <span className="status-badge status-unhealthy">Unhealthy</span>;
        case 'disabled':
          return <span className="status-badge status-disabled">Disabled</span>;
      }
    }

    return engine.enabled 
      ? <span className="status-badge status-unknown">Unknown</span>
      : <span className="status-badge status-disabled">Disabled</span>;
  };

  if (loading) {
    return (
      <div className="engine-management-page">
        <div className="loading">Loading engines...</div>
      </div>
    );
  }

  return (
    <div className="engine-management-page">
      <h1>Engine Management</h1>

      {error && (
        <div className="error-message">
          {error}
          <button onClick={() => setError(null)}>✕</button>
        </div>
      )}

      {/* Add Engine Section */}
      <section className="add-engine-section">
        <h2>Add New Engine</h2>
        <div className="add-engine-form">
          <div className="form-row">
            <label>
              Engine Name:
              <input
                type="text"
                value={newEngineName}
                onChange={(e) => setNewEngineName(e.target.value)}
                placeholder="e.g., Apery, YaneuraOu"
                disabled={addingEngine || validating}
              />
            </label>
          </div>

          <div className="form-row">
            <label>
              Engine Path:
              <div className="path-input-group">
                <input
                  type="text"
                  value={newEnginePath}
                  onChange={(e) => setNewEnginePath(e.target.value)}
                  placeholder="Path to engine executable"
                  disabled={addingEngine || validating}
                />
                <button onClick={handleBrowseEngine} disabled={addingEngine || validating}>
                  Browse...
                </button>
              </div>
            </label>
          </div>

          {validating && (
            <div className="validation-status">
              <span className="spinner">⟳</span> Validating engine...
            </div>
          )}

          {validationResult && (
            <div className="validation-result">
              <h3>✓ Engine Validated Successfully</h3>
              <p><strong>Name:</strong> {validationResult.name}</p>
              {validationResult.author && (
                <p><strong>Author:</strong> {validationResult.author}</p>
              )}
              <p><strong>Options:</strong> {validationResult.options.length} available</p>
            </div>
          )}

          <div className="form-actions">
            <button
              onClick={handleAddEngine}
              disabled={addingEngine || validating || !newEngineName || !newEnginePath}
              className="primary-button"
            >
              {addingEngine ? 'Adding...' : 'Add Engine'}
            </button>
            <button
              onClick={() => {
                setNewEngineName('');
                setNewEnginePath('');
                setValidationResult(null);
                setError(null);
              }}
              disabled={addingEngine || validating}
            >
              Clear
            </button>
          </div>
        </div>
      </section>

      {/* Engine List Section */}
      <section className="engine-list-section">
        <div className="section-header">
          <h2>Configured Engines ({engines.length})</h2>
          <button onClick={handleHealthCheck} className="secondary-button">
            Run Health Check
          </button>
        </div>

        {engines.length === 0 ? (
          <div className="empty-state">
            <p>No engines configured yet.</p>
            <p>Add an engine using the form above or the built-in engine will be registered automatically.</p>
          </div>
        ) : (
          <div className="engine-list">
            {engines.map((engine) => (
              <div key={engine.id} className="engine-card">
                <div className="engine-header">
                  <div className="engine-title">
                    <h3>
                      <button
                        onClick={() => handleToggleFavorite(engine.id)}
                        className="favorite-button"
                        title={engine.is_favorite ? "Unmark as favorite" : "Mark as favorite"}
                      >
                        {engine.is_favorite ? '⭐' : '☆'}
                      </button>
                      {engine.display_name}
                      {engine.is_builtin && <span className="builtin-badge">Built-in</span>}
                    </h3>
                    {getEngineStatusBadge(engine)}
                  </div>
                  <div className="engine-actions">
                    <button
                      onClick={() => handleOpenOptions(engine)}
                      className="options-button"
                      title="Configure engine options"
                      disabled={!engine.metadata || engine.metadata.options.length === 0}
                    >
                      Options
                    </button>
                    <button
                      onClick={() => handleCloneEngine(engine)}
                      className="clone-button"
                      title="Clone this engine"
                    >
                      Clone
                    </button>
                    {!engine.is_builtin && (
                      <button
                        onClick={() => handleRemoveEngine(engine.id, engine.display_name)}
                        className="remove-button"
                        title="Remove engine"
                      >
                        Remove
                      </button>
                    )}
                  </div>
                </div>

                <div className="engine-details">
                  <p className="engine-path">
                    <strong>Path:</strong> {engine.path}
                  </p>

                  {engine.metadata && (
                    <>
                      {engine.metadata.author && (
                        <p>
                          <strong>Author:</strong> {engine.metadata.author}
                        </p>
                      )}
                      <p>
                        <strong>Options:</strong> {engine.metadata.options.length} available
                      </p>
                    </>
                  )}

                  {engine.last_used && (
                    <p className="last-used">
                      <strong>Last used:</strong> {new Date(engine.last_used).toLocaleString()}
                    </p>
                  )}
                </div>

                {healthCheckResults.get(engine.id)?.error && (
                  <div className="engine-error">
                    <strong>Error:</strong> {healthCheckResults.get(engine.id)?.error}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </section>

      {/* Engine Options Modal */}
      <EngineOptionsModal
        isOpen={optionsModalOpen}
        engine={selectedEngineForOptions}
        onClose={handleCloseOptions}
      />
    </div>
  );
}

