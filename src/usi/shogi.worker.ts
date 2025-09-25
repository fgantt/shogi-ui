import init, { WasmUsiHandler, init_panic_hook } from '../../pkg/shogi_engine.js';

let handler: WasmUsiHandler | null = null;

async function initWasm() {
    if (!handler) {
        try {
            await init();
            init_panic_hook();
            handler = new WasmUsiHandler();
        } catch (error) {
            console.error('Failed to initialize WebAssembly module in worker:', error);
            throw error;
        }
    }
}

self.onmessage = async (e: MessageEvent) => {
    await initWasm();
    if (!handler) {
        console.error("WASM handler not initialized in worker.");
        return;
    }

    const { command } = e.data;
    const commandParts = command.split(' ');
    const baseCommand = commandParts[0];

    const postInfoToMainThread = (info: string) => {
        self.postMessage(info);
    };

    if (baseCommand === 'go') {
        handler.go_with_callback(command, postInfoToMainThread);
    } else {
        const result = handler.process_command(command);
        self.postMessage(result);
    }
};
