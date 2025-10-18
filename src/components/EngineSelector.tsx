import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { EngineConfig, CommandResponse } from '../types/engine';
import './EngineSelector.css';

interface EngineSelectorProps {
  selectedEngineId: string | null;
  onEngineSelect: (engineId: string | null) => void;
  label?: string;
  includeNone?: boolean;
  autoSelect?: boolean;
}

export function EngineSelector({
  selectedEngineId,
  onEngineSelect,
  label = 'Select Engine',
  includeNone = false,
  autoSelect = true,
}: EngineSelectorProps) {
  const [engines, setEngines] = useState<EngineConfig[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    loadEngines();
  }, []);

  const loadEngines = async () => {
    try {
      setLoading(true);
      setError(null);
      const response = await invoke<CommandResponse<EngineConfig[]>>('get_engines');
      
      if (response.success && response.data) {
        const enabledEngines = response.data.filter(e => e.enabled);
        setEngines(enabledEngines);
        
        // Auto-select favorite engine if nothing selected (only if autoSelect is enabled)
        if (autoSelect && !selectedEngineId && enabledEngines.length > 0) {
          // First, try to find the favorite engine
          const favoriteEngine = enabledEngines.find(e => e.is_favorite);
          if (favoriteEngine) {
            onEngineSelect(favoriteEngine.id);
          } else {
            // Fallback to built-in engine if no favorite is set
            const builtinEngine = enabledEngines.find(e => e.is_builtin);
            if (builtinEngine) {
              onEngineSelect(builtinEngine.id);
            } else if (enabledEngines.length > 0) {
              onEngineSelect(enabledEngines[0].id);
            }
          }
        }
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

  if (loading) {
    return (
      <div className="engine-selector">
        <label>{label}</label>
        <div className="engine-selector-loading">Loading engines...</div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="engine-selector">
        <label>{label}</label>
        <div className="engine-selector-error">
          {error}
          <button onClick={loadEngines} className="retry-button">Retry</button>
        </div>
      </div>
    );
  }

  if (engines.length === 0) {
    return (
      <div className="engine-selector">
        <label>{label}</label>
        <div className="engine-selector-empty">
          No engines available. <a href="/engines">Add an engine</a>
        </div>
      </div>
    );
  }

  return (
    <div className="engine-selector">
      <label htmlFor="engine-select">{label}</label>
      <select
        id="engine-select"
        value={selectedEngineId || ''}
        onChange={(e) => onEngineSelect(e.target.value || null)}
        className="engine-select"
      >
        {!autoSelect && !selectedEngineId && <option value="">-- Select an engine --</option>}
        {includeNone && <option value="">None (Human only)</option>}
        {engines.map((engine) => (
          <option key={engine.id} value={engine.id}>
            {engine.is_favorite ? '⭐ ' : ''}{engine.display_name} {engine.is_builtin ? '(Built-in)' : ''}
          </option>
        ))}
      </select>
      <button onClick={loadEngines} className="refresh-button" title="Refresh engine list">
        ⟳
      </button>
    </div>
  );
}

