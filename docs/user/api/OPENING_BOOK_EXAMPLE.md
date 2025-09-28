# Opening Book Format Example

This document demonstrates the new opening book format and shows how it addresses the issues with the current JSON-based implementation.

## Current JSON Format Issues

The existing `openingBook.json` has several problems:

```json
[
  {
    "name": "Aggressive Rook",
    "moves": {
      "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
        { "from": "27", "to": "26" }
      ]
    }
  }
]
```

**Problems:**
- String coordinates "27", "26" are unclear
- No move weights or evaluations
- Linear search through all openings
- No piece type information for drops
- Inefficient memory usage

## New Binary Format Structure

### Header (24 bytes)
```
Magic Number: "SBOB" (4 bytes)
Version: 1 (4 bytes)
Entry Count: 1000 (8 bytes)
Hash Table Size: 1024 (8 bytes)
```

### Hash Table Entry (16 bytes each)
```
Position Hash: 0x1234567890ABCDEF (8 bytes)
Entry Offset: 0x00001000 (8 bytes)
```

### Position Entry
```
FEN Length: 45 (4 bytes)
FEN: "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1" (45 bytes)
Move Count: 3 (4 bytes)
Moves: [Move Entry 1, Move Entry 2, Move Entry 3] (3 × 16 bytes)
```

### Move Entry (16 bytes)
```
From Position: 0x0102 (2 bytes) - row=1, col=2
To Position: 0x0101 (2 bytes) - row=1, col=1
Piece Type: 6 (1 byte) - Rook
Is Drop: 0 (1 byte) - false
Is Promotion: 0 (1 byte) - false
Weight: 850 (4 bytes) - high frequency move
Evaluation: 15 (4 bytes) - +0.15 pawns
Reserved: 0 (1 byte) - padding
```

## New JSON Format (for Development)

For easier development and debugging, we can also support a new JSON format:

```json
{
  "version": 1,
  "positions": {
    "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": {
      "moves": [
        {
          "from": [1, 2],
          "to": [1, 1],
          "piece_type": "Rook",
          "is_drop": false,
          "is_promotion": false,
          "weight": 850,
          "evaluation": 15,
          "opening_name": "Aggressive Rook",
          "move_notation": "2b-2a"
        },
        {
          "from": [1, 2],
          "to": [1, 0],
          "piece_type": "Rook",
          "is_drop": false,
          "is_promotion": false,
          "weight": 120,
          "evaluation": 8,
          "opening_name": "Aggressive Rook",
          "move_notation": "2b-2c"
        },
        {
          "from": null,
          "to": [2, 1],
          "piece_type": "Pawn",
          "is_drop": true,
          "is_promotion": false,
          "weight": 30,
          "evaluation": -5,
          "opening_name": "Aggressive Rook",
          "move_notation": "P*2b"
        }
      ]
    }
  }
}
```

## Comparison: Old vs New

| Aspect | Old JSON Format | New Binary Format |
|--------|----------------|-------------------|
| **Lookup Time** | O(n) linear search | O(1) hash lookup |
| **Memory Usage** | ~50KB for 100 positions | ~15KB for 100 positions |
| **Move Data** | Incomplete | Complete with weights/evaluations |
| **Coordinates** | String "27" | Clear [row, col] arrays |
| **Extensibility** | Hard to add metadata | Easy to add new fields |
| **Performance** | Slow parsing | Fast binary parsing |
| **WASM Size** | Large due to JSON overhead | Compact binary format |

## Migration Example

Converting from old format to new:

**Old Format:**
```json
{ "from": "27", "to": "26" }
```

**New Format:**
```json
{
  "from": [1, 2],  // 27 = 1*9 + 2 = row 1, col 2
  "to": [1, 1],    // 26 = 1*9 + 1 = row 1, col 1
  "piece_type": "Rook",
  "is_drop": false,
  "is_promotion": false,
  "weight": 850,
  "evaluation": 15
}
```

## Performance Benefits

1. **Faster Lookup**: Hash table lookup vs linear search
2. **Smaller Memory**: Binary format is ~70% smaller than JSON
3. **Better Caching**: Position data is more cache-friendly
4. **WASM Optimized**: Minimal allocations and efficient data structures
5. **Extensible**: Easy to add new move properties without breaking compatibility

## Implementation Priority

1. **Phase 1**: Implement new data structures and JSON parser
2. **Phase 2**: Create binary format and conversion tools
3. **Phase 3**: Migrate existing data and optimize for WASM
4. **Phase 4**: Add advanced features (learning, adaptation, etc.)





