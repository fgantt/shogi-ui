#!/bin/bash

# Build script for Shogi WebAssembly engine

echo "Building Shogi WebAssembly engine..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Check if Python is available for opening book conversion
if ! command -v python3 &> /dev/null; then
    echo "Warning: Python3 not found. Opening book conversion will be skipped."
    echo "Install Python3 to enable automatic opening book binary generation."
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf pkg/
rm -rf target/

# Generate opening book binary from JSON (if Python is available)
if command -v python3 &> /dev/null; then
    echo "Converting opening book from JSON to binary format..."
    if [ -f "src/ai/openingBook.json" ]; then
        python3 scripts/convert_opening_book.py src/ai/openingBook.json dist/opening_book.bin
        if [ $? -eq 0 ]; then
            echo "Opening book binary generated successfully: dist/opening_book.bin"
        else
            echo "Warning: Failed to generate opening book binary. Continuing with build..."
        fi
    else
        echo "Warning: src/ai/openingBook.json not found. Skipping opening book conversion."
    fi
else
    echo "Skipping opening book conversion (Python3 not available)"
fi

# Build for web target
echo "Building for web target..."
wasm-pack build --target web --dev --out-dir pkg

# Build for bundler target (for use with webpack/vite)
echo "Building for bundler target..."
wasm-pack build --target bundler --dev --out-dir pkg-bundler

# Copy opening book binary to output directories (if it exists)
if [ -f "dist/opening_book.bin" ]; then
    echo "Copying opening book binary to output directories..."
    cp dist/opening_book.bin pkg/
    cp dist/opening_book.bin pkg-bundler/
    echo "Opening book binary copied to pkg/ and pkg-bundler/"
fi

echo "Build complete!"
echo "Web target: pkg/"
echo "Bundler target: pkg-bundler/"
if [ -f "dist/opening_book.bin" ]; then
    echo "Opening book binary: dist/opening_book.bin"
fi
