#!/bin/bash

# Build script for Shogi WebAssembly engine

echo "Building Shogi WebAssembly engine..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "Installing wasm-pack..."
    cargo install wasm-pack
fi

# Clean previous builds
echo "Cleaning previous builds..."
rm -rf pkg/
rm -rf target/

# Build for web target
echo "Building for web target..."
wasm-pack build --target web --out-dir pkg

# Build for bundler target (for use with webpack/vite)
echo "Building for bundler target..."
wasm-pack build --target bundler --out-dir pkg-bundler

echo "Build complete!"
echo "Web target: pkg/"
echo "Bundler target: pkg-bundler/"
