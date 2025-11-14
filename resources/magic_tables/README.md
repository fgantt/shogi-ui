# Magic Bitboard Tables

This directory contains precomputed magic bitboard tables for fast initialization.

## Overview

Magic bitboard tables are precomputed lookup tables that enable efficient sliding piece move generation. Generating these tables from scratch takes 60+ seconds, but loading from a precomputed file takes less than 1 second.

## File Format

- **File:** `magic_table.bin`
- **Format:** Binary format with version header and checksum
- **Version:** 1
- **Magic Number:** `SHOGI_MAGIC_V1`

## Generation

To generate a precomputed magic table:

```bash
cargo run --bin generate_magic_tables
```

This will generate the table and save it to `resources/magic_tables/magic_table.bin` by default.

To specify a custom output path:

```bash
cargo run --bin generate_magic_tables -- --output /path/to/magic_table.bin
```

## Loading

The engine automatically attempts to load the precomputed table at startup:

1. Checks environment variable `SHOGI_MAGIC_TABLE_PATH` for custom path
2. Falls back to `resources/magic_tables/magic_table.bin` relative to executable or workspace root
3. If file not found or invalid, generates a new table (and optionally saves it)

## Configuration

### Environment Variable

Set `SHOGI_MAGIC_TABLE_PATH` to specify a custom path:

```bash
export SHOGI_MAGIC_TABLE_PATH=/custom/path/magic_table.bin
```

### Programmatic

The default path can be obtained via:

```rust
use shogi_engine::bitboards::magic::magic_table::get_default_magic_table_path;
let path = get_default_magic_table_path();
```

## Build Integration

To include precomputed tables in your build:

1. Generate the table during build (add to build script or CI)
2. Include `resources/magic_tables/magic_table.bin` in your distribution
3. Ensure the file is accessible at runtime relative to the executable

## File Format Details

The file format includes:

- **Header (17 bytes):**
  - Magic number: 16 bytes (`SHOGI_MAGIC_V1`)
  - Version: 1 byte (currently 1)
- **Data:**
  - 81 rook magic entries (8 + 16 + 1 + 8 + 8 bytes each)
  - 81 bishop magic entries (8 + 16 + 1 + 8 + 8 bytes each)
  - Attack storage: 4-byte length + variable-length bitboard array
- **Checksum (8 bytes):**
  - 64-bit checksum for integrity verification

## Performance

- **Generation:** ~60+ seconds (one-time cost)
- **Loading:** <1 second (typical)
- **Speedup:** 60x+ faster initialization

## Validation

The file format includes:
- Magic number validation (ensures correct file type)
- Version checking (ensures compatibility)
- Checksum verification (detects corruption)

If validation fails, the engine will generate a new table automatically.

## Troubleshooting

**Table not loading:**
- Check file exists at expected path
- Verify file permissions
- Check file is not corrupted (checksum validation will fail)

**Slow startup:**
- Ensure precomputed table is included in distribution
- Check file is accessible at runtime
- Verify `SHOGI_MAGIC_TABLE_PATH` is set correctly if using custom path

**Generation fails:**
- Check available memory (generation requires ~50MB)
- Verify write permissions for output directory
- Check disk space

