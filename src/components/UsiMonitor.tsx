import React, { useState, useEffect, useRef } from 'react';
import './UsiMonitor.css';

interface UsiMonitorProps {
  lastSentCommand: string;
  lastReceivedCommand: string;
  communicationHistory: Array<{
    id: string;
    timestamp: Date;
    direction: 'sent' | 'received';
    command: string;
  }>;
  isVisible: boolean;
  onToggle: () => void;
}

const UsiMonitor: React.FC<UsiMonitorProps> = ({
  lastSentCommand,
  lastReceivedCommand,
  communicationHistory,
  isVisible,
  onToggle
}) => {
  const historyRef = useRef<HTMLDivElement>(null);
  const [showDebugMessages, setShowDebugMessages] = useState(false);

  // Filter communication history based on debug toggle
  const filteredHistory = showDebugMessages 
    ? communicationHistory 
    : communicationHistory.filter(entry => !entry.command.includes('DEBUG:'));

  // Get the last non-debug received command for display
  const lastNonDebugReceivedCommand = showDebugMessages 
    ? lastReceivedCommand
    : communicationHistory
        .filter(entry => entry.direction === 'received' && !entry.command.includes('DEBUG:'))
        .slice(-1)[0]?.command || lastReceivedCommand;

  // Auto-scroll to bottom when new messages are added
  useEffect(() => {
    if (historyRef.current) {
      historyRef.current.scrollTop = historyRef.current.scrollHeight;
    }
  }, [filteredHistory]);

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
        <button onClick={onToggle} className="close-button">
          ×
        </button>
      </div>
      
      <div className="usi-monitor-content">
        <div className="last-commands">
          <div className="last-command-item">
            <label>Last Sent:</label>
            <div className="command-text sent-command">
              {lastSentCommand || 'None'}
            </div>
          </div>
          <div className="last-command-item">
            <label>Last Received:</label>
            <div className="command-text received-command">
              {lastNonDebugReceivedCommand || 'None'}
            </div>
          </div>
        </div>
        
        <div className="communication-history">
          <div className="history-header">
            <h4>Communication History</h4>
            <label className="debug-toggle">
              <input
                type="checkbox"
                checked={showDebugMessages}
                onChange={(e) => setShowDebugMessages(e.target.checked)}
              />
              <span className="toggle-label">Show Debug</span>
            </label>
          </div>
          <div className="history-container" ref={historyRef}>
            {filteredHistory.length === 0 ? (
              <div className="no-history">
                {communicationHistory.length === 0 
                  ? 'No communication yet' 
                  : 'No non-debug communication yet'}
              </div>
            ) : (
              filteredHistory.map((entry) => (
                <div key={entry.id} className={`history-entry ${entry.direction}`}>
                  <span className="direction-prefix">
                    {entry.direction === 'sent' ? '>' : '<'}
                  </span>
                  <span className="timestamp">
                    {entry.timestamp.toLocaleTimeString()}
                  </span>
                  <span className="command">
                    {entry.command}
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

export default UsiMonitor;
