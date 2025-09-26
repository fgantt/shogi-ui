import init, { WasmUsiHandler, init_panic_hook } from '../../pkg/shogi_engine.js';

let isWasmInitialized = false;
let isProcessing = false;
const messageQueue: MessageEvent[] = [];

// Game state to maintain between handler instances
let gameState: {
    position: string;
    currentPlayer: string;
    depth: number;
} | null = null;

async function initWasm() {
    if (!isWasmInitialized) {
        try {
            await init();
            init_panic_hook();
            isWasmInitialized = true;
        } catch (error) {
            console.error('Failed to initialize WebAssembly module in worker:', error);
            throw error;
        }
    }
}

async function processMessage(e: MessageEvent) {
    if (!isWasmInitialized) {
        console.error("WASM not initialized in worker.");
        return;
    }

    // Create a new handler for each message to avoid borrow checker issues
    const handler = new WasmUsiHandler();
    
    // Restore game state if it exists
    if (gameState) {
        handler.process_command(`position sfen ${gameState.position}`);
        handler.set_depth(gameState.depth);
    }
    
    const { command, playerId } = e.data;
    const commandParts = command.split(' ');
    const baseCommand = commandParts[0];

    const postInfoToMainThread = (info: string) => {
        self.postMessage({ info, playerId });
    };

    if (baseCommand === 'go') {
        handler.go_with_callback(command, postInfoToMainThread);
    } else {
        const result = handler.process_command(command);
        
        // Update game state for position commands
        if (baseCommand === 'position') {
            const positionMatch = command.match(/position sfen (.+)/);
            if (positionMatch) {
                gameState = {
                    position: positionMatch[1],
                    currentPlayer: command.includes(' b ') ? 'black' : 'white',
                    depth: gameState?.depth || 3
                };
            }
        }
        
        self.postMessage({ result, playerId });
    }
}

async function processQueue() {
    if (isProcessing || messageQueue.length === 0) {
        return;
    }

    isProcessing = true;
    const message = messageQueue.shift();
    
    if (message) {
        try {
            await processMessage(message);
        } catch (error) {
            console.error('Error processing message:', error);
        }
    }
    
    isProcessing = false;
    
    // Process next message if any
    if (messageQueue.length > 0) {
        setTimeout(processQueue, 0);
    }
}

self.onmessage = async (e: MessageEvent) => {
    await initWasm();
    messageQueue.push(e);
    processQueue();
};
