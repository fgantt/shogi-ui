#react #wasm #shogi

Of course. To stream messages from your Rust/WASM Shogi engine to your React UI, you'll pass a JavaScript callback function to your Rust code. The Rust engine will then invoke this callback whenever it generates a new info message.

To prevent the engine's search from freezing the browser's UI, the entire WASM module should be run inside a **Web Worker**.

Here's a complete breakdown of the architecture and implementation.

---

## Core Concept: The Callback Mechanism

The fundamental pattern is to bridge the two environments by giving the Rust/WASM module a direct line to your JavaScript code.

1. **React (Main Thread)**: Creates a Web Worker.
    
2. **Web Worker**: Imports and initializes the WASM module. It defines a JS function (e.g., `postInfoToMainThread`) that sends data back to the main thread.
    
3. **React -> Web Worker**: To start the engine, React sends a message to the worker.
    
4. **Web Worker -> Rust/WASM**: The worker calls the main Rust "thinking" function, passing the `postInfoToMainThread` function as an argument (a callback).
    
5. **Rust/WASM**: As the engine searches, it repeatedly calls the provided callback function with new info strings.
    
6. **Callback Execution**: Each call executes the `postInfoToMainThread` function in the worker, which relays the message back to the React component on the main thread.
    
7. **React State Update**: React receives the message and updates its state, causing the UI to re-render with the new information.
    

---

## Step 1: Rust Crate Setup ⚙️

First, ensure your Rust code is set up to accept and call a JavaScript function.

### Cargo.toml

Make sure you have `js-sys` as a dependency to handle the JavaScript function type.

Ini, TOML

```
[package]
name = "shogi_engine"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
wasm-bindgen = "0.2"
js-sys = "0.3"

# Optional, but highly recommended for better debugging
console_error_panic_hook = { version = "0.1.7", optional = true }
```

### lib.rs

Define a function that takes a string (like the board state in SFEN format) and a reference to a JavaScript `Function`.

Rust

```
use wasm_bindgen::prelude::*;
use js_sys::Function;
use std::thread;
use std::time::Duration;

// This function allows us to log to the browser console from Rust
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen]
pub fn start_thinking(board_sfen: &str, on_info: &Function) {
    log(&format!("Rust received board state: {}", board_sfen));

    // This loop simulates the engine thinking and generating info.
    // Replace this with your actual search algorithm (e.g., alpha-beta).
    for i in 1..=20 {
        let info_message = format!("info depth {} score cp {} pv 7g7f", i, 150 + i * 10);

        // Create a JsValue from our Rust string.
        let message = JsValue::from_str(&info_message);
        
        // `this` context for the JS function call. `null` is fine.
        let this = JsValue::NULL;

        // Call the JavaScript callback function with our message.
        // The `call1` method invokes the function with one argument.
        if let Err(e) = on_info.call1(&this, &message) {
            log(&format!("Error calling JS callback: {:?}", e));
        }

        // Simulate work being done. In a real engine, your search
        // would take time here.
        thread::sleep(Duration::from_millis(250));
    }
    
    // Announce the best move when done
    let best_move_message = JsValue::from_str("bestmove 7g7f");
    let this = JsValue::NULL;
    let _ = on_info.call1(&this, &best_move_message);
}
```

---

## Step 2: The Web Worker

The worker acts as an intermediary, ensuring the heavy computation in WASM doesn't block the UI.

### public/worker.ts

Create a new file in your `public` directory (or configure Vite to handle workers elsewhere).

TypeScript

```
import init, { start_thinking } from '../pkg/shogi_engine.js'; // Adjust path to your WASM package

// Initialize the WASM module. This is an async operation.
const wasmModule = init();

self.onmessage = async (event: MessageEvent) => {
  // Wait for the WASM module to be fully loaded and instantiated.
  await wasmModule;
  
  const { sfen } = event.data;

  // This is the callback function we will pass to Rust.
  // It takes one argument (the info string) and posts it
  // back to the main thread (our React component).
  const postInfoToMainThread = (info: string) => {
    self.postMessage(info);
  };

  try {
    // Start the engine's thinking process.
    start_thinking(sfen, postInfoToMainThread);
  } catch (error) {
    console.error("Error in WASM execution:", error);
    self.postMessage("Error: See worker console for details.");
  }
};
```

---

## Step 3: React Component Integration 🚀

Finally, wire up the React component to use the worker and display the streamed messages.

### ShogiMonitor.tsx

TypeScript

```
import React, { useState, useEffect, useRef } from 'react';

export const ShogiMonitor = () => {
  const [messages, setMessages] = useState<string[]>([]);
  const [isThinking, setIsThinking] = useState(false);
  const workerRef = useRef<Worker | null>(null);

  // Setup and cleanup the Web Worker
  useEffect(() => {
    // Create a new worker. The `new URL(...)` syntax is standard for Vite.
    const worker = new Worker(new URL('/worker.ts', import.meta.url), {
      type: 'module',
    });
    workerRef.current = worker;

    // Listen for messages from the worker
    worker.onmessage = (event: MessageEvent<string>) => {
      const message = event.data;
      
      if (message.startsWith('bestmove')) {
        setIsThinking(false);
      }
      
      // Add the new message to the top of the list
      setMessages(prev => [message, ...prev]);
    };

    // Cleanup: terminate the worker when the component unmounts
    return () => {
      worker.terminate();
    };
  }, []);

  const handleStartThinking = () => {
    if (workerRef.current) {
      setMessages([]); // Clear previous messages
      setIsThinking(true);
      // Send a message to the worker to start the process
      workerRef.current.postMessage({
        sfen: 'lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1',
      });
    }
  };

  return (
    <div>
      <h2>Shogi Engine Monitor</h2>
      <button onClick={handleStartThinking} disabled={isThinking}>
        {isThinking ? '🤔 Thinking...' : 'Start Thinking'}
      </button>
      <pre style={{
        backgroundColor: '#222',
        color: '#0f0',
        padding: '1rem',
        borderRadius: '5px',
        height: '300px',
        overflowY: 'scroll',
        fontFamily: 'monospace',
        whiteSpace: 'pre-wrap'
      }}>
        {messages.join('\n')}
      </pre>
    </div>
  );
};
```

This setup provides a robust, non-blocking way to get real-time feedback from your Rust-based engine directly into your React application.