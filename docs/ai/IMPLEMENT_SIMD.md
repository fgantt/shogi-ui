# Implementation Plan: SIMD Optimization

## 1. Objective

To leverage SIMD (Single Instruction, Multiple Data) instructions to accelerate bitboard operations, particularly for move generation and evaluation. This will increase the raw calculation speed of the engine, allowing it to process more nodes per second.

## 2. Background

SIMD is a CPU feature that allows a single instruction to perform the same operation on multiple pieces of data simultaneously. A shogi bitboard is 81 bits, which fits into a `u128`. This `u128` can be treated as a vector of two `u64`s. With SIMD, a bitwise operation (like AND, OR, or a shift) can be performed on both 64-bit halves of the bitboard at the same time, effectively doubling the speed of that operation.

This is especially powerful for generating attack maps for sliding pieces (rooks, bishops, lances), which involves numerous bitwise operations. For our target, WebAssembly, the `simd128` feature is widely supported in modern browsers and provides a `v128` data type that maps directly to these hardware capabilities.

## 3. Core Logic and Implementation Plan

This plan involves refactoring the core `Bitboard` type and its associated operations to use SIMD intrinsics.

### Step 1: Enable SIMD in the Build Configuration

The Rust compiler needs to be told to target CPUs with SIMD capabilities.

**File:** `.cargo/config.toml` (create if it doesn't exist)

```toml
[build]
rustflags = ["-C", "target-feature=+simd128"]
```

This flag enables the `simd128` feature for the WebAssembly target, making the `v128` type and its intrinsics available.

### Step 2: Refactor the `Bitboard` Type

Change the underlying representation of the `Bitboard` to use the native 128-bit SIMD vector type.

**File:** `src/bitboards.rs`

```rust
// Import the necessary SIMD modules for wasm32 architecture
#[cfg(target_arch = "wasm32")]
use std::arch::wasm32::*;

// Old representation:
// pub type Bitboard = u128;

// New SIMD representation:
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct Bitboard(pub v128);

// Implement basic bitwise operations using SIMD intrinsics
impl std::ops::BitAnd for Bitboard {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Bitboard(u8x16_bitmask(self.0) & u8x16_bitmask(rhs.0))
    }
}

impl std::ops::BitOr for Bitboard {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Bitboard(u8x16_bitmask(self.0) | u8x16_bitmask(rhs.0))
    }
}

// ... implement other operators like BitXor, Not, Shl, Shr ...
```
*(Note: The exact intrinsic functions, like `u8x16_bitmask`, may vary. The key is to replace the standard scalar operators with their `std::arch::wasm32` SIMD equivalents.)*

### Step 3: Update Bitboard Manipulation Functions

Go through functions that perform bitboard logic, especially in `src/moves.rs` and `src/bitboards.rs`, and ensure they work correctly with the new `Bitboard` struct. The goal is to have the compiler use SIMD instructions wherever bitwise operations are performed.

**Example: Sliding Piece Attack Generation**

The functions that generate attack sets for rooks and bishops involve loops with multiple shifts and ORs. These are prime candidates for manual optimization using SIMD intrinsics if the compiler's auto-vectorization is not sufficient.

```rust
// In MoveGenerator
fn get_rook_attacks(&self, pos: Position, occupied: Bitboard) -> Bitboard {
    // The logic here involves ray casting in four directions.
    // Each direction's calculation can be a series of bitwise operations
    // that will now leverage the overloaded SIMD operators on the Bitboard struct.
    // ...
}
```

## 4. Dependencies and Considerations

*   **Target Architecture:** This plan is specific to the `wasm32` target. If the engine were to be compiled for desktop use, different SIMD intrinsics (`std::arch::x86_64` for SSE/AVX) would be needed, requiring conditional compilation (`#[cfg(...)]`).
*   **Compiler Auto-Vectorization:** Modern Rust compilers are very good at auto-vectorizing code. Before manually refactoring complex algorithms, it is worth inspecting the compiler output (e.g., the generated WASM or assembly) to see if loops are already being vectorized. Manual implementation is often only needed for complex patterns the compiler can't recognize.
*   **Code Readability:** Using SIMD intrinsics directly can make code less readable. It's best to wrap them in well-named functions or overload operators, as shown with the `Bitboard` struct.

## 5. Verification Plan

1.  **Correctness Testing (Crucial):** SIMD logic is prone to subtle bugs. Create a comprehensive test suite that compares the result of every modified bitboard operation against the result of the original, scalar (`u128`) implementation. The two results must be bit-for-bit identical for a wide range of inputs.
2.  **Benchmarking:** This is a pure performance optimization. The primary verification is to benchmark the most bitboard-heavy functions, such as the attack generation for sliding pieces. These functions should show a significant speedup. The overall Nodes Per Second (NPS) of the engine should also increase.
3.  **Profiler Analysis:** Use a profiler to compare the performance before and after. The time spent in the optimized functions should be noticeably lower.
4.  **Browser Compatibility:** While `simd128` is widely supported, ensure the application still runs correctly in all target browsers. Some older browser versions might not have support.

