import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { EngineConfig, EngineOption, CommandResponse } from '../types/engine';
import './EngineOptionsModal.css';

interface EngineOptionsModalProps {
  isOpen: boolean;
  engine: EngineConfig | null;
  onClose: () => void;
}

interface OptionValue {
  [optionName: string]: string;
}

export function EngineOptionsModal({ isOpen, engine, onClose }: EngineOptionsModalProps) {
  const [optionValues, setOptionValues] = useState<OptionValue>({});
  const [savedOptions, setSavedOptions] = useState<OptionValue>({});
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

  // Check for changes whenever option values change
  useEffect(() => {
    if (isOpen && engine) {
      const changed = Object.keys(optionValues).some(key => 
        optionValues[key] !== savedOptions[key]
      ) || Object.keys(savedOptions).some(key => 
        savedOptions[key] !== optionValues[key]
      );
      setHasChanges(changed);
    }
  }, [optionValues, savedOptions, isOpen, engine]);

  const loadSavedOptions = async () => {
    if (!engine) return;

    try {
      setLoading(true);
      setError(null);
      
      // Try to load saved options first
      const response = await invoke<CommandResponse<OptionValue>>('get_engine_options', {
        engineId: engine.id
      });

      if (response.success && response.data) {
        setSavedOptions(response.data);
        setOptionValues({ ...response.data });
      } else {
        // No saved options, use defaults from engine metadata
        const defaults: OptionValue = {};
        if (engine.metadata?.options) {
          engine.metadata.options.forEach(option => {
            if (option.default) {
              defaults[option.name] = option.default;
            }
          });
        }
        setSavedOptions(defaults);
        setOptionValues({ ...defaults });
      }
    } catch (err) {
      console.error('Error loading saved options:', err);
      setError(`Failed to load saved options: ${err}`);
      
      // Fallback to defaults
      const defaults: OptionValue = {};
      if (engine.metadata?.options) {
        engine.metadata.options.forEach(option => {
          if (option.default) {
            defaults[option.name] = option.default;
          }
        });
      }
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

      const response = await invoke<CommandResponse>('save_engine_options', {
        engineId: engine.id,
        options: optionValues
      });

      if (response.success) {
        setSavedOptions({ ...optionValues });
        setHasChanges(false);
        // Don't close modal automatically, let user close manually
      } else {
        setError(response.message || 'Failed to save options');
      }
    } catch (err) {
      console.error('Error saving options:', err);
      setError(`Failed to save options: ${err}`);
    } finally {
      setSaving(false);
    }
  };

  const handleResetToDefaults = () => {
    if (!engine?.metadata?.options) return;

    const defaults: OptionValue = {};
    engine.metadata.options.forEach(option => {
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
            <h3>{engine.name}</h3>
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
          ) : engine.metadata?.options && engine.metadata.options.length > 0 ? (
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
                {engine.metadata.options.map((option, index) => (
                  <div key={index} className="option-item">
                    <div className="option-header">
                      <label className="option-name">{option.name}</label>
                      <span className="option-type">({option.option_type})</span>
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

                    {option.min && option.max && (
                      <div className="option-info">
                        <span className="option-range">
                          Range: {option.min} - {option.max}
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
