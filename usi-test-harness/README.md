# USI Test Harness

This is a simple command-line test harness for validating the implementation of a Shogi engine that conforms to the Universal Shogi Interface (USI) protocol.

## Purpose

The primary purpose of this tool is to perform a basic integration test on a USI-compliant Shogi engine. It does this by:

1.  Starting the engine as a child process.
2.  Communicating with the engine over standard input and output.
3.  Playing a game of Shogi by sending `position` and `go` commands.
4.  Receiving the `bestmove` from the engine.
5.  Validating the move against its own internal board representation.
6.  Updating its internal state and continuing the game loop.

This allows for catching basic errors in move generation, move validation, and state tracking within the engine or the test harness itself.

## Compilation

This is a standard Rust project using Cargo. To compile it, navigate to this directory (`usi-test-harness`) in your terminal and run:

```bash
cargo build
```

For a release build, which is optimized and runs faster, use:

```bash
cargo build --release
```

The compiled executable will be located in the `target/debug/` or `target/release/` directory.

## Usage

To run the test harness, you need to provide the path to the USI engine executable as a command-line argument.

### Example

If you have a compiled engine at `../target/release/shogi_engine`, you would run the test harness like this:

From the `usi-test-harness` directory:

**For a debug build:**
```bash
./target/debug/usi-test-harness ../target/release/shogi_engine
```

**For a release build:**
```bash
./target/release/usi-test-harness ../target/release/shogi_engine
```

The test harness will then start playing a game and printing the communication with the engine to the console.
