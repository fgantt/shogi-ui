import React, { useState, useEffect, useRef } from 'react';
import { useMultipleEngineEvents } from '../hooks/useTauriEvents';
import './UsiMonitor.css';

interface UsiMessage {
  id: string;
  timestamp: Date;
  direction: 'sent' | 'received';
  message: string;
  engineId: string;
}

interface TauriUsiMonitorProps {
  engineIds: string[];
  isVisible: boolean;
  onToggle: () => void;
  onSendCommand?: (engineId: string, command: string) => void;
}

export const TauriUsiMonitor: React.FC<TauriUsiMonitorProps> = ({
  engineIds,
  isVisible,
  onToggle,
  onSendCommand,
}) => {
  const historyRef = useRef<HTMLDivElement>(null);
  const [showDebugMessages, setShowDebugMessages] = useState(false);
  const [selectedEngine, setSelectedEngine] = useState<string>('all');
  const [communicationHistory, setCommunicationHistory] = useState<UsiMessage[]>([]);
  const [manualCommand, setManualCommand] = useState('');

  // Listen to all engine messages
  useMultipleEngineEvents(engineIds, {
    onUsiMessage: (engineId, message) => {
      const newMessage: UsiMessage = {
        id: `received-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'received',
        message,
        engineId,
      };
      setCommunicationHistory(prev => [...prev, newMessage]);
    },
    onUsiError: (engineId, error) => {
      const errorMessage: UsiMessage = {
        id: `error-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'received',
        message: `ERROR: ${error}`,
        engineId,
      };
      setCommunicationHistory(prev => [...prev, errorMessage]);
    },
  });

  // Add sent messages to history
  const addSentMessage = (engineId: string, command: string) => {
    const newMessage: UsiMessage = {
      id: `sent-${Date.now()}-${Math.random()}`,
      timestamp: new Date(),
      direction: 'sent',
      message: command,
      engineId,
    };
    setCommunicationHistory(prev => [...prev, newMessage]);
  };

  // Filter history based on debug toggle and selected engine
  const filteredHistory = communicationHistory.filter(entry => {
    const isDebug = entry.message.includes('DEBUG:') || entry.message.includes('info string');
    const engineMatch = selectedEngine === 'all' || entry.engineId === selectedEngine;
    return (!isDebug || showDebugMessages) && engineMatch;
  });

  // Get last sent and received commands
  const lastSentCommand = communicationHistory
    .filter(entry => entry.direction === 'sent')
    .slice(-1)[0]?.message || 'None';

  const lastReceivedCommand = communicationHistory
    .filter(entry => entry.direction === 'received' && (!entry.message.includes('DEBUG:') || showDebugMessages))
    .slice(-1)[0]?.message || 'None';

  // Auto-scroll to bottom when new messages are added
  useEffect(() => {
    if (historyRef.current) {
      historyRef.current.scrollTop = historyRef.current.scrollHeight;
    }
  }, [filteredHistory]);

  const handleSendCommand = () => {
    if (!manualCommand.trim() || !onSendCommand) return;
    
    const targetEngine = selectedEngine === 'all' ? engineIds[0] : selectedEngine;
    if (targetEngine) {
      onSendCommand(targetEngine, manualCommand.trim());
      addSentMessage(targetEngine, manualCommand.trim());
      setManualCommand('');
    }
  };

  const handleClearHistory = () => {
    setCommunicationHistory([]);
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
        <div className="session-selector">
          <label htmlFor="engine-select">Engine:</label>
          <select 
            id="engine-select" 
            value={selectedEngine} 
            onChange={(e) => setSelectedEngine(e.target.value)}
          >
            <option value="all">All Engines</option>
            {engineIds.map(engineId => (
              <option key={engineId} value={engineId}>{engineId.substring(0, 8)}...</option>
            ))}
          </select>
        </div>
        <button onClick={onToggle} className="close-button">
          ×
        </button>
      </div>
      
      <div className="usi-monitor-content">
        <div className="last-commands">
          <div className="last-command-item">
            <label>Last Sent:</label>
            <div className="command-text sent-command">
              {lastSentCommand}
            </div>
          </div>
          <div className="last-command-item">
            <label>Last Received:</label>
            <div className="command-text received-command">
              {lastReceivedCommand}
            </div>
          </div>
        </div>

        {onSendCommand && (
          <div className="manual-command">
            <input
              type="text"
              placeholder="Enter USI command..."
              value={manualCommand}
              onChange={(e) => setManualCommand(e.target.value)}
              onKeyPress={(e) => e.key === 'Enter' && handleSendCommand()}
            />
            <button onClick={handleSendCommand} disabled={!manualCommand.trim()}>
              Send
            </button>
          </div>
        )}
        
        <div className="communication-history">
          <div className="history-header">
            <h4>Communication History ({filteredHistory.length})</h4>
            <div className="history-controls">
              <label className="debug-toggle">
                <input
                  type="checkbox"
                  checked={showDebugMessages}
                  onChange={(e) => setShowDebugMessages(e.target.checked)}
                />
                <span className="toggle-label">Show Debug</span>
              </label>
              <button onClick={handleClearHistory} className="clear-button">
                Clear
              </button>
            </div>
          </div>
          <div className="history-container" ref={historyRef}>
            {filteredHistory.length === 0 ? (
              <div className="no-history">
                {communicationHistory.length === 0 
                  ? 'No communication yet' 
                  : 'No messages match the current filter'}
              </div>
            ) : (
              filteredHistory.map((entry) => (
                <div key={entry.id} className={`history-entry ${entry.direction}`}>
                  <span className="direction-prefix">
                    {entry.direction === 'sent' ? '→' : '←'}
                  </span>
                  <span className="timestamp">
                    {entry.timestamp.toLocaleTimeString()}
                  </span>
                  <span className="session-id">
                    [{entry.engineId.substring(0, 8)}]
                  </span>
                  <span className="command">
                    {entry.message}
                  </span>
                </div>
              ))
            )}
          </div>
        </div>
      </div>
    </div>
  );
};

