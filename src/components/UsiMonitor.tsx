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
    sessionId: string;
  }>;
  sessions: string[];
  isVisible: boolean;
  onToggle: () => void;
}

const UsiMonitor: React.FC<UsiMonitorProps> = ({
  lastSentCommand,
  lastReceivedCommand,
  communicationHistory,
  sessions,
  isVisible,
  onToggle
}) => {
  const historyRef = useRef<HTMLDivElement>(null);
  const [showDebugMessages, setShowDebugMessages] = useState(false);
  const [selectedSession, setSelectedSession] = useState<string>('all');

  // Filter communication history based on debug toggle and selected session
  const filteredHistory = communicationHistory.filter(entry => {
    const isDebug = entry.command.includes('DEBUG:');
    const sessionMatch = selectedSession === 'all' || entry.sessionId === selectedSession;
    return (!isDebug || showDebugMessages) && sessionMatch;
  });

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
        <div className="session-selector">
          <label htmlFor="session-select">Session:</label>
          <select id="session-select" value={selectedSession} onChange={(e) => setSelectedSession(e.target.value)}>
            <option value="all">All</option>
            {sessions.map(session => (
              <option key={session} value={session}>{session}</option>
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
                  <span className="session-id">
                    [{entry.sessionId}]
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
