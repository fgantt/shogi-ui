import init, { WasmUsiHandler } from '../../pkg/shogi_engine.js';

let handler: WasmUsiHandler | null = null;

async function initWasm() {
    if (!handler) {
        try {
            await init();
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

    const { command, ...args } = e.data;

    const postInfoToMainThread = (info: string) => {
        self.postMessage(info);
    };

    if (command === 'go') {
        const goCommand = `go ${Object.entries(args).map(([key, value]) => `${key} ${value}`).join(' ')}`;
        handler.go_with_callback(goCommand, postInfoToMainThread);
    } else {
        const result = handler.process_command(command);
        self.postMessage(result);
    }
};