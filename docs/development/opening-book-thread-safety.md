# Opening Book Thread Safety

## Overview

The `OpeningBook` struct is **not thread-safe** by design. It is intended for single-threaded access only.

## Thread Safety Guarantees

### Current Implementation

- **Single-threaded access only**: `OpeningBook` does not implement `Send` or `Sync` traits
- **No internal synchronization**: The struct uses `HashMap` and `LruCache` which are not thread-safe
- **Mutable state**: Methods like `get_moves()`, `get_best_move()`, and `load_lazy_position()` modify internal state (cache, lazy loading)

### Why Single-Threaded?

1. **Performance**: Avoiding synchronization overhead allows for maximum lookup performance
2. **Simplicity**: Single-threaded design is simpler and easier to reason about
3. **Use case**: Opening book lookups are typically fast enough that they don't require parallel access

## Thread-Safe Wrapper

If you need thread-safe access to the opening book, you can use `ThreadSafeOpeningBook` which wraps `OpeningBook` with a `Mutex`:

```rust
use shogi_engine::opening_book::ThreadSafeOpeningBook;

let book = ThreadSafeOpeningBook::new(opening_book);
// Now safe to share across threads
```

## Best Practices

1. **Single-threaded access**: Use `OpeningBook` directly in single-threaded contexts
2. **Thread-safe wrapper**: Use `ThreadSafeOpeningBook` if you need to share across threads
3. **Read-only access**: If you only need read access, consider cloning the book or using immutable methods
4. **Avoid concurrent modification**: Never modify the book from multiple threads simultaneously

## Implementation Details

The `OpeningBook` struct contains:
- `HashMap<u64, PositionEntry>` - Not thread-safe
- `HashMap<u64, LazyPositionEntry>` - Not thread-safe
- `LruCache<u64, PositionEntry>` - Not thread-safe
- Mutable state for caching and lazy loading

All of these require exclusive access for safe modification.

