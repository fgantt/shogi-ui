# Evaluation Cache - WASM Compatibility Guide

## Overview

The evaluation cache is fully compatible with WebAssembly (WASM) targets, with optimizations for browser environments.

## WASM-Specific Features

### 1. Optimized Default Configuration

The cache automatically uses smaller defaults for WASM:

```rust
// Native: 1M entries (~32MB)
// WASM:   64K entries (~2MB)

let cache = EvaluationCache::new(); // Automatically WASM-optimized
```

### 2. WASM-Optimized Constructor

```rust
#[cfg(target_arch = "wasm32")]
let cache = EvaluationCache::new_wasm_optimized();
// Creates 32K entry cache (~1MB) with stats/verification disabled
```

### 3. Memory-Efficient Configuration

```rust
use shogi_vibe_usi::evaluation::eval_cache::*;

#[cfg(target_arch = "wasm32")]
fn create_wasm_cache() -> EvaluationCache {
    let config = EvaluationCacheConfig {
        size: 16384, // 16K entries (~512KB)
        replacement_policy: ReplacementPolicy::DepthPreferred,
        enable_statistics: false, // Reduce overhead
        enable_verification: false, // Reduce overhead
    };
    EvaluationCache::with_config(config)
}
```

## Browser Integration

### Example: Use in Web Worker

```javascript
// In JavaScript/TypeScript
import init, { SearchEngine } from './pkg/shogi_engine.js';

async function setupEngine() {
    await init();
    
    const engine = SearchEngine.new(null, 16);
    
    // Enable cache (automatically WASM-optimized)
    engine.enable_eval_cache();
    
    // Use normally
    const result = engine.search_at_depth(board, captured, player, 5, 5000, -10000, 10000);
}
```

### Memory Recommendations for WASM

**Mobile Browsers** (limited memory):
```rust
let config = EvaluationCacheConfig::with_size_mb(1); // 1MB
```

**Desktop Browsers** (more memory):
```rust
let config = EvaluationCacheConfig::with_size_mb(4); // 4MB
```

**High-End Systems**:
```rust
let config = EvaluationCacheConfig::with_size_mb(8); // 8MB
```

## WASM Binary Size Impact

### Estimated Binary Size Increase

- **Cache Code**: ~100KB (compressed)
- **With Dependencies**: ~120KB (compressed)
- **Impact**: Minimal (<5% of typical WASM binary)

### Optimization Tips

1. **Disable unused features** (done automatically)
2. **Use smaller default cache** (done automatically)
3. **Disable stats in production** (recommended)

```rust
#[cfg(target_arch = "wasm32")]
{
    cache.set_statistics_enabled(false);
    cache.set_verification_enabled(false);
}
```

## Platform-Specific Optimizations

### Conditional Compilation

The cache uses conditional compilation for WASM:

```rust
// Different defaults for WASM vs native
#[cfg(target_arch = "wasm32")]
const DEFAULT_CACHE_SIZE: usize = 64 * 1024; // 64K for WASM

#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_CACHE_SIZE: usize = 1024 * 1024; // 1M for native
```

### Memory-Conscious Features

**WASM builds automatically:**
- Use smaller default cache size
- Skip unnecessary allocations
- Optimize for browser memory constraints

## Testing in Browser

### Manual Testing

```javascript
// Test cache in browser console
const engine = new SearchEngine(null, 16);
engine.enable_eval_cache();

console.log("Cache enabled:", engine.is_eval_cache_enabled());

// Perform search
const result = engine.search_at_depth(board, captured, player, 4, 3000, -10000, 10000);
console.log("Search result:", result);

// Check statistics
const stats = engine.get_eval_cache_statistics();
console.log("Cache stats:", stats);
```

## Performance in WASM

### Expected Performance

**Native:**
- Cache probe: <50ns
- Full evaluation: ~1000-5000ns
- Speedup: 20-100x

**WASM:**
- Cache probe: ~100-200ns (2-4x slower than native)
- Full evaluation: ~3000-10000ns (2-3x slower than native)
- Speedup: 15-50x (still significant)

**Conclusion**: Cache still very beneficial in WASM!

## Browser-Specific Considerations

### 1. Memory Limits

Browsers typically limit WASM memory to 1-2GB:
- Use conservative cache sizes (1-8MB)
- Monitor memory usage
- Enable compaction

### 2. Thread Support

Cache is thread-safe but browser threading limited:
- Web Workers for background processing
- Cache works correctly in workers
- Statistics tracking safe across workers

### 3. Persistence

Use browser storage for cache persistence:

```javascript
// Save cache
const cacheData = await engine.export_cache_json();
localStorage.setItem('eval_cache', cacheData);

// Load cache
const savedCache = localStorage.getItem('eval_cache');
if (savedCache) {
    await engine.load_cache_json(savedCache);
}
```

## WASM-Specific Configuration

### Recommended WASM Config

```rust
#[cfg(target_arch = "wasm32")]
pub fn get_wasm_config() -> EvaluationCacheConfig {
    EvaluationCacheConfig {
        size: 32768, // 32K entries (~1MB)
        replacement_policy: ReplacementPolicy::DepthPreferred,
        enable_statistics: false, // Minimal overhead
        enable_verification: false, // Minimal overhead
    }
}
```

## Compatibility Status

✅ **Core Features**: Fully compatible
- Hash table implementation
- Zobrist hashing
- All replacement policies
- Entry management

✅ **Advanced Features**: Compatible
- Multi-level cache
- Prefetching
- Statistics tracking
- Configuration system

⚠️ **File I/O**: Not available in browser
- Use localStorage/IndexedDB instead
- Compression supported via JS libraries

✅ **Performance**: Optimized
- Smaller default sizes
- Conditional compilation
- Reduced overhead

## Testing WASM Builds

### Build for WASM

```bash
# Build WASM target
cargo build --target wasm32-unknown-unknown --release

# With wasm-pack
wasm-pack build --target web
```

### Test in Browser

```html
<!DOCTYPE html>
<html>
<head>
    <script type="module">
        import init, { SearchEngine } from './pkg/shogi_engine.js';
        
        async function test() {
            await init();
            
            const engine = new SearchEngine(null, 16);
            engine.enable_eval_cache();
            
            console.log("Cache enabled:", engine.is_eval_cache_enabled());
            
            // Test evaluation...
        }
        
        test();
    </script>
</head>
<body>
    <h1>WASM Cache Test</h1>
</body>
</html>
```

## Best Practices for WASM

### ✅ DO:
- Use smaller cache sizes (1-8MB)
- Disable stats in production
- Test memory usage in browser
- Use localStorage for persistence
- Monitor performance in browser

### ❌ DON'T:
- Use large caches (>16MB) in browser
- Leave stats enabled in production
- Assume native performance
- Ignore memory limits
- Use file I/O directly

## See Also

- `EVALUATION_CACHE_API.md` - Full API documentation
- `EVALUATION_CACHE_BEST_PRACTICES.md` - Best practices
- WASM integration tests in `benches/` directory
