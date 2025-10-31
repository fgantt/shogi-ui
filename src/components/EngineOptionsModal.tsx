import React, { useState, useEffect, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { EngineConfig, EngineOption, CommandResponse } from '../types/engine';
import './EngineOptionsModal.css';

interface EngineOptionsModalProps {
  isOpen: boolean;
  engine: EngineConfig | null;
  onClose: () => void;
  onSave?: (options: OptionValue) => void;  // Optional: for temporary options mode
  tempOptions?: OptionValue | null;  // Optional: pre-populated temporary options
}

interface OptionValue {
  [optionName: string]: string;
}

export function EngineOptionsModal({ isOpen, engine, onClose, onSave, tempOptions }: EngineOptionsModalProps) {
  const [optionValues, setOptionValues] = useState<OptionValue>({});
  const [savedOptions, setSavedOptions] = useState<OptionValue>({});
  const [displayName, setDisplayName] = useState('');
  const [originalDisplayName, setOriginalDisplayName] = useState('');
  const [loading, setLoading] = useState(false);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [hasChanges, setHasChanges] = useState(false);

  // Load saved options when modal opens
  useEffect(() => {
    if (isOpen && engine) {
      loadSavedOptions();
    }
  }, [isOpen, engine]);

  // Check for changes whenever option values or display name change
  useEffect(() => {
    if (isOpen && engine) {
      const optionsChanged = Object.keys(optionValues).some(key => 
        optionValues[key] !== savedOptions[key]
      ) || Object.keys(savedOptions).some(key => 
        savedOptions[key] !== optionValues[key]
      );
      const displayNameChanged = displayName !== originalDisplayName;
      setHasChanges(optionsChanged || displayNameChanged);
    }
  }, [optionValues, savedOptions, displayName, originalDisplayName, isOpen, engine]);

  const loadSavedOptions = async () => {
    if (!engine) return;

    try {
      setLoading(true);
      setError(null);
      
      // Set display name
      setDisplayName(engine.display_name);
      setOriginalDisplayName(engine.display_name);
      
      // If tempOptions provided, use those (temporary mode for game)
      if (tempOptions) {
        setSavedOptions(tempOptions);
        setOptionValues({ ...tempOptions });
        setLoading(false);
        return;
      }
      
      // Try to load saved options first
      const response = await invoke<CommandResponse<OptionValue>>('get_engine_options', {
        engineId: engine.id
      });

      if (response.success && response.data) {
        setSavedOptions(response.data);
        setOptionValues({ ...response.data });
      } else {
        // No saved options, use defaults from engine metadata (with fallback injection)
        const defaults: OptionValue = {};
        const opts = effectiveOptions;
        opts.forEach(option => {
          if (option.default) {
            defaults[option.name] = option.default;
          }
        });
        setSavedOptions(defaults);
        setOptionValues({ ...defaults });
      }
    } catch (err) {
      console.error('Error loading saved options:', err);
      setError(`Failed to load saved options: ${err}`);
      
      // Fallback to defaults (with fallback injection)
      const defaults: OptionValue = {};
      const opts = effectiveOptions;
      opts.forEach(option => {
        if (option.default) {
          defaults[option.name] = option.default;
        }
      });
      setSavedOptions(defaults);
      setOptionValues({ ...defaults });
    } finally {
      setLoading(false);
    }
  };

  const handleOptionChange = (optionName: string, value: string) => {
    setOptionValues(prev => ({
      ...prev,
      [optionName]: value
    }));
  };

  const handleSaveOptions = async () => {
    if (!engine) return;

    try {
      setSaving(true);
      setError(null);

      // If onSave callback provided, use temporary mode (don't persist to storage)
      if (onSave) {
        onSave(optionValues);
        setSavedOptions({ ...optionValues });
        setHasChanges(false);
        onClose();
        setSaving(false);
        return;
      }

      // Otherwise, save permanently to storage
      // Save engine options
      const optionsResponse = await invoke<CommandResponse>('save_engine_options', {
        engineId: engine.id,
        options: optionValues
      });

      if (!optionsResponse.success) {
        setError(optionsResponse.message || 'Failed to save options');
        return;
      }

      // Save display name if it changed
      if (displayName !== originalDisplayName) {
        const displayNameResponse = await invoke<CommandResponse>('update_engine_display_name', {
          engineId: engine.id,
          newDisplayName: displayName
        });

        if (!displayNameResponse.success) {
          setError(displayNameResponse.message || 'Failed to save display name');
          return;
        }
      }

      setSavedOptions({ ...optionValues });
      setOriginalDisplayName(displayName);
      setHasChanges(false);
      // Don't close modal automatically, let user close manually
    } catch (err) {
      console.error('Error saving options:', err);
      setError(`Failed to save options: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleResetToDefaults = () => {
    const defaults: OptionValue = {};
    effectiveOptions.forEach(option => {
      if (option.default) {
        defaults[option.name] = option.default;
      }
    });
    setOptionValues(defaults);
  };

  const renderOptionInput = (option: EngineOption) => {
    const currentValue = optionValues[option.name] || option.default || '';

    switch (option.option_type) {
      case 'check':
        return (
          <div className="option-check">
            <label className="checkbox-label">
              <input
                type="checkbox"
                checked={currentValue === 'true'}
                onChange={(e) => handleOptionChange(option.name, e.target.checked ? 'true' : 'false')}
                className="option-checkbox"
              />
              <span className="checkbox-text">
                {currentValue === 'true' ? 'ON' : 'OFF'}
              </span>
            </label>
          </div>
        );

      case 'spin':
        const min = option.min ? parseInt(option.min) : 0;
        const max = option.max ? parseInt(option.max) : 1000;
        const current = parseInt(currentValue) || min;
        
        return (
          <div className="option-spin">
            <input
              type="number"
              min={min}
              max={max}
              value={current}
              onChange={(e) => handleOptionChange(option.name, e.target.value)}
              className="option-number"
            />
            <span className="option-range">
              (Range: {min} - {max})
            </span>
          </div>
        );

      case 'string':
        return (
          <input
            type="text"
            value={currentValue}
            onChange={(e) => handleOptionChange(option.name, e.target.value)}
            className="option-string"
            placeholder="Enter value..."
          />
        );

      case 'combo':
        return (
          <select
            value={currentValue}
            onChange={(e) => handleOptionChange(option.name, e.target.value)}
            className="option-select"
          >
            {option.var.map((variant, index) => (
              <option key={index} value={variant}>
                {variant}
              </option>
            ))}
          </select>
        );

      case 'button':
        return (
          <button
            onClick={() => handleOptionChange(option.name, '')}
            className="option-button"
          >
            {option.name}
          </button>
        );

      default:
        return (
          <input
            type="text"
            value={currentValue}
            onChange={(e) => handleOptionChange(option.name, e.target.value)}
            className="option-string"
            placeholder="Enter value..."
          />
        );
    }
  };

  // Build effective options list, injecting USI_Threads if engine didn't expose it in metadata
  const effectiveOptions = useMemo(() => {
    const opts = engine?.metadata?.options ? [...engine.metadata.options] : [] as EngineOption[];
    const hasThreads = opts.some(o => o.name === 'USI_Threads');
    if (!hasThreads) {
      const cores = (typeof navigator !== 'undefined' && (navigator as any).hardwareConcurrency) ? String((navigator as any).hardwareConcurrency) : '2';
      opts.push({
        name: 'USI_Threads',
        option_type: 'spin',
        default: cores,
        min: '1',
        max: '32',
        var: [],
      });
    }
    return opts;
  }, [engine]);

  if (!isOpen || !engine) {
    return null;
  }

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Configure Engine Options</h2>
          <button className="modal-close" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="modal-body">
          <div className="engine-info">
            <div className="display-name-field">
              <label htmlFor="display-name-input">
                <strong>Display Name:</strong>
              </label>
              <input
                id="display-name-input"
                type="text"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                className="display-name-input"
                placeholder="Enter display name..."
              />
            </div>
            {engine.metadata?.author && (
              <p><strong>Author:</strong> {engine.metadata.author}</p>
            )}
            <p><strong>Path:</strong> {engine.path}</p>
          </div>

          {error && (
            <div className="error-message">
              {error}
              <button onClick={() => setError(null)}>×</button>
            </div>
          )}

          {loading ? (
            <div className="loading-message">
              Loading options...
            </div>
          ) : effectiveOptions.length > 0 ? (
            <div className="options-section">
              <div className="options-header">
                <h4>Available Options:</h4>
                <button
                  onClick={handleResetToDefaults}
                  className="reset-button"
                  disabled={saving}
                >
                  Reset to Default Values
                </button>
              </div>

              <div className="options-list">
                {effectiveOptions.map((option, index) => (
                  <div key={index} className="option-item">
                    <div className="option-header">
                      <label className="option-name">{option.name}</label>
                      <span className={`option-type-badge option-type-${option.option_type}`}>
                        {option.option_type}
                      </span>
                    </div>
                    
                    <div className="option-controls">
                      {renderOptionInput(option)}
                    </div>

                    {option.default && (
                      <div className="option-info">
                        <span className="option-default">
                          Default: {option.default}
                        </span>
                      </div>
                    )}
                  </div>
                ))}
              </div>
            </div>
          ) : (
            <div className="no-options">
              <p>No configurable options available for this engine.</p>
            </div>
          )}
        </div>

        <div className="modal-footer">
          <button 
            onClick={onClose} 
            className="secondary-button"
            disabled={saving}
          >
            Cancel
          </button>
          
          {hasChanges && (
            <button 
              onClick={handleSaveOptions} 
              className="primary-button"
              disabled={saving}
            >
              {saving ? 'Saving...' : 'Save Options'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
