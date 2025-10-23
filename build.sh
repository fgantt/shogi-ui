#!/bin/bash

# Build script for Shogi Tauri application

echo "Building Shogi Tauri application..."

# Check if Python is available for opening book conversion
if ! command -v python3 &> /dev/null; then
    echo "Warning: Python3 not found. Opening book conversion will be skipped."
    echo "Install Python3 to enable automatic opening book binary generation."
fi

# Clean previous builds
echo "Cleaning previous builds..."
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

# Build the Rust engine
echo "Building Rust engine..."
cargo build --bin usi-engine --release

if [ $? -eq 0 ]; then
    echo "Rust engine built successfully!"
else
    echo "Error: Failed to build Rust engine"
    exit 1
fi

# Build the Tauri application
echo "Building Tauri application..."
npm run tauri:build

if [ $? -eq 0 ]; then
    echo "Tauri application built successfully!"
    echo "Build artifacts are available in src-tauri/target/release/"
else
    echo "Error: Failed to build Tauri application"
    exit 1
fi

echo "Build complete!"
echo "Tauri application: src-tauri/target/release/"
if [ -f "dist/opening_book.bin" ]; then
    echo "Opening book binary: dist/opening_book.bin"
fi