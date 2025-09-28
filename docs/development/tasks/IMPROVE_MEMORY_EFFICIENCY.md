# Implementation Plan: Memory and Cache Efficiency

## 1. Objective

To analyze and optimize the engine's core data structures to improve CPU cache utilization and reduce memory bandwidth. This optimization aims to increase the engine's search speed (measured in nodes per second) by minimizing the time the CPU spends waiting for data from main memory.

## 2. Background

A modern CPU is significantly faster than main memory (RAM). To bridge this gap, CPUs use several layers of small, fast cache memory (L1, L2, L3). When the CPU needs data, it first checks the cache. A "cache hit" is fast. A "cache miss" forces a slow trip to RAM, stalling the CPU.

In a search-intensive program like a shogi engine, the transposition table is accessed millions of times. If its entries are large and poorly aligned, the cache miss rate will be high, and performance will be memory-bound. By making our data structures smaller and aligning them with cache lines, we can dramatically improve performance.

## 3. Core Logic and Implementation Plan

This plan focuses on the most frequently accessed data structures: the `TranspositionEntry` and the `Move` list.

### Step 1: Profile and Identify Bottlenecks

Before optimizing, it's crucial to measure. Use a profiling tool (like `perf` on Linux, Instruments on macOS, or browser-based WASM profilers) to analyze the current performance. Pay close attention to metrics related to cache misses and memory access patterns during the search.

### Step 2: Optimize the `TranspositionEntry` Struct

The transposition table entry is the most critical structure to optimize.

**Goal:** Reduce the size of `TranspositionEntry` to a power of two (e.g., 16 bytes) to ensure dense packing and efficient cache alignment.

**File:** `src/search.rs`

**Current (Conceptual) Struct:**
```rust
struct TranspositionEntry {
    fen_key: String, // Very large and slow!
    depth: u8,
    score: i32,
    flag: TranspositionFlag,
    best_move: Option<Move>,
}
```

**Proposed Optimized Struct:**
We will replace the FEN key with a `u64` Zobrist hash and pack the remaining data tightly.

```rust
// A compact representation of a move (16 bits)
// 6 bits for `from` square (0-80)
// 6 bits for `to` square (0-80)
// 4 bits for promotion info, etc.
struct CompactMove(u16);

// The new, smaller TT entry
#[repr(C)] // Ensure defined layout
struct TranspositionEntry {
    zobrist_hash: u64, // 8 bytes, for hash collision checks
    packed_data: u64,  // 8 bytes for score, depth, flag, and move
}

impl TranspositionEntry {
    // Example of packing score, depth, flag, and move into a single u64
    fn pack(score: i32, depth: u8, flag: TranspositionFlag, best_move: CompactMove) -> u64 {
        // ... bitwise operations to pack all data ...
    }

    fn unpack_score(&self) -> i32 { /* ... */ }
    fn unpack_depth(&self) -> u8 { /* ... */ }
    // ... other unpackers
}
```

**Actions:**
1.  Implement Zobrist hashing for the board state if not already present. This replaces the slow `String` FEN key.
2.  Refactor `TranspositionEntry` to use bit-packing. Store the score, depth, entry flag (Exact, LowerBound, UpperBound), and a compacted best move into one or two `u64` fields.
3.  A `Move` object is large. Create a `CompactMove` representation (likely a `u16` or `u32`) that stores only the `from` and `to` squares and promotion info. Write functions to convert between `Move` and `CompactMove`.

### Step 3: Optimize Move List Memory Allocation

Generating a `Vec<Move>` at every node in the search tree puts pressure on the memory allocator.

**Goal:** Avoid heap allocations within the search loop.

**File:** `src/search.rs`

**Actions:**
1.  Create a `MoveList` struct that contains a fixed-size array of moves (e.g., `[Move; 256]`, as there can be no more than ~256 legal moves in a shogi position).
2.  Instead of `generate_legal_moves` returning a `Vec<Move>`, modify it to take a mutable reference to a `MoveList` and fill it in place: `generate_legal_moves(&self, ..., move_list: &mut MoveList)`.
3.  In the `negamax` function, declare a single `MoveList` on the stack at the beginning of the function and pass a mutable reference to it down the call stack. This completely avoids heap allocations for move lists during the search.

## 4. Dependencies and Considerations

*   **Complexity:** Bit-packing and memory management optimizations can make the code harder to read and debug. It is essential to add clear comments and unit tests for the packing/unpacking logic.
*   **Profiling is Key:** Do not optimize prematurely. Use profiling data to guide your efforts to the most significant bottlenecks.
*   **Zobrist Hashing:** A correct and efficient Zobrist hash implementation is a prerequisite for optimizing the transposition table key.
*   **Correctness:** After implementing these changes, a full suite of regression tests is critical to ensure that the packing/unpacking logic and other changes have not introduced subtle bugs into the search or evaluation.

## 5. Verification Plan

1.  **Benchmarking (Primary):** The definitive test for this optimization is performance. Measure the **Nodes Per Second (NPS)** on a large, standardized test suite of positions before and after the changes. A successful optimization should yield a clear and significant NPS improvement (a 10-30% increase is a reasonable target).
2.  **Profiler Analysis:** Use a profiler again after the changes are implemented. The analysis should confirm that the cache miss rate has decreased and that time spent in memory allocation functions has been significantly reduced.
3.  **Regression Testing:** Run all existing unit and integration tests to ensure that the engine still functions correctly and produces the same results for tactical puzzles and specific test positions.

