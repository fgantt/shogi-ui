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
    
    const { command, playerId } = e.data;
    const commandParts = command.split(' ');
    const baseCommand = commandParts[0];

    const postInfoToMainThread = (info: string) => {
        self.postMessage({ info, playerId });
    };

    if (baseCommand === 'go') {
        // Restore game state if it exists before starting search
        if (gameState) {
            console.log('Worker: Restoring game state, depth:', gameState.depth);
            handler.process_command(`position sfen ${gameState.position}`);
            handler.set_depth(gameState.depth);
            console.log('Worker: Handler depth after restoration:', handler.get_depth());
        } else {
            console.log('Worker: No game state, using default depth');
        }
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
        
        // Update game state for setoption commands (especially depth)
        if (baseCommand === 'setoption') {
            console.log('Worker: Processing setoption command:', command);
            const depthMatch = command.match(/setoption name depth value (\d+)/);
            if (depthMatch) {
                const newDepth = parseInt(depthMatch[1], 10);
                console.log('Worker: Setting depth to:', newDepth);
                if (gameState) {
                    gameState.depth = newDepth;
                } else {
                    gameState = {
                        position: '',
                        currentPlayer: 'black',
                        depth: newDepth
                    };
                }
                console.log('Worker: Updated gameState.depth to:', gameState.depth);
                console.log('Worker: Handler depth after setoption:', handler.get_depth());
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
