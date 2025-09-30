# Tasks: Opening Book Implementation

## Relevant Files

- `src/opening_book.rs` - Current opening book implementation that needs to be completely rewritten
- `src/opening_book.rs` - Unit tests for the new opening book implementation
- `src/ai/openingBook.json` - Current JSON format that needs to be migrated
- `src/lib.rs` - Main engine file that integrates opening book with search
- `src/types.rs` - Core data structures that may need updates for new opening book
- `src/bitboards.rs` - Board representation used for FEN generation
- `tests/opening_book_tests.rs` - Comprehensive test suite for opening book functionality
- `tests/integration_opening_book_tests.rs` - Integration tests with search engine
- `tests/performance_opening_book_tests.rs` - Performance benchmarks for opening book
- `scripts/convert_opening_book.py` - Python script to convert JSON to binary format
- `scripts/generate_opening_book.py` - Python script to generate opening book from game databases
- `Cargo.toml` - Rust dependencies (may need updates for binary format support)
- `build.sh` - Build script that may need updates for opening book compilation

### Notes

- Unit tests should be placed alongside the code files they are testing (e.g., `opening_book.rs` and `opening_book_tests.rs` in the same directory).
- Use `cargo test [optional/path/to/test/file]` to run tests. Running without a path executes all tests found by the Cargo configuration.
- The current implementation uses `include_str!` to embed JSON data, which will be replaced with binary data loading.

## Tasks

- [x] 1.0 Design and Implement New Opening Book Data Structures
  - [x] 1.1 Create `BookMove` struct with enhanced fields (weight, evaluation, is_drop, etc.)
  - [x] 1.2 Create `PositionEntry` struct to hold FEN string and associated moves
  - [x] 1.3 Create new `OpeningBook` struct with HashMap-based lookup instead of Vec
  - [x] 1.4 Add proper error handling types for opening book operations
  - [x] 1.5 Implement `Serialize` and `Deserialize` traits for new data structures
  - [x] 1.6 Add helper methods for coordinate conversion (string to Position)
  - [x] 1.7 Create builder pattern for constructing opening book entries

- [x] 2.0 Create Binary Format Parser and Generator
  - [x] 2.1 Design binary format specification with header, hash table, and position entries
  - [x] 2.2 Implement binary format writer that converts data structures to bytes
  - [x] 2.3 Implement binary format reader that parses bytes back to data structures
  - [x] 2.4 Add magic number validation and version checking
  - [x] 2.5 Implement hash table generation for O(1) position lookup
  - [x] 2.6 Add compression support for similar positions
  - [x] 2.7 Create binary format validation and integrity checking

- [x] 3.0 Implement JSON to Binary Migration Tools
  - [x] 3.1 Create Python script to parse existing `openingBook.json` format
  - [x] 3.2 Implement coordinate conversion from string format ("27") to [row, col]
  - [x] 3.3 Add move weight assignment based on frequency analysis
  - [x] 3.4 Generate position evaluations using engine analysis
  - [x] 3.5 Create binary format output from converted JSON data
  - [x] 3.6 Add validation to ensure all moves are properly converted
  - [x] 3.7 Create migration report showing conversion statistics

- [x] 4.0 Rewrite Opening Book Core Logic
  - [x] 4.1 Replace linear search with HashMap-based O(1) lookup
  - [x] 4.2 Implement `load_from_binary()` method for binary format loading
  - [x] 4.3 Implement `load_from_json()` method for backward compatibility
  - [x] 4.4 Add `get_best_move()` with weight-based move selection
  - [x] 4.5 Add `get_random_move()` with weighted random selection
  - [x] 4.6 Implement `get_moves()` to return all available moves for a position
  - [x] 4.7 Add move conversion utilities from book format to engine format
  - [x] 4.8 Implement FEN hashing for efficient position lookup

- [x] 5.0 Integrate New Opening Book with Engine
  - [x] 5.1 Update `ShogiEngine::new()` to use new opening book constructor
  - [x] 5.2 Modify `get_best_move()` to use new opening book API
  - [x] 5.3 Add proper move property determination (is_capture, gives_check)
  - [x] 5.4 Update player assignment for book moves
  - [x] 5.5 Add opening book move logging and debugging
  - [x] 5.6 Implement fallback to search when no book move available
  - [x] 5.7 Add opening book statistics and performance monitoring

- [x] 6.0 Create Comprehensive Test Suite
  - [x] 6.1 Create unit tests for `BookMove` struct creation and validation
  - [x] 6.2 Create unit tests for `PositionEntry` operations
  - [x] 6.3 Create unit tests for `OpeningBook` lookup functionality
  - [x] 6.4 Create unit tests for binary format serialization/deserialization
  - [x] 6.5 Create unit tests for coordinate conversion utilities
  - [x] 6.6 Create integration tests with `ShogiEngine` and search functionality
  - [x] 6.7 Create performance tests comparing old vs new implementation
  - [x] 6.8 Create tests for edge cases (empty book, invalid positions, etc.)

- [x] 7.0 Optimize for WASM Performance
  - [x] 7.1 Replace `Vec<u8>` with `Box<[u8]>` for binary data storage
  - [x] 7.2 Implement lazy loading for rarely accessed positions
  - [x] 7.3 Add position caching for frequently accessed FENs
  - [x] 7.4 Optimize hash function for WASM environment
  - [x] 7.5 Minimize heap allocations in hot paths
  - [x] 7.6 Implement streaming for large opening books
  - [x] 7.7 Add memory usage monitoring and optimization

- [x] 8.0 Update Build System and Documentation
  - [x] 8.1 Update `Cargo.toml` with any new dependencies for binary format
  - [x] 8.2 Modify `build.sh` to include opening book binary generation
  - [x] 8.3 Update `README.md` with new opening book features
  - [x] 8.4 Create migration guide for updating existing opening books
  - [x] 8.5 Add performance benchmarks to documentation
  - [x] 8.6 Update API documentation for new opening book methods
  - [x] 8.7 Create example usage documentation for developers
Mar