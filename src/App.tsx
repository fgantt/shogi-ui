import { Routes, Route } from 'react-router-dom';
import HomePage from './components/HomePage';
import GamePage from './components/GamePage';
import PracticePage from './components/PracticePage';
import PracticeExerciseDetail from './components/PracticeExerciseDetail';
import HelpPage from './components/HelpPage';
import AboutPage from './components/AboutPage';
import EngineSettings from './components/EngineSettings';

import { ShogiController } from './usi/controller';
import { WasmEngineAdapter } from './usi/engine';
import { ShogiControllerProvider } from './context/ShogiControllerContext';

import './App.css';
import './styles/shogi.css';
import './styles/settings.css';
import { useEffect, useState } from 'react';

// --- Singleton ShogiController ---
const wasmEngineAdapter = new WasmEngineAdapter();
const shogiController = new ShogiController(wasmEngineAdapter);
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
  }>>([]);

  useEffect(() => {
    // USI communication event handlers
    const onUsiCommandSent = ({ command }: { command: string }) => {
      setLastSentCommand(command);
      const newEntry = {
        id: `sent-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'sent' as const,
        command
      };
      setCommunicationHistory(prev => [...prev, newEntry]);
    };

    const onUsiCommandReceived = ({ command }: { command: string }) => {
      setLastReceivedCommand(command);
      const newEntry = {
        id: `received-${Date.now()}-${Math.random()}`,
        timestamp: new Date(),
        direction: 'received' as const,
        command
      };
      setCommunicationHistory(prev => [...prev, newEntry]);
    };

    const engine = (shogiController as any).engine;
    if (engine) {
      engine.on('usiCommandSent', onUsiCommandSent);
      engine.on('usiCommandReceived', onUsiCommandReceived);
    }

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
      if (engine) {
        engine.off('usiCommandSent', onUsiCommandSent);
        engine.off('usiCommandReceived', onUsiCommandReceived);
      }
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
          <Route path="/settings/engine" element={<EngineSettings />} />
        </Routes>
      </ShogiControllerProvider>
    </div>
  );
}

export default App;
