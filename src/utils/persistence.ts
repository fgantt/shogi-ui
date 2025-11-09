type PlayerType = 'human' | 'ai';

const isBrowserEnvironment = (): boolean => {
  return typeof window !== 'undefined' && typeof window.localStorage !== 'undefined';
};

const safeRead = (key: string): string | null => {
  if (!isBrowserEnvironment()) {
    return null;
  }

  try {
    return window.localStorage.getItem(key);
  } catch (error) {
    console.error(`Failed to read localStorage key "${key}":`, error);
    return null;
  }
};

const safeWrite = (key: string, value: string): void => {
  if (!isBrowserEnvironment()) {
    return;
  }

  try {
    window.localStorage.setItem(key, value);
  } catch (error) {
    console.error(`Failed to write localStorage key "${key}":`, error);
  }
};

const safeRemove = (key: string): void => {
  if (!isBrowserEnvironment()) {
    return;
  }

  try {
    window.localStorage.removeItem(key);
  } catch (error) {
    console.error(`Failed to remove localStorage key "${key}":`, error);
  }
};

export const NEW_GAME_SETTINGS_STORAGE_KEY = 'shogi-new-game-settings';
export const USI_MONITOR_STATE_STORAGE_KEY = 'shogi-usi-monitor-state';
export const WINDOW_SIZE_STORAGE_KEY = 'shogi-window-size';

export interface StoredNewGameSettings {
  player1Type: PlayerType;
  player2Type: PlayerType;
  player1EngineId: string | null;
  player2EngineId: string | null;
  minutesPerSide: number;
  byoyomiInSeconds: number;
  initialSfen: string;
  selectedCannedPosition: string;
  isInitialPositionCollapsed: boolean;
  player1TempOptions?: Record<string, string> | null;
  player2TempOptions?: Record<string, string> | null;
}

export const loadNewGameSettings = (): StoredNewGameSettings | null => {
  const raw = safeRead(NEW_GAME_SETTINGS_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') {
      return null;
    }

    return {
      player1Type: parsed.player1Type === 'ai' ? 'ai' : 'human',
      player2Type: parsed.player2Type === 'human' ? 'human' : 'ai',
      player1EngineId: typeof parsed.player1EngineId === 'string' ? parsed.player1EngineId : null,
      player2EngineId: typeof parsed.player2EngineId === 'string' ? parsed.player2EngineId : null,
      minutesPerSide: Number.isFinite(parsed.minutesPerSide) ? parsed.minutesPerSide : 30,
      byoyomiInSeconds: Number.isFinite(parsed.byoyomiInSeconds) ? parsed.byoyomiInSeconds : 10,
      initialSfen: typeof parsed.initialSfen === 'string' ? parsed.initialSfen : '',
      selectedCannedPosition: typeof parsed.selectedCannedPosition === 'string' ? parsed.selectedCannedPosition : 'Standard',
      isInitialPositionCollapsed: Boolean(parsed.isInitialPositionCollapsed),
      player1TempOptions: parsed.player1TempOptions && typeof parsed.player1TempOptions === 'object'
        ? parsed.player1TempOptions as Record<string, string>
        : null,
      player2TempOptions: parsed.player2TempOptions && typeof parsed.player2TempOptions === 'object'
        ? parsed.player2TempOptions as Record<string, string>
        : null,
    };
  } catch (error) {
    console.error('Failed to parse stored new game settings:', error);
    safeRemove(NEW_GAME_SETTINGS_STORAGE_KEY);
    return null;
  }
};

export const saveNewGameSettings = (settings: StoredNewGameSettings): void => {
  safeWrite(NEW_GAME_SETTINGS_STORAGE_KEY, JSON.stringify(settings));
};

export interface StoredUsiMonitorState {
  isVisible: boolean;
  activeTab: 'engine' | 'search';
}

const DEFAULT_USI_MONITOR_STATE: StoredUsiMonitorState = {
  isVisible: false,
  activeTab: 'engine',
};

export const loadUsiMonitorState = (): StoredUsiMonitorState => {
  const raw = safeRead(USI_MONITOR_STATE_STORAGE_KEY);
  if (!raw) {
    return { ...DEFAULT_USI_MONITOR_STATE };
  }

  try {
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== 'object') {
      return { ...DEFAULT_USI_MONITOR_STATE };
    }

    return {
      isVisible: Boolean(parsed.isVisible),
      activeTab: parsed.activeTab === 'search' ? 'search' : 'engine',
    };
  } catch (error) {
    console.error('Failed to parse stored USI monitor state:', error);
    safeRemove(USI_MONITOR_STATE_STORAGE_KEY);
    return { ...DEFAULT_USI_MONITOR_STATE };
  }
};

export const saveUsiMonitorState = (update: Partial<StoredUsiMonitorState>): void => {
  const current = loadUsiMonitorState();
  const nextState = {
    ...current,
    ...update,
  };
  safeWrite(USI_MONITOR_STATE_STORAGE_KEY, JSON.stringify(nextState));
};

export interface StoredWindowSize {
  width: number;
  height: number;
}

export const loadWindowSize = (): StoredWindowSize | null => {
  const raw = safeRead(WINDOW_SIZE_STORAGE_KEY);
  if (!raw) {
    return null;
  }

  try {
    const parsed = JSON.parse(raw);
    if (
      !parsed ||
      typeof parsed !== 'object' ||
      !Number.isFinite(parsed.width) ||
      !Number.isFinite(parsed.height)
    ) {
      return null;
    }

    return {
      width: parsed.width,
      height: parsed.height,
    };
  } catch (error) {
    console.error('Failed to parse stored window size:', error);
    safeRemove(WINDOW_SIZE_STORAGE_KEY);
    return null;
  }
};

export const saveWindowSize = (size: StoredWindowSize): void => {
  if (!Number.isFinite(size.width) || !Number.isFinite(size.height)) {
    return;
  }

  safeWrite(
    WINDOW_SIZE_STORAGE_KEY,
    JSON.stringify({
      width: Math.max(100, Math.round(size.width)),
      height: Math.max(100, Math.round(size.height)),
    })
  );
};

