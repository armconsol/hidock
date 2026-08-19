#!/bin/bash
# Local build script for HiNotes Desktop
# Usage: ./build_local.sh [macos|linux|windows]

set -e

PLATFORM="${1:-$(uname -s | tr '[:upper:]' '[:lower:]')}"
VERSION=$(node -p "require('./package.json').version")

echo "========================================="
echo "Building HiNotes Desktop v${VERSION}"
echo "Platform: ${PLATFORM}"
echo "========================================="
echo ""

# Install dependencies
echo "📦 Installing dependencies..."
npm ci

# Build frontend
echo "🎨 Building frontend..."
npm run build

# Build based on platform
case "${PLATFORM}" in
  darwin|macos)
    echo "🍎 Building macOS Universal Binary..."
    npm run tauri build -- --target universal-apple-darwin
    
    echo ""
    echo "✅ Build complete!"
    echo "Output: src-tauri/target/universal-apple-darwin/release/bundle/dmg/"
    ;;
    
  linux)
    echo "🐧 Building Linux packages..."
    npm run tauri build
    
    echo ""
    echo "✅ Build complete!"
    echo "AppImage: src-tauri/target/release/bundle/appimage/"
    echo "deb: src-tauri/target/release/bundle/deb/"
    echo "rpm: src-tauri/target/release/bundle/rpm/"
    ;;
    
  windows|win32)
    echo "🪟 Building Windows installer..."
    npm run tauri build
    
    echo ""
    echo "✅ Build complete!"
    echo "MSI: src-tauri/target/release/bundle/msi/"
    ;;
    
  *)
    echo "❌ Unknown platform: ${PLATFORM}"
    echo "Usage: $0 [macos|linux|windows]"
    exit 1
    ;;
esac

echo ""
echo "========================================="
echo "Build artifacts created for ${PLATFORM}"
echo "Version: ${VERSION}"
echo "========================================="
