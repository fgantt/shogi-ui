#!/bin/bash

# Script to update version numbers across all configuration files
# Usage: ./scripts/update-version.sh 1.0.0

set -e

if [ -z "$1" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.0"
    exit 1
fi

NEW_VERSION=$1

# Validate semantic versioning format
if ! [[ $NEW_VERSION =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "Error: Version must be in semantic versioning format (e.g., 1.0.0)"
    exit 1
fi

echo "Updating version to $NEW_VERSION..."

# Update package.json
if [ -f "package.json" ]; then
    echo "Updating package.json..."
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" package.json
    rm package.json.bak
fi

# Update src-tauri/tauri.conf.json
if [ -f "src-tauri/tauri.conf.json" ]; then
    echo "Updating src-tauri/tauri.conf.json..."
    sed -i.bak "s/\"version\": \".*\"/\"version\": \"$NEW_VERSION\"/" src-tauri/tauri.conf.json
    rm src-tauri/tauri.conf.json.bak
fi

# Update src-tauri/Cargo.toml
if [ -f "src-tauri/Cargo.toml" ]; then
    echo "Updating src-tauri/Cargo.toml..."
    sed -i.bak "0,/version = \".*\"/s//version = \"$NEW_VERSION\"/" src-tauri/Cargo.toml
    rm src-tauri/Cargo.toml.bak
fi

# Update root Cargo.toml
if [ -f "Cargo.toml" ]; then
    echo "Updating Cargo.toml..."
    sed -i.bak "0,/version = \".*\"/s//version = \"$NEW_VERSION\"/" Cargo.toml
    rm Cargo.toml.bak
fi

echo "✓ Version updated to $NEW_VERSION in all files!"
echo ""
echo "Next steps:"
echo "1. Review changes: git diff"
echo "2. Update CHANGELOG.md with release notes"
echo "3. Test the build: npm run tauri:build"
echo "4. Commit: git add . && git commit -m \"Release v$NEW_VERSION\""
echo "5. Tag: git tag -a v$NEW_VERSION -m \"Release v$NEW_VERSION\""
echo "6. Push: git push origin main && git push origin v$NEW_VERSION"

