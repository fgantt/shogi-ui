/**
 * Utility functions for Tauri-based engine communication
 * This module provides a bridge between the game logic and Tauri backend
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import type { CommandResponse } from '../types/engine';

/**
 * Spawn and initialize an engine
 */
export async function spawnEngine(
  engineId: string,
  name: string,
  path: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await invoke<CommandResponse>('spawn_engine', {
      engineId,
      name,
      path,
    });

    if (!response.success) {
      return { success: false, error: response.message };
    }

    return { success: true };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}

/**
 * Send a USI command to an engine
 */
export async function sendUsiCommand(
  engineId: string,
  command: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await invoke<CommandResponse>('send_usi_command', {
      engineId,
      command,
    });

    if (!response.success) {
      return { success: false, error: response.message };
    }

    return { success: true };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}

/**
 * Wait for a specific USI response from the engine
 * Returns a promise that resolves when the expected message is received
 */
export async function waitForUsiResponse(
  engineId: string,
  expectedPrefix: string,
  timeoutMs: number = 5000
): Promise<{ success: boolean; message?: string; error?: string }> {
  console.log(`[waitForUsiResponse] Setting up listener for "${expectedPrefix}" from engine ${engineId}`);
  
  return new Promise(async (resolve) => {
    let unlisten: UnlistenFn | null = null;
    let timeout: NodeJS.Timeout | null = null;

    const cleanup = () => {
      if (timeout) clearTimeout(timeout);
      if (unlisten) unlisten();
    };

    console.log(`[waitForUsiResponse] Listening to event: usi-message::${engineId}`);
    
    // CRITICAL: Wait for listener to be registered BEFORE starting timeout
    try {
      unlisten = await listen<string>(`usi-message::${engineId}`, (event) => {
        const message = event.payload;
        console.log(`[waitForUsiResponse] Received message: "${message}"`);
        
        if (message.startsWith(expectedPrefix)) {
          console.log(`[waitForUsiResponse] Message matches "${expectedPrefix}"! Resolving.`);
          cleanup();
          resolve({ success: true, message });
        } else {
          console.log(`[waitForUsiResponse] Message doesn't match "${expectedPrefix}", continuing to wait...`);
        }
      });
      
      console.log(`[waitForUsiResponse] Listener registered successfully, starting timeout`);
      
      // Only start timeout AFTER listener is registered
      timeout = setTimeout(() => {
        console.error(`[waitForUsiResponse] TIMEOUT waiting for "${expectedPrefix}" from engine ${engineId}`);
        cleanup();
        resolve({ success: false, error: `Timeout waiting for ${expectedPrefix}` });
      }, timeoutMs);
      
    } catch (error) {
      console.error(`[waitForUsiResponse] Error setting up listener:`, error);
      cleanup();
      resolve({ success: false, error: String(error) });
    }
  });
}

/**
 * Send isready and wait for readyok
 */
export async function sendIsReadyAndWait(
  engineId: string
): Promise<{ success: boolean; error?: string }> {
  console.log('[sendIsReadyAndWait] Setting up listener for readyok from engine:', engineId);
  
  // Create a promise that will handle the entire flow
  return new Promise(async (resolve) => {
    console.log('[sendIsReadyAndWait] Registering listener...');
    
    // Set up the listener first and wait for it to be registered
    let unlisten: (() => void) | null = null;
    let timeout: NodeJS.Timeout | null = null;
    
    const cleanup = () => {
      if (timeout) clearTimeout(timeout);
      if (unlisten) unlisten();
    };
    
    try {
      // Import listen from Tauri
      const { listen } = await import('@tauri-apps/api/event');
      
      console.log('[sendIsReadyAndWait] Setting up event listener for usi-message::' + engineId);
      unlisten = await listen<string>(`usi-message::${engineId}`, (event) => {
        const message = event.payload;
        console.log(`[sendIsReadyAndWait] Received message: "${message}"`);
        
        if (message.startsWith('readyok')) {
          console.log('[sendIsReadyAndWait] Received readyok! Resolving.');
          cleanup();
          resolve({ success: true });
        }
      });
      
      console.log('[sendIsReadyAndWait] Listener registered successfully');
      
      // Set timeout
      timeout = setTimeout(() => {
        console.error('[sendIsReadyAndWait] TIMEOUT waiting for readyok');
        cleanup();
        resolve({ success: false, error: 'Timeout waiting for readyok' });
      }, 10000);
      
      // Now send the command
      console.log('[sendIsReadyAndWait] Sending isready to engine:', engineId);
      const sendResult = await sendUsiCommand(engineId, 'isready');
      
      if (!sendResult.success) {
        console.error('[sendIsReadyAndWait] Failed to send isready:', sendResult.error);
        cleanup();
        resolve({ success: false, error: sendResult.error });
        return;
      }
      
      console.log('[sendIsReadyAndWait] isready sent successfully, waiting for response...');
      
    } catch (error) {
      console.error('[sendIsReadyAndWait] Error:', error);
      cleanup();
      resolve({ success: false, error: String(error) });
    }
  });
}

/**
 * Stop an engine
 */
export async function stopEngine(
  engineId: string
): Promise<{ success: boolean; error?: string }> {
  try {
    const response = await invoke<CommandResponse>('stop_engine', {
      engineId,
    });

    if (!response.success) {
      return { success: false, error: response.message };
    }

    return { success: true };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}

/**
 * Get the path to the built-in engine
 */
export async function getBuiltinEnginePath(): Promise<{
  success: boolean;
  path?: string;
  error?: string;
}> {
  try {
    const response = await invoke<CommandResponse<{ path: string }>>(
      'get_builtin_engine_path'
    );

    if (!response.success || !response.data) {
      return { success: false, error: response.message };
    }

    return { success: true, path: response.data.path };
  } catch (error) {
    return { success: false, error: String(error) };
  }
}

/**
 * Initialize a game session with an engine
 * This sends the initial USI handshake and prepares the engine for play
 */
export async function initializeEngineSession(
  engineId: string,
  enginePath: string,
  engineName: string
): Promise<{ success: boolean; error?: string }> {
  // First spawn the engine
  const spawnResult = await spawnEngine(engineId, engineName, enginePath);
  if (!spawnResult.success) {
    return spawnResult;
  }

  // Give the engine a moment to initialize
  await new Promise(resolve => setTimeout(resolve, 100));

  // Send usinewgame command to reset for a new game
  const newGameResult = await sendUsiCommand(engineId, 'usinewgame');
  if (!newGameResult.success) {
    return newGameResult;
  }

  // Send isready and wait for readyok
  const readyResult = await sendUsiCommand(engineId, 'isready');
  if (!readyResult.success) {
    return readyResult;
  }

  return { success: true };
}

/**
 * Request a move from the engine
 */
export async function requestEngineMove(
  engineId: string,
  position: string,
  moves: string[],
  timeControl: {
    btime?: number;
    wtime?: number;
    byoyomi?: number;
  }
): Promise<{ success: boolean; error?: string }> {
  // Build position command
  let positionCmd = `position ${position}`;
  if (moves.length > 0) {
    positionCmd += ` moves ${moves.join(' ')}`;
  }

  // Send position
  const posResult = await sendUsiCommand(engineId, positionCmd);
  if (!posResult.success) {
    return posResult;
  }

  // Build go command
  const goParams: string[] = [];
  if (timeControl.btime !== undefined) {
    goParams.push(`btime ${timeControl.btime}`);
  }
  if (timeControl.wtime !== undefined) {
    goParams.push(`wtime ${timeControl.wtime}`);
  }
  if (timeControl.byoyomi !== undefined) {
    goParams.push(`byoyomi ${timeControl.byoyomi}`);
  }

  const goCmd = goParams.length > 0 ? `go ${goParams.join(' ')}` : 'go';

  // Send go command
  const goResult = await sendUsiCommand(engineId, goCmd);
  if (!goResult.success) {
    return goResult;
  }

  return { success: true };
}

/**
 * Stop the engine from thinking
 */
export async function stopEngineThinking(
  engineId: string
): Promise<{ success: boolean; error?: string }> {
  return await sendUsiCommand(engineId, 'stop');
}

/**
 * Parse a USI bestmove response
 * Example: "bestmove 7g7f" or "bestmove 7g7f ponder 3c3d"
 */
export function parseBestMove(usiMessage: string): {
  move: string | null;
  ponder?: string;
} {
  const parts = usiMessage.trim().split(/\s+/);
  
  if (parts[0] !== 'bestmove') {
    return { move: null };
  }

  const move = parts[1];
  
  // Check for ponder move
  if (parts.length >= 4 && parts[2] === 'ponder') {
    return { move, ponder: parts[3] };
  }

  return { move };
}

/**
 * Parse engine info messages
 * Example: "info depth 5 score cp 120 nodes 1234 nps 5000 pv 7g7f 3c3d"
 */
export function parseEngineInfo(usiMessage: string): {
  depth?: number;
  score?: number;
  nodes?: number;
  nps?: number;
  pv?: string[];
  time?: number;
} {
  if (!usiMessage.startsWith('info ')) {
    return {};
  }

  const parts = usiMessage.split(/\s+/);
  const info: ReturnType<typeof parseEngineInfo> = {};

  for (let i = 1; i < parts.length; i++) {
    switch (parts[i]) {
      case 'depth':
        info.depth = parseInt(parts[++i]);
        break;
      case 'score':
        if (parts[i + 1] === 'cp') {
          info.score = parseInt(parts[i + 2]);
          i += 2;
        }
        break;
      case 'nodes':
        info.nodes = parseInt(parts[++i]);
        break;
      case 'nps':
        info.nps = parseInt(parts[++i]);
        break;
      case 'time':
        info.time = parseInt(parts[++i]);
        break;
      case 'pv':
        info.pv = parts.slice(i + 1);
        i = parts.length; // End loop
        break;
    }
  }

  return info;
}

