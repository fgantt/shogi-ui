# How to USI and tsshogi
Here’s a guide on how to use tsshogi (TypeScript Shogi) library:

## 1) Installation
npm install tsshogi

or

yarn add tsshogi



## 2) Basic Usage

### Importing and reading a KIF (or other format) file

```ts
import fs from "node:fs";
import { importKIF } from "tsshogi";

const data = fs.readFileSync("sample.kifu", "utf-8");
const record = importKIF(data);
if (record instanceof Error) {
  throw record;
}

// Traverse moves
while (record.goForward()) {
  console.log(record.current.ply, record.current.displayText);
}

// Append a new move
const move = record.position.createMoveByUSI("4a3b");
if (!move) throw new Error("Invalid SFEN");

// append returns boolean (illegal move, etc.)
if (!record.append(move)) throw new Error("Illegal move");
```

## 3) Core Concepts

- **Record**: represents an entire game or game tree with branches; supports navigation forward/back and branch switching.   
- **Position**: represents the current board state. 
- **Board**, **Hand**, **Move**, **Square**, **Color**: classes/enums modeling the game.  
- There are corresponding **Immutable** interface types if you want read-only access. 


## 4) Useful Helper Functions

- `detectRecordFormat` – to guess whether input is KIF, SFEN, CSA, etc. 
- Constants like `InitialPositionSFEN` for starting setup. 
- Piece counting, jishogi scoring and detection utilities (`countExistingPieces`, `judgeJishogiDeclaration`, etc.). 


## 5) Typical Flow Outline

1. **Read** game notation (e.g., KIF, SFEN, CSA).
2. **Import** into a `Record` using format-specific importers.
3. **Navigate** moves using `goForward`, `goBack`, or manually target a node.
4. **Manipulate** (e.g., append moves, switch branches).
5. **Inspect** positions via `record.position`, `Board`, `Hand`, etc.
6. **Export** or analyze via your own logic or serialization.


## 6) Code Example: Simple Game Loader

```ts
import fs from "node:fs";
import { detectRecordFormat, importKIF /* or importCSA, importSFEN? */ } from "tsshogi";

const raw = fs.readFileSync("game.kif", "utf-8");
const format = detectRecordFormat(raw);
let record;
if (format === "KIF") {
  record = importKIF(raw);
} else {
  throw new Error("Unsupported format: " + format);
}
if (record instanceof Error) throw record;

// Print all moves
do {
  console.log(record.current.displayText);
} while (record.goForward());
```

## 7) Recommendations / Next Steps
Browse the GitHub repository for the most up-to-date code and usage examples. 
GitHub

Explore type definitions to understand available methods and class structures.

If you specify what you’re trying to build (e.g., a GUI board, an engine interface, notation converter), I can help tailor the usage further.

Let me know if you’d like help with examples for importing other formats, exporting, or manipulating the game tree!

# tsshogi as “glue” between a GUI board and an engine 
It already knows how to represent:

- The record (tree of moves, comments, branches)

- The position (board + hands + side-to-move)

- Moves (validation, notation conversions)

Here’s how you can structure things:

## 🔧 Architecture
```GUI Board   <—>   tsshogi Record/Position   <—>   Engine```

### GUI Board

Displays current record.position

Sends user input (drag-drop or click) → a move string (e.g., USI "7g7f")

### tsshogi

Validates moves (position.createMoveByUSI(...))

Updates the Record (append, goBack, branch switching)

Provides data for rendering (position.board, position.hands, position.turn)

### Engine

Reads current position via SFEN (position.toSFEN())

Returns a move string (USI)

You pass it back through tsshogi to validate & apply to the record

## ⚡ Core Flow
1. Setup
```ts
import { Record, InitialPositionSFEN } from "tsshogi";

// Start fresh record
const record = Record.newBySFEN(InitialPositionSFEN);
```

2. Handling a GUI move
```ts
// User drags/drops → gives you a USI move like "7g7f"
const move = record.position.createMoveByUSI("7g7f");

if (!move) {
  console.error("Illegal move");
} else if (!record.append(move)) {
  console.error("Move rejected (maybe checkmate or illegal state)");
} else {
  console.log("Move added:", move.toUSI());
  // Update GUI board using record.position.board
}
```
3. Sending position to engine
```ts
// Engine often speaks USI protocol → SFEN is the main position format
const sfen = record.position.toSFEN();

// Example: send to engine process
engine.send(`position sfen ${sfen} moves`);

// Later engine says: bestmove 2g2f
const engineMove = "2g2f";
const move2 = record.position.createMoveByUSI(engineMove);
if (move2) record.append(move2);
```
4. Updating the GUI
The board state is inside record.position:
```ts
// Board squares
for (const [sq, piece] of record.position.board.entries()) {
  console.log(`Square ${sq.toUSI()}:`, piece?.toUSI());
}

// Hands
console.log(record.position.blackHand, record.position.whiteHand);

// Turn
console.log(record.position.turn); // 0 = Black, 1 = White
```
So your GUI can just re-render from record.position.

## ✨ Extra Features
Branching: If a player tries an alternate move, tsshogi can fork branches.

Navigation: Use record.goBack(), record.goForward(), record.goto() to step through the game.

Formats: Import/export KIF, CSA, SFEN — useful for saving/loading games.

# Running engine as a JS worker / WASM or connect to an external USI engine process (like YaneuraOu)?

👍 You want tsshogi in the middle, with the ability to:

✅ Run local engine (JS/WASM in a web worker, or Node)

✅ Talk to external USI engines (via process or socket)

✅ Keep the GUI unified (doesn’t care where the moves came from)

Here’s how you can design it:

## 🔧 Unified Design
```
GUI <-> Controller <-> tsshogi Record/Position <-> EngineAdapter
                                               ├─ LocalEngineAdapter
                                               └─ USIEngineAdapter
```
GUI: renders record.position, sends user moves.

Controller: validates moves via tsshogi, applies them to the record, and asks the engine when needed.

EngineAdapter: abstract interface for any engine.

LocalEngineAdapter: runs engine logic in JS/WASM worker.

USIEngineAdapter: talks to external USI-compatible engines.

## ⚡ Engine Adapter Interface
```ts
interface EngineAdapter {
  init(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  getBestMove(): Promise<string>; // returns USI move string
  quit(): Promise<void>;
}
```
## 🖥️ Local Engine (JS/WASM)
```ts
class LocalEngineAdapter implements EngineAdapter {
  constructor(private worker: Worker) {}

  async init() {
    this.worker.postMessage({ type: "init" });
  }

  async setPosition(sfen: string, moves: string[]) {
    this.worker.postMessage({ type: "position", sfen, moves });
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (e: MessageEvent) => {
        if (e.data.type === "bestmove") {
          this.worker.removeEventListener("message", handler);
          resolve(e.data.move);
        }
      };
      this.worker.addEventListener("message", handler);
      this.worker.postMessage({ type: "go" });
    });
  }

  async quit() {
    this.worker.terminate();
  }
}
```

## ⚙️ External USI Engine (Node.js child_process)
```ts
import { spawn, ChildProcessWithoutNullStreams } from "child_process";

class USIEngineAdapter implements EngineAdapter {
  private proc!: ChildProcessWithoutNullStreams;

  constructor(private path: string) {}

  async init() {
    this.proc = spawn(this.path);
    this.proc.stdin.write("usi\n");
    // TODO: parse "usiok"
  }

  async setPosition(sfen: string, moves: string[]) {
    const moveStr = moves.length ? " moves " + moves.join(" ") : "";
    this.proc.stdin.write(`position sfen ${sfen}${moveStr}\n`);
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      this.proc.stdout.on("data", (data) => {
        const line = data.toString();
        if (line.startsWith("bestmove")) {
          resolve(line.split(" ")[1].trim());
        }
      });
      this.proc.stdin.write("go\n");
    });
  }

  async quit() {
    this.proc.stdin.write("quit\n");
  }
}
```

## 🎮 Controller Example (gluing GUI + tsshogi + Engine)
```ts
import { Record, InitialPositionSFEN } from "tsshogi";

class ShogiController {
  private record = Record.newBySFEN(InitialPositionSFEN);

  constructor(private engine: EngineAdapter) {}

  async userMove(usi: string) {
    const move = this.record.position.createMoveByUSI(usi);
    if (!move || !this.record.append(move)) throw new Error("Illegal move");
    this.updateGUI();

    // Ask engine
    await this.engine.setPosition(
      this.record.root.position.toSFEN(),
      this.record.moves.map((m) => m.toUSI())
    );
    const reply = await this.engine.getBestMove();
    this.engineMove(reply);
  }

  private engineMove(usi: string) {
    const move = this.record.position.createMoveByUSI(usi);
    if (move) this.record.append(move);
    this.updateGUI();
  }

  private updateGUI() {
    // send record.position to GUI
    console.log(this.record.position.toSFEN());
  }
}
```

✅ This way:

If you want local JS/WASM engine, pass a LocalEngineAdapter.

If you want external USI engine, pass a USIEngineAdapter.

The controller + GUI don’t need to change.

## Minimal working demo (say, index.ts + a simple mock LocalEngine worker) that you can run right away to see the flow end-to-end?

Great 🚀 — let’s put together a minimal demo with:

index.ts: sets up the controller + GUI stub.

localEngineWorker.ts: a fake “engine” that just picks a random legal move.

This will run entirely in Node.js / browser bundler with no external engine required, so you can test the flow.

1. localEngineWorker.ts
```ts
// A dummy local engine that picks a random legal move
import { parentPort } from "worker_threads";
import { Position, InitialPositionSFEN } from "tsshogi";

let position = Position.newBySFEN(InitialPositionSFEN);

parentPort?.on("message", (msg) => {
  if (msg.type === "init") {
    position = Position.newBySFEN(InitialPositionSFEN);
  }

  if (msg.type === "position") {
    position = Position.newBySFEN(msg.sfen);
    for (const usi of msg.moves) {
      const m = position.createMoveByUSI(usi);
      if (m) position.doMove(m);
    }
  }

  if (msg.type === "go") {
    const moves = position.generateMoves();
    if (moves.length === 0) {
      parentPort?.postMessage({ type: "bestmove", move: "resign" });
    } else {
      const pick = moves[Math.floor(Math.random() * moves.length)];
      parentPort?.postMessage({ type: "bestmove", move: pick.toUSI() });
    }
  }
});
```

2. engineAdapters.ts
```ts
import { Worker } from "worker_threads";

export interface EngineAdapter {
  init(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  getBestMove(): Promise<string>;
  quit(): Promise<void>;
}

export class LocalEngineAdapter implements EngineAdapter {
  constructor(private worker: Worker) {}

  async init() {
    this.worker.postMessage({ type: "init" });
  }

  async setPosition(sfen: string, moves: string[]) {
    this.worker.postMessage({ type: "position", sfen, moves });
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (msg: any) => {
        if (msg.type === "bestmove") {
          this.worker.off("message", handler);
          resolve(msg.move);
        }
      };
      this.worker.on("message", handler);
      this.worker.postMessage({ type: "go" });
    });
  }

  async quit() {
    this.worker.terminate();
  }
}
```

3. controller.ts
```ts
import { Record, InitialPositionSFEN } from "tsshogi";
import type { EngineAdapter } from "./engineAdapters";

export class ShogiController {
  private record = Record.newBySFEN(InitialPositionSFEN);

  constructor(private engine: EngineAdapter) {}

  async userMove(usi: string) {
    const move = this.record.position.createMoveByUSI(usi);
    if (!move || !this.record.append(move)) throw new Error("Illegal move");

    this.updateGUI("User");

    await this.engine.setPosition(
      this.record.root.position.toSFEN(),
      this.record.moves.map((m) => m.toUSI())
    );

    const reply = await this.engine.getBestMove();
    this.engineMove(reply);
  }

  private engineMove(usi: string) {
    const move = this.record.position.createMoveByUSI(usi);
    if (move) this.record.append(move);
    this.updateGUI("Engine");
  }

  private updateGUI(by: string) {
    console.log(`[${by}] -> ${this.record.position.toSFEN()}`);
  }
}
```

4. index.ts
```ts
import { Worker } from "worker_threads";
import { LocalEngineAdapter } from "./engineAdapters";
import { ShogiController } from "./controller";

async function main() {
  const worker = new Worker("./localEngineWorker.js"); // transpile .ts first
  const engine = new LocalEngineAdapter(worker);
  await engine.init();

  const controller = new ShogiController(engine);

  // Simulate user moves
  await controller.userMove("7g7f"); // pawn push
  await controller.userMove("2g2f"); // another pawn push

  await engine.quit();
}

main();
```

▶️ How to Run
Install deps:

npm install tsshogi
Put the above files in src/.

Compile with TypeScript:

tsc
Run:

node dist/index.js
You should see console logs like:

[User] -> lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1
[Engine] -> ...

# 👉 Extending demo so the same controller can switch seamlessly between LocalEngineAdapter and a USIEngineAdapter (external engine) with zero GUI changes

The only difference will be which EngineAdapter you pass into the controller.

1. Update engineAdapters.ts
```ts
import { Worker } from "worker_threads";
import { spawn, ChildProcessWithoutNullStreams } from "child_process";

export interface EngineAdapter {
  init(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  getBestMove(): Promise<string>;
  quit(): Promise<void>;
}

// -------------------- Local Worker Engine --------------------
export class LocalEngineAdapter implements EngineAdapter {
  constructor(private worker: Worker) {}

  async init() {
    this.worker.postMessage({ type: "init" });
  }

  async setPosition(sfen: string, moves: string[]) {
    this.worker.postMessage({ type: "position", sfen, moves });
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (msg: any) => {
        if (msg.type === "bestmove") {
          this.worker.off("message", handler);
          resolve(msg.move);
        }
      };
      this.worker.on("message", handler);
      this.worker.postMessage({ type: "go" });
    });
  }

  async quit() {
    this.worker.terminate();
  }
}

// -------------------- External USI Engine --------------------
export class USIEngineAdapter implements EngineAdapter {
  private proc!: ChildProcessWithoutNullStreams;

  constructor(private path: string) {}

  async init() {
    this.proc = spawn(this.path);
    this.proc.stdin.write("usi\n");

    // Wait for "usiok"
    await new Promise<void>((resolve) => {
      const handler = (data: Buffer) => {
        if (data.toString().includes("usiok")) {
          this.proc.stdout.off("data", handler);
          resolve();
        }
      };
      this.proc.stdout.on("data", handler);
    });
  }

  async setPosition(sfen: string, moves: string[]) {
    const moveStr = moves.length ? " moves " + moves.join(" ") : "";
    this.proc.stdin.write(`position sfen ${sfen}${moveStr}\n`);
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (data: Buffer) => {
        const lines = data.toString().split("\n");
        for (const line of lines) {
          if (line.startsWith("bestmove")) {
            const move = line.split(" ")[1].trim();
            this.proc.stdout.off("data", handler);
            resolve(move);
          }
        }
      };
      this.proc.stdout.on("data", handler);
      this.proc.stdin.write("go\n");
    });
  }

  async quit() {
    this.proc.stdin.write("quit\n");
  }
}
```

2. Controller stays the same ✅

(controller.ts does not change at all.)

3. index.ts – choose engine dynamically
```ts
import { Worker } from "worker_threads";
import { LocalEngineAdapter, USIEngineAdapter } from "./engineAdapters";
import { ShogiController } from "./controller";

async function main() {
  // ----- Option A: Local Worker Engine -----
  const worker = new Worker("./localEngineWorker.js"); // compiled worker
  const engine = new LocalEngineAdapter(worker);

  // ----- Option B: External USI Engine -----
  // const engine = new USIEngineAdapter("/path/to/yaneuraou"); 

  await engine.init();
  const controller = new ShogiController(engine);

  // Simulate user moves
  await controller.userMove("7g7f"); // Black pawn push
  await controller.userMove("8c8d"); // White pawn push

  await engine.quit();
}

main();
```

✅ Results
If you pass LocalEngineAdapter(worker), your GUI plays against the built-in dummy engine.

If you pass USIEngineAdapter("/path/to/engine"), your GUI plays against a real external USI engine.

The controller and GUI code don’t change at all — they only talk to EngineAdapter.

# 👉 Adapt for a browser build (where you can’t spawn a USI engine, but you can still run WASM engines in workers)

In the browser things change a little:

You cannot spawn child processes → so you can’t talk to external USI engines directly.

But you can run engines compiled to WebAssembly (WASM) inside Web Workers.

The adapter pattern still works → just different implementations.

Here’s how you can adapt the design:

## 🔧 Browser Architecture
```
GUI  <->  Controller  <->  tsshogi.Record/Position  <->  EngineAdapter
                                                 └─ BrowserLocalEngineAdapter (WASM in Worker)
```

1. Browser Worker Engine (browserLocalEngineWorker.ts)
This is basically the same as our Node dummy engine, but uses self instead of parentPort:

```ts
// browserLocalEngineWorker.ts
import { Position, InitialPositionSFEN } from "tsshogi";

let position = Position.newBySFEN(InitialPositionSFEN);

self.onmessage = (e: MessageEvent) => {
  const msg = e.data;
  if (msg.type === "init") {
    position = Position.newBySFEN(InitialPositionSFEN);
  }

  if (msg.type === "position") {
    position = Position.newBySFEN(msg.sfen);
    for (const usi of msg.moves) {
      const m = position.createMoveByUSI(usi);
      if (m) position.doMove(m);
    }
  }

  if (msg.type === "go") {
    const moves = position.generateMoves();
    if (moves.length === 0) {
      (self as any).postMessage({ type: "bestmove", move: "resign" });
    } else {
      const pick = moves[Math.floor(Math.random() * moves.length)];
      (self as any).postMessage({ type: "bestmove", move: pick.toUSI() });
    }
  }
};
```

2. Browser Engine Adapter (browserEngineAdapters.ts)
```ts
export interface EngineAdapter {
  init(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  getBestMove(): Promise<string>;
  quit(): Promise<void>;
}

export class BrowserLocalEngineAdapter implements EngineAdapter {
  constructor(private worker: Worker) {}

  async init() {
    this.worker.postMessage({ type: "init" });
  }

  async setPosition(sfen: string, moves: string[]) {
    this.worker.postMessage({ type: "position", sfen, moves });
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (e: MessageEvent) => {
        if (e.data.type === "bestmove") {
          this.worker.removeEventListener("message", handler);
          resolve(e.data.move);
        }
      };
      this.worker.addEventListener("message", handler);
      this.worker.postMessage({ type: "go" });
    });
  }

  async quit() {
    this.worker.terminate();
  }
}
```

3. Controller (same as Node ✅)

Your ShogiController does not change — it only depends on the EngineAdapter interface.

4. Browser Entry (index.ts)
```ts
import { ShogiController } from "./controller";
import { BrowserLocalEngineAdapter } from "./browserEngineAdapters";

// In browser bundlers (Vite, Webpack, etc.)
// you can import a worker with a loader:
import EngineWorker from "./browserLocalEngineWorker?worker";

async function main() {
  const worker = new EngineWorker();
  const engine = new BrowserLocalEngineAdapter(worker);
  await engine.init();

  const controller = new ShogiController(engine);

  // Example: user makes a move
  await controller.userMove("7g7f"); // Pawn forward
  await controller.userMove("8c8d"); // Opponent pawn
}

main();
```

✅ Summary
Node.js:

LocalEngineAdapter (worker_threads)

USIEngineAdapter (spawn external process)

Browser:

BrowserLocalEngineAdapter (Web Worker with JS/WASM engine)

(No direct USI engines, unless you proxy via a server)


# 👉 How to proxy an external USI engine over WebSocket so your browser GUI can still use engines like YaneuraOu remotely

We’ll do it with a WebSocket proxy:

Browser GUI → WebSocket → Node Server → spawn USI Engine

1. Node Server (engine proxy)
We’ll make a lightweight server that:

Starts the USI engine with child_process.spawn.

Forwards setPosition / go requests from the browser.

Sends back bestmove.

```ts
// usi-proxy-server.ts
import { WebSocketServer } from "ws";
import { spawn } from "child_process";

const wss = new WebSocketServer({ port: 8080 });

wss.on("connection", (ws) => {
  console.log("Client connected");

  // spawn engine (change path!)
  const engine = spawn("/path/to/yaneuraou");

  engine.stdout.on("data", (data) => {
    const text = data.toString();
    ws.send(JSON.stringify({ type: "engine", output: text }));

    // quick parse "bestmove"
    for (const line of text.split("\n")) {
      if (line.startsWith("bestmove")) {
        ws.send(JSON.stringify({ type: "bestmove", move: line.split(" ")[1] }));
      }
    }
  });

  ws.on("message", (raw) => {
    const msg = JSON.parse(raw.toString());
    if (msg.type === "cmd") {
      engine.stdin.write(msg.cmd + "\n");
    }
  });

  ws.on("close", () => {
    engine.stdin.write("quit\n");
    engine.kill();
    console.log("Client disconnected, engine closed");
  });
});

console.log("USI proxy server running on ws://localhost:8080");
2. Browser Engine Adapter (WebSocket client)
export interface EngineAdapter {
  init(): Promise<void>;
  setPosition(sfen: string, moves: string[]): Promise<void>;
  getBestMove(): Promise<string>;
  quit(): Promise<void>;
}

export class WebSocketUSIEngineAdapter implements EngineAdapter {
  private socket!: WebSocket;

  constructor(private url: string) {}

  async init() {
    this.socket = new WebSocket(this.url);

    return new Promise<void>((resolve) => {
      this.socket.onopen = () => {
        this.socket.send(JSON.stringify({ type: "cmd", cmd: "usi" }));
        resolve();
      };
    });
  }

  async setPosition(sfen: string, moves: string[]) {
    const moveStr = moves.length ? " moves " + moves.join(" ") : "";
    this.socket.send(
      JSON.stringify({ type: "cmd", cmd: `position sfen ${sfen}${moveStr}` })
    );
  }

  async getBestMove() {
    return new Promise<string>((resolve) => {
      const handler = (e: MessageEvent) => {
        const msg = JSON.parse(e.data);
        if (msg.type === "bestmove") {
          this.socket.removeEventListener("message", handler);
          resolve(msg.move);
        }
      };
      this.socket.addEventListener("message", handler);
      this.socket.send(JSON.stringify({ type: "cmd", cmd: "go" }));
    });
  }

  async quit() {
    this.socket.send(JSON.stringify({ type: "cmd", cmd: "quit" }));
    this.socket.close();
  }
}
```

3. Controller (no changes ✅)

Your ShogiController is still the same.

4. Browser index.ts
```ts
import { ShogiController } from "./controller";
import { WebSocketUSIEngineAdapter } from "./websocketEngineAdapter";

async function main() {
  const engine = new WebSocketUSIEngineAdapter("ws://localhost:8080");
  await engine.init();

  const controller = new ShogiController(engine);

  // Example: user moves
  await controller.userMove("7g7f"); // black pawn
  await controller.userMove("3c3d"); // white pawn
}

main();
```

✅ Result

Start usi-proxy-server.ts on your Node backend.

Open your browser app → it connects to the proxy over WebSocket.

Proxy spawns YaneuraOu (or another USI engine) and relays messages.

From the browser’s perspective, it’s just another EngineAdapter.


# 👉 Make this proxy multi-client aware (so multiple browsers can play simultaneously, each with its own engine process)

1. Multi-Client USI Proxy Server
```ts
// usi-proxy-multi.ts
import { WebSocketServer } from "ws";
import { spawn, ChildProcessWithoutNullStreams } from "child_process";

const wss = new WebSocketServer({ port: 8080 });

wss.on("connection", (ws) => {
  console.log("Client connected");

  // Spawn a separate engine for this client
  const engine: ChildProcessWithoutNullStreams = spawn("/path/to/yaneuraou");

  // Forward engine stdout to the client
  engine.stdout.on("data", (data) => {
    const text = data.toString();
    ws.send(JSON.stringify({ type: "engine", output: text }));

    // Quick parse for bestmove
    for (const line of text.split("\n")) {
      if (line.startsWith("bestmove")) {
        ws.send(JSON.stringify({ type: "bestmove", move: line.split(" ")[1] }));
      }
    }
  });

  engine.stderr.on("data", (data) => {
    console.error(`Engine error: ${data.toString()}`);
  });

  ws.on("message", (raw) => {
    try {
      const msg = JSON.parse(raw.toString());
      if (msg.type === "cmd") {
        engine.stdin.write(msg.cmd + "\n");
      }
    } catch (e) {
      console.error("Invalid message:", raw.toString());
    }
  });

  ws.on("close", () => {
    engine.stdin.write("quit\n");
    engine.kill();
    console.log("Client disconnected, engine process terminated");
  });

  ws.on("error", (err) => {
    console.error("WebSocket error:", err);
  });
});

console.log("Multi-client USI proxy server running on ws://localhost:8080");
```

2. Browser Adapter (no changes)

Your WebSocketUSIEngineAdapter works exactly the same — each client connects to the server and gets its own engine automatically.

```ts
const engine = new WebSocketUSIEngineAdapter("ws://localhost:8080");
await engine.init();
```
Each browser tab gets a fresh engine process on the server.

3. Notes / Recommendations

Resource management

Each engine process uses CPU/memory. Limit total clients or use a queue if your server can’t handle too many simultaneous games.

Engine selection

You could modify the adapter to let clients pick which engine or level to use.

Example: spawn("/path/to/yaneuraou", ["-depth", "10"]) per client.

Timeouts

If an engine hangs, consider a timeout for getBestMove() and forcibly kill/restart.

Security

Only allow authorized clients to connect if exposed to the internet.

✅ Now you have a fully browser-compatible, multi-client USI engine setup:

Browser GUI → WebSocket → Node Proxy → One USI Engine per client

The ShogiController stays identical for all cases — local worker, single-node, or multi-client remote engine.

# diagram + sequence flow showing how GUI moves → tsshogi → engine → GUI work in multi-client mode

Great! Let’s create a diagram + sequence flow showing the full interaction between GUI, tsshogi, and the multi-client engine setup. I’ll also describe each step in detail.

## 1️⃣ Architecture Diagram
```
+----------------+        +----------------+       +------------------------+
|  Browser GUI   | <----> | ShogiController | <---- | EngineAdapter (client) |
|                |        |  (tsshogi)     |       |   WebSocketUSIAdapter  |
+----------------+        +----------------+       +------------------------+
                                                        |
                                                        v
                                                 +----------------+
                                                 | Node Proxy WS  |
                                                 |  (multi-client)|
                                                 +----------------+
                                                        |
                                                        v
                                           +------------------------+
                                           | USI Engine (one per client) |
                                           +------------------------+
```
Browser GUI: displays board, receives user input.

ShogiController: manages moves via tsshogi.Record and communicates with engine adapter.

EngineAdapter: abstracts engine communication, can be local (worker) or remote (WebSocket to Node).

Node Proxy: runs USI engine processes, one per client.

USI Engine: computes best moves.

## 2️⃣ Sequence Flow
User Move: GUI sends a move (USI format) to ShogiController.

Validation: Controller uses tsshogi.Record → position.createMoveByUSI() → append.

Update GUI: Controller notifies GUI of new board state (position.board, hands, turn).

Engine Request: Controller calls EngineAdapter.setPosition() and getBestMove().

WebSocket Adapter: Browser adapter sends JSON over WebSocket to Node proxy.

Proxy Forwards: Node proxy writes USI commands to the engine process.

Engine Computes: USI engine calculates best move.

Proxy Returns: Node proxy parses bestmove from engine stdout and sends JSON back to browser.

Controller Receives: Controller applies engine move via tsshogi.Record.

Update GUI: Controller notifies GUI of updated board state.

Example Flow (Pawn Push)

Step	Actor	Action
1.	GUI	User moves pawn 7g7f
2.	Controller	Validate & append move via tsshogi
3.	Controller → GUI	Update board display
4.	Controller → Adapter	setPosition() & getBestMove()
5.	Adapter → WebSocket	Send JSON { type: "cmd", cmd: "position sfen ..." }
6.	Node Proxy → Engine	position sfen ... moves 7g7f
7.	Engine	Calculates best move, e.g., 3c3d
8.	Node Proxy → Adapter	Send JSON { type: "bestmove", move: "3c3d" }
9.	Controller	Append engine move via tsshogi
10.	Controller → GUI	Update board display with engine move

✅ This diagram + flow makes it clear how your GUI stays agnostic of engine type:

Local JS/WASM worker → same sequence minus WebSocket.

Remote USI engine → sequence passes through Node proxy.

# Detailed explanation of the USI protocol.
The USI (Universal Shogi Interface) protocol is the standard way for Shogi engines and GUIs (graphical user interfaces) to communicate with each other. It’s modeled after the UCI protocol used in chess engines, but adapted for the rules and needs of Shogi.

Here’s a detailed explanation of how it works:

1. Overview
Purpose: Connects a Shogi GUI (like Shogidokoro, Lishogi, or your custom app) with a Shogi engine (like Bonanza, Apery, YaneuraOu).

Communication: Simple text-based protocol over standard input/output (stdin/stdout).

Flow: GUI sends commands → Engine responds → GUI interprets responses → Board/game state updates.

2. Initialization Phase
When the GUI launches an engine, it sends:

```
usi
```

The engine responds with:

```
id name <engine-name>
id author <author-name>
option name <name> type <type> default <value> [min <min> max <max>]
usiok
```

id → Provides engine name and author.

option → Lists configurable engine settings (hash size, threads, book file, etc.).

usiok → Confirms it supports USI and is ready.

3. Setting Options
The GUI can configure engine options:

```
setoption name Threads value 4
setoption name Hash value 1024
setoption → Adjusts engine parameters before starting a game.
```

When done:

```
isready
```

Engine responds:

```
readyok
```

This ensures the engine has applied settings and is synchronized.

4. New Game Start

The GUI tells the engine to reset:

```
usinewgame
```

This clears the search history, TT (transposition table), and prepares for a new match.

5. Position Setup

The GUI sets the current board position with SFEN (Shogi Forsyth–Edwards Notation).

```
Example: Starting position
position startpos moves
Example: Custom SFEN with moves
position sfen lnsgkgsnl/1r5b1/p1ppppppp/9/9/9/P1PPPPPPP/1B5R1/LNSGKGSNL b - 1 moves 7g7f 3c3d
sfen → Board state.

b or w → Side to move (black = sente, white = gote).

moves → Sequence of moves from that position.
```

6. Search Commands

The GUI requests the engine to compute moves:

```
go btime 60000 wtime 60000 byoyomi 1000
btime / wtime → Remaining time (ms).
```

byoyomi → Extra time per move.

Variants exist (e.g., go infinite, go ponder).

7. Engine Output During Search

While thinking, the engine sends info lines:

```
info depth 10 score cp 36 pv 7g7f 3c3d 2g2f
info depth 12 score mate + pv 7g7f 3c3d 2g2f
```
depth → Search depth (plies).

score cp → Evaluation in centipawns.

score mate → Indicates mate in N moves.

pv → Principal variation (best line the engine sees).

This allows the GUI to display analysis.

8. Engine Best Move Response

When done, the engine sends:

```
bestmove 7g7f
```
Or with pondering:

```
bestmove 7g7f ponder 3c3d
```

bestmove → Engine’s chosen move.

ponder → Move it expects the opponent to reply with.

9. Stopping the Engine

If the GUI wants to stop a search early:

```
stop
```

Engine responds with the current best move.

10. Quitting

When the GUI closes:

```
quit
```
Engine exits gracefully.

11. Full Example Session

```
usi
id name YaneuraOu 2023
id author Issei Yamamoto
usiok
setoption name Threads value 4
setoption name Hash value 1024
isready
readyok
usinewgame
position startpos moves 7g7f
go btime 300000 wtime 300000 byoyomi 5000
info depth 10 score cp 24 pv 7g7f 3c3d
info depth 12 score cp 34 pv 7g7f 3c3d 2g2f
bestmove 7g7f
quit
```

✅ That’s the full lifecycle of the USI protocol — initialization, configuration, game setup, position updates, searching, reporting results, and termination.

# Map the USI protocol flow into setup with GUI ↔ tsshogi ↔ Engine Adapter ↔ Engine.

Who sends what at each stage:

## 🔄 USI Protocol in Your Architecture
1. Initialization

GUI (your app) → tells tsshogi to load an engine:

```
usi
```

Engine (via adapter/proxy) → responds:
```
id name YaneuraOu
id author Issei
option name Threads type spin default 1 min 1 max 16
option name Hash type spin default 256 min 1 max 4096
usiok
```

👉 tsshogi’s role:

Parses this handshake.

Passes engine options up to the GUI so you can show them in settings.

2. Set Options

GUI → user changes options in UI (e.g., threads = 4).

tsshogi translates to:

```
setoption name Threads value 4
setoption name Hash value 1024
```

Engine → applies them silently.

GUI → sync point:
```
isready
```

Engine:
```
readyok
```
👉 tsshogi’s role:

Exposes a clean API like engine.setOption("Threads", 4) and handles the wire protocol.

Confirms readiness before letting GUI proceed.

3. Start New Game

GUI (start button in your app) →

```
usinewgame
```

Engine → clears tables.

tsshogi just forwards.

4. Position Setup

GUI (via tsshogi) builds SFEN from its board state:

```
position startpos moves 7g7f 3c3d
```

Engine stores this position internally.

👉 tsshogi’s role:

Provides helpers like position.setSfen(...) or position.applyMove(...).

You don’t need to manually build SFEN strings.

5. Search

GUI (user presses “Engine move”) →

```
go btime 300000 wtime 300000 byoyomi 5000
```

Engine → while searching:
```
info depth 10 score cp 24 pv 7g7f 3c3d
info depth 12 score cp 34 pv 7g7f 3c3d 2g2f
```

Engine → final answer:

```
bestmove 7g7f ponder 3c3d
```

👉 tsshogi’s role:

Parses info lines into structured JSON objects: { depth, score, pv }.

GUI can update eval bar, PV line, search depth display.

Provides event hooks like onBestMove(...).

6. Stop / Quit

GUI (user clicks stop) →

```
stop
```

Engine → returns best move found so far.

GUI exit →

```
quit
```

👉 tsshogi’s role:

Wraps stop/quit into functions (engine.stop(), engine.quit()).

🔧 Example with tsshogi API (conceptual)
```ts
import { Engine } from "tsshogi";

const engine = new Engine("/path/to/yaneuraou");

// Initialize
await engine.send("usi");
await engine.waitFor("usiok");

// Set options
await engine.setOption("Threads", 4);
await engine.setOption("Hash", 1024);
await engine.sync(); // isready → readyok

// Start new game
await engine.send("usinewgame");

// Set position
await engine.position("startpos", ["7g7f", "3c3d"]);

// Start search
engine.go({ btime: 300000, wtime: 300000, byoyomi: 5000 });

// Listen for analysis updates
engine.on("info", info => {
  console.log("Depth:", info.depth, "Score:", info.score, "PV:", info.pv);
});

// Best move
engine.on("bestmove", move => {
  console.log("Engine plays:", move);
});
```

✅ So in short:

GUI = user interaction + board rendering

tsshogi = speaks USI, translates commands into structured API

Engine = computes moves

# Example TypeScript integration layer that connects your GUI board events to the tsshogi engine interface


## 🎮 GUI ↔ tsshogi Engine Integration (TypeScript)

1. Define Engine Wrapper

We’ll make a small class to hide the raw USI details:

```ts
// engine-wrapper.ts
import { Engine } from "tsshogi";

export class ShogiEngine {
  private engine: Engine;

  constructor(enginePath: string) {
    this.engine = new Engine(enginePath);
  }

  async init() {
    await this.engine.send("usi");
    await this.engine.waitFor("usiok");

    await this.engine.setOption("Threads", 4);
    await this.engine.setOption("Hash", 1024);
    await this.engine.sync(); // isready -> readyok
  }

  async newGame() {
    await this.engine.send("usinewgame");
  }

  async setPosition(moves: string[]) {
    await this.engine.position("startpos", moves);
  }

  async go(time: { btime: number; wtime: number; byoyomi: number }) {
    this.engine.go(time);
  }

  onInfo(callback: (info: any) => void) {
    this.engine.on("info", callback);
  }

  onBestMove(callback: (move: string) => void) {
    this.engine.on("bestmove", callback);
  }

  stop() {
    this.engine.send("stop");
  }

  quit() {
    this.engine.send("quit");
  }
}
```

2. Connect to GUI Board

Suppose your GUI board emits events like:

```ts
board.onMove((move: string) => {
  controller.handlePlayerMove(move);
});
```

We can bind that into tsshogi:

```ts
// controller.ts
import { ShogiEngine } from "./engine-wrapper";

export class GameController {
  private moves: string[] = [];
  private engine: ShogiEngine;

  constructor(enginePath: string) {
    this.engine = new ShogiEngine(enginePath);
  }

  async init() {
    await this.engine.init();
    await this.engine.newGame();

    this.engine.onInfo(info => {
      console.log("Engine analysis:", info);
      // GUI: update eval bar, PV, etc.
    });

    this.engine.onBestMove(move => {
      console.log("Engine plays:", move);
      // GUI: apply engine move on board
    });
  }

  async handlePlayerMove(move: string) {
    this.moves.push(move);
    await this.engine.setPosition(this.moves);

    // After player moves, let engine think
    await this.engine.go({
      btime: 300000,
      wtime: 300000,
      byoyomi: 5000,
    });
  }
}
```

3. Hook Up GUI

Example usage:

```ts
// app.ts
import { GameController } from "./controller";

const controller = new GameController("/path/to/yaneuraou");

async function startGame(board: any) {
  await controller.init();

  board.onMove((move: string) => {
    controller.handlePlayerMove(move);
  });
}

startGame(myShogiBoardUI);
```

## 🔑 Flow Summary
User clicks piece → GUI creates a move string (e.g. "7g7f")

GameController → appends move, calls engine.setPosition(...)

Engine → searches via go

tsshogi → emits info (analysis) and bestmove

GUI → updates search bar + plays engine move


# How to convert between tsshogi’s Move objects and GUI move strings (like "7g7f", drops, promotions), so the two sides speak the same language

This is the tricky part that makes the GUI ↔ engine bridge really usable.
The USI protocol + tsshogi uses SFEN move notation (like "7g7f", "P*5d", "2h3f+"), while your GUI probably has its own move objects ({from, to, piece, promotion}).

Let’s build converters both ways.

## 🔄 Move Conversion Between GUI and USI/tsshogi
1. Coordinate System

USI format:

Files (1–9) from right to left (Black’s perspective).

Ranks (a–i) from top to bottom.

So "7g7f" means: from file 7, rank g → file 7, rank f.

Drops: "P*5d" means drop a Pawn on 5d.

Promotion: "2h3f+" means promote when moving.

2. GUI Move Object

Example structure:

```ts
type GuiMove = {
  from?: { file: number; rank: number }; // source square, 1–9
  to: { file: number; rank: number };    // destination square
  piece: string;                         // "P", "L", "N", etc.
  promotion?: boolean;
  drop?: boolean;
};
```

3. Converter: GUI → USI String
```ts
// utils/moveConverter.ts

// rank 1–9 -> letter a–i
function rankToLetter(rank: number): string {
  return String.fromCharCode("a".charCodeAt(0) + (rank - 1));
}

export function guiMoveToUsi(move: GuiMove): string {
  if (move.drop) {
    // Drop move: "P*5d"
    return `${move.piece}*${move.to.file}${rankToLetter(move.to.rank)}`;
  }

  const from = `${move.from!.file}${rankToLetter(move.from!.rank)}`;
  const to = `${move.to.file}${rankToLetter(move.to.rank)}`;
  return move.promotion ? `${from}${to}+` : `${from}${to}`;
}
```

✅ Examples:

```ts
guiMoveToUsi({ from: {file:7, rank:7}, to: {file:7, rank:6}, piece: "P" });
// → "7g7f"

guiMoveToUsi({ to: {file:5, rank:4}, piece: "P", drop: true });
// → "P*5d"
```

4. Converter: USI String → GUI Move

```ts
function letterToRank(letter: string): number {
  return letter.charCodeAt(0) - "a".charCodeAt(0) + 1;
}

export function usiToGuiMove(usi: string): GuiMove {
  if (usi.includes("*")) {
    // Drop
    const [piece, dest] = usi.split("*");
    return {
      to: { file: parseInt(dest[0], 10), rank: letterToRank(dest[1]) },
      piece,
      drop: true,
    };
  }

  const fromFile = parseInt(usi[0], 10);
  const fromRank = letterToRank(usi[1]);
  const toFile = parseInt(usi[2], 10);
  const toRank = letterToRank(usi[3]);

  return {
    from: { file: fromFile, rank: fromRank },
    to: { file: toFile, rank: toRank },
    promotion: usi.endsWith("+"),
    piece: "?", // you can look this up from board state
  };
}
```

✅ Examples:

```ts
usiToGuiMove("7g7f");
// → { from:{file:7, rank:7}, to:{file:7, rank:6}, promotion:false, piece:"?" }

usiToGuiMove("P*5d");
// → { to:{file:5, rank:4}, piece:"P", drop:true }
```

5. Integration into Controller

When player moves on GUI:

```ts
const usiMove = guiMoveToUsi(move);
moves.push(usiMove);
engine.setPosition(moves);
```

When engine responds:

```ts
engine.onBestMove(usiMove => {
  const guiMove = usiToGuiMove(usiMove);
  board.applyMove(guiMove);
});
```

✅ Now your GUI and engine speak the same language:

GUI works in {file, rank}

Engine works in USI SFEN strings

The converter bridges them.

# Extend the USI → GUI converter so the "piece" field isn’t "?" but is actually resolved from the current board state inside tsshogi.Position.

🔄 Using tsshogi.Position to Look Up Pieces

tsshogi already tracks the board and hands. You can ask it what piece is on a square before making a move. That’s how we know which piece actually moved.

1. Accessing a Piece from a Square

tsshogi uses a 9×9 board indexed by square IDs (e.g., "7g", "5d").

Example:

```ts
import { Position, Square, Piece } from "tsshogi";

const pos = new Position(); // startpos
const piece = pos.getPiece(Square.fromString("7g")); 
console.log(piece); // → e.g. "P"
```

2. Improved USI → GUI Conversion
```ts
import { Position, Square } from "tsshogi";

function letterToRank(letter: string): number {
  return letter.charCodeAt(0) - "a".charCodeAt(0) + 1;
}

export function usiToGuiMove(usi: string, pos: Position): GuiMove {
  if (usi.includes("*")) {
    // Drop move, e.g. "P*5d"
    const [piece, dest] = usi.split("*");
    return {
      to: { file: parseInt(dest[0], 10), rank: letterToRank(dest[1]) },
      piece,
      drop: true,
    };
  }

  // Normal move
  const fromFile = parseInt(usi[0], 10);
  const fromRank = letterToRank(usi[1]);
  const toFile = parseInt(usi[2], 10);
  const toRank = letterToRank(usi[3]);
  const promotion = usi.endsWith("+");

  // Look up piece on board
  const fromSquare = Square.fromString(`${fromFile}${usi[1]}`);
  const piece = pos.getPiece(fromSquare); // returns "P", "L", etc.

  return {
    from: { file: fromFile, rank: fromRank },
    to: { file: toFile, rank: toRank },
    promotion,
    piece,
  };
}
```
3. Workflow in the Controller

When applying an engine move:

```ts
engine.onBestMove(usiMove => {
  // Convert using current Position
  const guiMove = usiToGuiMove(usiMove, position);

  // Update GUI
  board.applyMove(guiMove);

  // Advance Position so it stays in sync
  position.doMove(usiMove);
});
```

When player moves:

```ts
const usi = guiMoveToUsi(guiMove);
position.doMove(usi);  // update tsshogi position
engine.setPosition(moves); // send to engine
```

✅ Now you have a two-way sync:

GUI clicks → USI string (with promotion/drop info).

USI string → GUI move (with correct piece looked up).

tsshogi.Position keeps both consistent.

# Handle captures and hand updates

We’ll let tsshogi.Position be the source of truth — after each move (from GUI or engine), we’ll:

Apply it to Position

Update the GUI board squares and hand pieces

🏯 Captures & Hands with tsshogi

1. How Captures Work in tsshogi

When you call position.doMove(usiMove):

If a piece is captured, it is automatically moved into the capturer’s hand.

The captured piece is demoted if needed (e.g., a promoted Pawn becomes a Pawn in hand).

You can query hands via position.getHand(player).

Example:

```ts
import { Player } from "tsshogi";

const hand = position.getHand(Player.Black); 
console.log(hand); 
// Map of { P: 2, L: 1, ... } pieces in hand
```

2. Updating Hands After a Move

When applying any move:
```ts
// Update board + hand after engine move
engine.onBestMove(usiMove => {
  position.doMove(usiMove);       // <-- apply move in tsshogi
  const guiMove = usiToGuiMove(usiMove, position);
  
  board.applyMove(guiMove);       // update board squares
  
  // Update hands in GUI
  const blackHand = position.getHand("black");
  const whiteHand = position.getHand("white");
  gui.updateHands({ black: blackHand, white: whiteHand });
});
```

3. Example GUI Hand Data Structure
```ts
type GuiHands = {
  black: Record<string, number>; // { P: 2, L: 1, ... }
  white: Record<string, number>;
};
```
Your GUI can render each piece with its count. Example: "P x 2".

4. Capture Example Walkthrough

Imagine startpos. Black plays:

```ts
const move = "7g7f";   // pawn forward
position.doMove(move);
```

White replies:
```ts
const move = "3c3d";   // pawn forward
position.doMove(move);
```

Later White plays:
```ts
const move = "7f7g+";  // White pawn captures Black pawn and promotes
position.doMove(move);
```
Now:

```ts
position.getHand(Player.White) → { P: 1 }
```

Board square 7g now has White promoted pawn (+P)

Your GUI would show White’s hand has a Pawn.

5. Drops (using the hand)

When a player chooses a drop in GUI:

GUI move object:

{ piece: "P", to: {file:5, rank:4}, drop:true }
Converted to USI: "P*5d".

Apply in engine:

```ts
position.doMove("P*5d");
```

Hands update automatically (hand count decreases by 1).

✅ With this setup:

```ts
position.doMove(usiMove) updates board + hands
```

You query `position.getHand()` to refresh GUI hands

GUI always stays in sync with engine

# Sample GameController class that ties all this together (GUI events ↔ tsshogi Position ↔ Engine communication)


GUI input → USI string → tsshogi.Position → Engine

Engine output (bestmove) → tsshogi.Position → GUI update (board + hands)

This way, the GUI never touches engine or tsshogi directly — it only talks to GameController.

## 🏯 Example GameController
```ts
import { Position, Square, Player } from "tsshogi";
import { guiMoveToUsi, usiToGuiMove } from "./converters"; // your helper functions
import type { GuiMove, GuiHands } from "./types";

export class GameController {
  private position: Position;
  private moveHistory: string[] = [];

  constructor(
    private gui: {
      applyMove: (move: GuiMove) => void;
      updateHands: (hands: GuiHands) => void;
    },
    private engine: {
      setPosition: (moves: string[]) => void;
      go: () => void;
      onBestMove: (cb: (usiMove: string) => void) => void;
    }
  ) {
    this.position = new Position(); // startpos

    // Listen to engine output
    this.engine.onBestMove((usiMove: string) => {
      this.applyEngineMove(usiMove);
    });

    // Initial hands sync
    this.syncHands();
  }

  // === Called when player makes a GUI move ===
  handlePlayerMove(guiMove: GuiMove) {
    const usi = guiMoveToUsi(guiMove);

    // Update position
    this.position.doMove(usi);
    this.moveHistory.push(usi);

    // Update GUI
    this.gui.applyMove(guiMove);
    this.syncHands();

    // Send to engine
    this.engine.setPosition(this.moveHistory);
    this.engine.go();
  }

  // === Called when engine responds ===
  private applyEngineMove(usiMove: string) {
    this.position.doMove(usiMove);
    this.moveHistory.push(usiMove);

    const guiMove = usiToGuiMove(usiMove, this.position);

    this.gui.applyMove(guiMove);
    this.syncHands();
  }

  // === Sync hands from tsshogi into GUI ===
  private syncHands() {
    const blackHand = this.position.getHand(Player.Black);
    const whiteHand = this.position.getHand(Player.White);

    this.gui.updateHands({
      black: Object.fromEntries(blackHand),
      white: Object.fromEntries(whiteHand),
    });
  }

  // Reset game
  reset() {
    this.position = new Position();
    this.moveHistory = [];
    this.syncHands();
  }
}
```
## 🖼 How It Fits Together
GUI → Controller

Player drags/drops piece

GameController.handlePlayerMove() is called

Converts to USI, updates position, syncs GUI, sends to engine

Engine → Controller

Engine returns "bestmove ...".

Controller applies with applyEngineMove().

Updates Position, move history, and GUI

Hands

Always refreshed via position.getHand(Player.Black/White)

## ✅ Benefits
Single source of truth (tsshogi.Position)

GUI is dumb: it only renders what controller tells it

Engine doesn’t care about GUI — only USI

# Extend our GameController with undo / redo support.
tsshogi makes this easy because you can:

Keep a move history (string[] of USI moves).

Rebuild any position using Position.setSfen("sfen … moves …").

That means undo/redo = trimming or extending the move history and resetting the Position.

## 🏯 Updated GameController with Undo/Redo
```ts
import { Position, Player } from "tsshogi";
import { guiMoveToUsi, usiToGuiMove } from "./converters"; 
import type { GuiMove, GuiHands } from "./types";

export class GameController {
  private position: Position;
  private moveHistory: string[] = [];
  private redoStack: string[] = [];

  constructor(
    private gui: {
      applyMove: (move: GuiMove) => void;
      updateHands: (hands: GuiHands) => void;
      renderFull: (position: Position) => void; // re-render full board
    },
    private engine: {
      setPosition: (moves: string[]) => void;
      go: () => void;
      onBestMove: (cb: (usiMove: string) => void) => void;
    }
  ) {
    this.position = new Position(); // startpos

    this.engine.onBestMove((usiMove: string) => {
      this.applyEngineMove(usiMove);
    });

    this.syncHands();
  }

  // === Called when player makes a GUI move ===
  handlePlayerMove(guiMove: GuiMove) {
    const usi = guiMoveToUsi(guiMove);

    this.position.doMove(usi);
    this.moveHistory.push(usi);
    this.redoStack = []; // clear redo when new branch chosen

    this.gui.applyMove(guiMove);
    this.syncHands();

    this.engine.setPosition(this.moveHistory);
    this.engine.go();
  }

  // === Called when engine responds ===
  private applyEngineMove(usiMove: string) {
    this.position.doMove(usiMove);
    this.moveHistory.push(usiMove);
    this.redoStack = []; // clear redo stack

    const guiMove = usiToGuiMove(usiMove, this.position);
    this.gui.applyMove(guiMove);
    this.syncHands();
  }

  // === Undo last move ===
  undo() {
    if (this.moveHistory.length === 0) return;

    const lastMove = this.moveHistory.pop()!;
    this.redoStack.push(lastMove);

    this.rebuildPosition();
  }

  // === Redo last undone move ===
  redo() {
    if (this.redoStack.length === 0) return;

    const move = this.redoStack.pop()!;
    this.moveHistory.push(move);

    this.rebuildPosition();
  }

  // === Reset full position from move history ===
  private rebuildPosition() {
    this.position = new Position();
    for (const move of this.moveHistory) {
      this.position.doMove(move);
    }

    // Force GUI to re-render entire state (board + hands)
    this.gui.renderFull(this.position);
    this.syncHands();

    // Tell engine current state
    this.engine.setPosition(this.moveHistory);
  }

  private syncHands() {
    const blackHand = this.position.getHand(Player.Black);
    const whiteHand = this.position.getHand(Player.White);

    this.gui.updateHands({
      black: Object.fromEntries(blackHand),
      white: Object.fromEntries(whiteHand),
    });
  }

  reset() {
    this.position = new Position();
    this.moveHistory = [];
    this.redoStack = [];
    this.gui.renderFull(this.position);
    this.syncHands();
  }
}
```
## 🖼 GUI Responsibilities
The GUI now needs 3 entry points from the controller:

applyMove(move) → animate piece move (normal case).

updateHands(hands) → update captured pieces display.

renderFull(position) → re-render full board (for undo/redo/reset).

## ✅ Features Now
Normal play loop (GUI ↔ engine) ✅

Undo (ctrl+z style) ✅

Redo (ctrl+shift+z style) ✅

Reset to startpos ✅

Engine always kept in sync ✅

# Extend to support branching game trees (like a full replay mode with variations, not just linear undo/redo)

Undo/redo works for a linear history, but for Shogi (and Chess-like games), you want variations. That would make it closer to a proper shogi database tool.

Every move can branch into one or more child variations.

You can navigate back to any earlier move and try a new line without destroying the old one.

You can still undo/redo linearly within a branch.

This is how professional Shogi GUIs (Shogidokoro, ShogiGUI) work, and how engines manage PGN/KIF/KI2/CSA trees.

## 🏯 Core Idea: Game Tree
Instead of moveHistory: string[], we keep a tree structure:

```ts
interface GameNode {
  move?: string;              // USI move from parent → this node
  parent?: GameNode;
  children: GameNode[];
  comment?: string;
}

class GameTree {
  root: GameNode;
  current: GameNode;

  constructor() {
    this.root = { children: [] };
    this.current = this.root;
  }

  addMove(usiMove: string): GameNode {
    const newNode: GameNode = { move: usiMove, parent: this.current, children: [] };
    this.current.children.push(newNode);
    this.current = newNode;
    return newNode;
  }

  goTo(node: GameNode) {
    this.current = node;
  }

  undo() {
    if (this.current.parent) {
      this.current = this.current.parent;
    }
  }

  redo(index: number = 0) {
    if (this.current.children[index]) {
      this.current = this.current.children[index];
    }
  }

  getMainlineMoves(): string[] {
    const moves: string[] = [];
    let node = this.root;
    while (node.children[0]) {
      node = node.children[0];
      if (node.move) moves.push(node.move);
    }
    return moves;
  }

  getLineToCurrent(): string[] {
    const moves: string[] = [];
    let node = this.current;
    while (node.parent) {
      if (node.move) moves.unshift(node.move);
      node = node.parent;
    }
    return moves;
  }
}
```

This gives us a tree of moves, not just a list.
You can branch anywhere, store comments/annotations, and reconstruct any position.

## 🏯 GameController with Tree
We swap moveHistory with GameTree:

```ts
import { Position, Player } from "tsshogi";
import { guiMoveToUsi, usiToGuiMove } from "./converters";
import type { GuiMove, GuiHands } from "./types";
import { GameTree } from "./GameTree";

export class GameController {
  private position: Position;
  private tree: GameTree;

  constructor(
    private gui: {
      applyMove: (move: GuiMove) => void;
      updateHands: (hands: GuiHands) => void;
      renderFull: (position: Position) => void;
    },
    private engine: {
      setPosition: (moves: string[]) => void;
      go: () => void;
      onBestMove: (cb: (usiMove: string) => void) => void;
    }
  ) {
    this.position = new Position();
    this.tree = new GameTree();

    this.engine.onBestMove((usiMove: string) => {
      this.applyEngineMove(usiMove);
    });

    this.syncHands();
  }

  handlePlayerMove(guiMove: GuiMove) {
    const usi = guiMoveToUsi(guiMove);

    // Add move to tree
    this.tree.addMove(usi);

    // Rebuild position from tree path
    this.rebuildPosition();

    this.gui.applyMove(guiMove);
    this.syncHands();

    this.engine.setPosition(this.tree.getLineToCurrent());
    this.engine.go();
  }

  private applyEngineMove(usiMove: string) {
    this.tree.addMove(usiMove);
    this.rebuildPosition();

    const guiMove = usiToGuiMove(usiMove, this.position);
    this.gui.applyMove(guiMove);
    this.syncHands();
  }

  undo() {
    this.tree.undo();
    this.rebuildPosition();
  }

  redo(branchIndex: number = 0) {
    this.tree.redo(branchIndex);
    this.rebuildPosition();
  }

  private rebuildPosition() {
    this.position = new Position();
    for (const move of this.tree.getLineToCurrent()) {
      this.position.doMove(move);
    }

    this.gui.renderFull(this.position);
    this.syncHands();

    this.engine.setPosition(this.tree.getLineToCurrent());
  }

  private syncHands() {
    const blackHand = this.position.getHand(Player.Black);
    const whiteHand = this.position.getHand(Player.White);

    this.gui.updateHands({
      black: Object.fromEntries(blackHand),
      white: Object.fromEntries(whiteHand),
    });
  }

  reset() {
    this.tree = new GameTree();
    this.position = new Position();
    this.gui.renderFull(this.position);
    this.syncHands();
  }
}
```

✅ What This Adds
Variations: multiple children at any move

Undo/redo works per branch

Branch selection: pass an index to redo(branchIndex)

Reconstruction: any position can be rebuilt from its path

## 🖼 GUI Impact
You now need:

A move tree view (like ShogiGUI’s side panel) to pick variations.

Buttons:

Undo → controller.undo()

Redo (next variation) → controller.redo(idx)

Reset → controller.reset()

# How to export/import this game tree in a Shogi-friendly format (like KIF, CSA, or USI logs)

Shogi has a few common formats:

USI move list → simple array of USI moves (easy, but no comments/branches).

CSA → widely supported, line-based, supports variations.

KIF/KI2 → human-readable, used in Japanese GUIs (supports comments & branches).

JSON → easiest for your own app (preserve full tree, comments, branches).

Since you already have a GameTree structure, we can support both JSON (internal) and USI/CSA (interop).

## 🏯 JSON Export/Import (full tree, comments, branches)
Extend GameTree:

```ts
interface GameNode {
  move?: string;
  parent?: GameNode;
  children: GameNode[];
  comment?: string;
}

export class GameTree {
  root: GameNode;
  current: GameNode;

  constructor() {
    this.root = { children: [] };
    this.current = this.root;
  }

  // --- existing methods (addMove, undo, redo, etc.) ---

  exportJSON(): string {
    function serialize(node: GameNode): any {
      return {
        move: node.move,
        comment: node.comment,
        children: node.children.map(serialize),
      };
    }
    return JSON.stringify(serialize(this.root), null, 2);
  }

  importJSON(json: string) {
    const data = JSON.parse(json);

    function deserialize(data: any, parent?: GameNode): GameNode {
      const node: GameNode = {
        move: data.move,
        parent,
        comment: data.comment,
        children: [],
      };
      node.children = data.children.map((child: any) =>
        deserialize(child, node)
      );
      return node;
    }

    this.root = deserialize(data);
    this.current = this.root;
  }
}
```
Now you can:

```ts
const json = tree.exportJSON();
tree.importJSON(json);
```

## 🏯 USI Move List Export (linear only)
This is useful to set position for an engine:

```ts
export class GameTree {
  // ...

  exportUSI(): string {
    return this.getLineToCurrent().join(" ");
  }

  importUSI(moves: string[]) {
    this.root = { children: [] };
    this.current = this.root;
    for (const move of moves) {
      this.addMove(move);
    }
  }
}
```
Example:

```ts
tree.importUSI(["7g7f", "3c3d"]);
console.log(tree.exportUSI()); 
// "7g7f 3c3d"
```

## 🏯 CSA Export (variation-friendly, standard Shogi format)
CSA is a text format where each move is written as +7776FU or -3334FU.
USI moves like 7g7f can be converted with a simple mapper.

```ts
function usiToCsa(usi: string, player: "black" | "white"): string {
  // Example: 7g7f → +7776FU
  // Convert file/rank to numbers: a=1 … i=9
  const fileMap = { a: "1", b: "2", c: "3", d: "4", e: "5", f: "6", g: "7", h: "8", i: "9" };

  const fromFile = usi[0];
  const fromRank = usi[1];
  const toFile = usi[2];
  const toRank = usi[3];

  const prefix = player === "black" ? "+" : "-";
  // TODO: map piece properly (FU, KY, KE, GI, KI, KA, HI, OU, etc.)
  const piece = "FU"; 

  return `${prefix}${fromFile}${fromRank}${toFile}${toRank}${piece}`;
}
```

Then traversal of GameTree can output CSA with branches.
(Since CSA supports N+ and N- branching, this maps nicely!)

## ✅ Next Steps
For engine replay / setting position → use exportUSI().

For saving/loading in your app → use exportJSON() / importJSON().

For sharing with other Shogi GUIs → add a proper CSA export/import layer.

If you need KIF (Japanese standard) → that’s more complex (requires Kanji + move numbering), but doable once JSON tree is stable.

Write the CSA export/import functions that fully map USI notation (7g7f, P*5e, promotions) to proper CSA strings (+7776FU, +0055KA, +2728TO)?
