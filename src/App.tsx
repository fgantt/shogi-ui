import { Routes, Route } from 'react-router-dom';
import HomePage from './components/HomePage';
import GamePage from './components/GamePage';
import PracticePage from './components/PracticePage';
import PracticeExerciseDetail from './components/PracticeExerciseDetail';
import HelpPage from './components/HelpPage';
import AboutPage from './components/AboutPage';
import { EngineManagementPage } from './components/EngineManagementPage';
import { TauriGameDemo } from './components/TauriGameDemo';


import { ShogiController } from './usi/controller';
import { ShogiControllerProvider } from './context/ShogiControllerContext';

import './styles/themes.css';
import './App.css';
import './styles/shogi.css';
import './styles/settings.css';
import { useEffect, useState } from 'react';

// --- Singleton ShogiController ---
const shogiController = new ShogiController();
// ---------------------------------

function App() {
  const [isControllerInitialized, setIsControllerInitialized] = useState(shogiController.isInitialized());

  // USI Monitor state
  const [isUsiMonitorVisible, setIsUsiMonitorVisible] = useState(false);
  const [lastSentCommand, setLastSentCommand] = useState<string>('');
  const [lastReceivedCommand, setLastReceivedCommand] = useState<string>('');
  const [communicationHistory, setCommunicationHistory] = useState<Array<{
    id: string;
    timestamp: Date;
    direction: 'sent' | 'received';
    command: string;
    sessionId: string;
  }>>([]);
  const [sessions, setSessions] = useState<string[]>([]);

  useEffect(() => {
    // USI communication event handlers
    const onUsiCommandSent = ({ command, sessionId }: { command: string, sessionId: string }) => {
      setLastSentCommand(command);
      const newEntry = {
        id: `sent-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'sent' as const,
        command,
        sessionId,
      };
      setCommunicationHistory(prev => [...prev, newEntry]);
    };

    const onUsiCommandReceived = ({ command, sessionId }: { command: string, sessionId: string }) => {
      setLastReceivedCommand(command);
      const newEntry = {
        id: `received-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'received' as const,
        command,
        sessionId,
      };
      setCommunicationHistory(prev => [...prev, newEntry]);
    };

    const onSessionCreated = ({ sessionId, engine }: { sessionId: string, engine: any }) => {
      setSessions(prev => [...prev, sessionId]);
      engine.on('usiCommandSent', onUsiCommandSent);
      engine.on('usiCommandReceived', onUsiCommandReceived);
    };

    shogiController.on('sessionCreated', onSessionCreated);

    if (!shogiController.isInitialized()) {
      shogiController.initialize().then(() => {
        setIsControllerInitialized(true);
      });
    }

    // Initialize default wallpaper for all routes
    const initializeDefaultWallpaper = async () => {
      const modules = import.meta.glob('/public/wallpapers/*.{jpg,svg}');
      const paths = Object.keys(modules).map(path => path.replace('/public', ''));
      if (paths.length > 0) {
        // Set photo1.jpg as the default wallpaper
        const defaultWallpaper = '/wallpapers/photo1.jpg';
        const initialWallpaper = paths.includes(defaultWallpaper) ? defaultWallpaper : paths[0];
        document.body.style.backgroundImage = `url('${initialWallpaper}')`;
        document.body.style.backgroundSize = 'cover';
        document.body.style.backgroundRepeat = 'no-repeat';
        document.body.style.backgroundPosition = 'center center';
        document.body.style.backgroundAttachment = 'fixed';
      }
    };
    
    initializeDefaultWallpaper();

    return () => {
      shogiController.off('sessionCreated', onSessionCreated);
      // Engine cleanup is now handled by Tauri engines in GamePage
    };
  }, []); // Empty dependency array to run only once

  if (!isControllerInitialized) {
    return <div className="loading-screen">Initializing Engine...</div>;
  }

  return (
    <div className="app">
      <ShogiControllerProvider controller={shogiController}>
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route 
            path="/game" 
            element={<GamePage 
              isUsiMonitorVisible={isUsiMonitorVisible}
              lastSentCommand={lastSentCommand}
              lastReceivedCommand={lastReceivedCommand}
              communicationHistory={communicationHistory}
              sessions={sessions}
              onToggleUsiMonitor={() => setIsUsiMonitorVisible(!isUsiMonitorVisible)}
              clearUsiHistory={() => {
                setCommunicationHistory([]);
                setLastSentCommand('');
                setLastReceivedCommand('');
              }}
            />} 
          />
          <Route path="/practice" element={<PracticePage />} />
          <Route path="/practice/:exerciseId" element={<PracticeExerciseDetail />} />
          <Route path="/help" element={<HelpPage />} />
          <Route path="/about" element={<AboutPage />} />
          <Route path="/engines" element={<EngineManagementPage />} />
          <Route path="/demo" element={<TauriGameDemo />} />

        </Routes>
      </ShogiControllerProvider>
    </div>
  );
}

export default App;
