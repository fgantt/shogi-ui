import { useEffect } from 'react';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

interface UseTauriEventsOptions {
  onUsiMessage?: (engineId: string, message: string) => void;
  onUsiError?: (engineId: string, error: string) => void;
}

/**
 * Custom hook to subscribe to Tauri events from USI engines
 * 
 * @param engineId - The ID of the engine to listen to (or 'all' for all engines)
 * @param options - Callback functions for handling events
 */
export function useTauriEvents(
  engineId: string | null,
  options: UseTauriEventsOptions
) {
  const { onUsiMessage, onUsiError } = options;

  useEffect(() => {
    if (!engineId) return;

    const unlistenFunctions: UnlistenFn[] = [];

    const setupListeners = async () => {
      // Listen for USI messages
      if (onUsiMessage) {
        const messageEvent = `usi-message::${engineId}`;
        const unlisten = await listen<string>(messageEvent, (event) => {
          onUsiMessage(engineId, event.payload);
        });
        unlistenFunctions.push(unlisten);
      }

      // Listen for USI errors
      if (onUsiError) {
        const errorEvent = `usi-error::${engineId}`;
        const unlisten = await listen<string>(errorEvent, (event) => {
          onUsiError(engineId, event.payload);
        });
        unlistenFunctions.push(unlisten);
      }
    };

    setupListeners();

    // Cleanup
    return () => {
      unlistenFunctions.forEach((unlisten) => unlisten());
    };
  }, [engineId, onUsiMessage, onUsiError]);
}

/**
 * Hook to listen to multiple engines at once
 */
export function useMultipleEngineEvents(
  engineIds: string[],
  options: UseTauriEventsOptions
) {
  const { onUsiMessage, onUsiError } = options;

  useEffect(() => {
    if (engineIds.length === 0) return;

    const unlistenFunctions: UnlistenFn[] = [];

    const setupListeners = async () => {
      for (const engineId of engineIds) {
        // Listen for USI messages
        if (onUsiMessage) {
          const messageEvent = `usi-message::${engineId}`;
          const unlisten = await listen<string>(messageEvent, (event) => {
            onUsiMessage(engineId, event.payload);
          });
          unlistenFunctions.push(unlisten);
        }

        // Listen for USI errors
        if (onUsiError) {
          const errorEvent = `usi-error::${engineId}`;
          const unlisten = await listen<string>(errorEvent, (event) => {
            onUsiError(engineId, event.payload);
          });
          unlistenFunctions.push(unlisten);
        }
      }
    };

    setupListeners();

    // Cleanup
    return () => {
      unlistenFunctions.forEach((unlisten) => unlisten());
    };
  }, [engineIds, onUsiMessage, onUsiError]);
}

/**
 * Simpler hook for listening to a single engine's messages
 */
export function useEngineMessages(
  engineId: string | null,
  onMessage: (message: string) => void
) {
  useTauriEvents(engineId, { onUsiMessage: (_, msg) => onMessage(msg) });
}

