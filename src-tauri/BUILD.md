# Build Configuration Guide

This document describes the Tauri build configuration for HiNotes Desktop.

## Overview

HiNotes Desktop uses Tauri v2 to create cross-platform desktop applications with native system integration for USB device access, file system operations, and network communication.

## Bundle Targets

### macOS
- **DMG** (`.dmg`) - Disk image installer
- **App Bundle** (`.app`) - Application bundle
- **Architecture**: Universal binary (x86_64 + aarch64)
- **Minimum OS**: macOS 10.13 (High Sierra)

### Linux
- **Debian Package** (`.deb`) - For Debian/Ubuntu-based distributions
- **AppImage** (`.AppImage`) - Portable, distribution-agnostic format
- **RPM** (`.rpm`) - For Red Hat/Fedora-based distributions

### Windows
- **MSI Installer** (`.msi`) - Windows Installer package
- **WebView**: Automatically downloads and installs Edge WebView2 if not present

## App Metadata

```json
{
  "productName": "HiNotes Desktop",
  "version": "Read from package.json",
  "identifier": "com.sarman.hinotes-desktop",
  "category": "Productivity",
  "description": "Cross-platform desktop app for HiNotes transcription device",
  "author": "Shaun Arman",
  "license": "See LICENSE file"
}
```

## Capabilities and Permissions

### Core Permissions
- Window management (close, minimize, maximize, show, hide, set title)
- Event system (listen, emit)
- Path access (app directories, user directories)

### File System Access
The app has scoped access to:
- `$APPDATA` - Application data directory
- `$APPLOCALDATA` - Local application data
- `$APPCONFIG` - Configuration directory
- `$APPLOG` - Log files
- `$APPCACHE` - Cache directory
- `$AUDIO` - User's audio directory (for recordings)
- `$DOCUMENT` - User's documents directory
- `$DOWNLOAD` - User's downloads directory
- `$TEMP` - Temporary files

### Network Access
- **API Endpoint**: `https://hinotes.hidock.com`
- Content Security Policy allows secure connections to HiNotes API
- WebSocket support for real-time sync

### USB Device Access
- Uses `rusb` library for USB communication with HiDoc P1 device
- Requires system-level USB permissions (handled by OS)

### Notifications
- Native system notifications for sync status, device events, and updates

## Auto-Updater Configuration

The app includes Tauri's built-in updater plugin for automatic updates.

### Setup

1. **Generate Update Keys**:
   ```bash
   npm run tauri signer generate -- -w ~/.tauri/myapp.key
   ```

2. **Update Configuration**:
   - Replace `YOUR_PUBLIC_KEY_HERE` in `tauri.conf.json` with your public key
   - Update `endpoints` URL to your GitHub releases or custom update server

3. **Release Process**:
   ```bash
   # Build with updater artifacts
   npm run tauri build
   
   # Upload to GitHub releases:
   # - .tar.gz (macOS)
   # - .AppImage.tar.gz (Linux)
   # - .msi.zip (Windows)
   # - latest.json (version manifest)
   ```

### Update Endpoint Format

The `latest.json` file should contain:

```json
{
  "version": "1.0.0-alpha",
  "notes": "Release notes here",
  "pub_date": "2025-01-15T12:00:00Z",
  "platforms": {
    "darwin-x86_64": {
      "signature": "...",
      "url": "https://github.com/yourusername/hidoc/releases/download/v1.0.0-alpha/HiNotes-Desktop_1.0.0-alpha_x64.app.tar.gz"
    },
    "darwin-aarch64": {
      "signature": "...",
      "url": "https://github.com/yourusername/hidoc/releases/download/v1.0.0-alpha/HiNotes-Desktop_1.0.0-alpha_aarch64.app.tar.gz"
    },
    "linux-x86_64": {
      "signature": "...",
      "url": "https://github.com/yourusername/hidoc/releases/download/v1.0.0-alpha/hinotes-desktop_1.0.0-alpha_amd64.AppImage.tar.gz"
    },
    "windows-x86_64": {
      "signature": "...",
      "url": "https://github.com/yourusername/hidoc/releases/download/v1.0.0-alpha/HiNotes-Desktop_1.0.0-alpha_x64-setup.msi.zip"
    }
  }
}
```

## Building

### Development
```bash
npm run tauri dev
```

### Production Build
```bash
npm run tauri build
```

Outputs are in `src-tauri/target/release/bundle/`:
- `dmg/` - macOS disk images
- `macos/` - macOS app bundles
- `deb/` - Debian packages
- `appimage/` - AppImages
- `rpm/` - RPM packages
- `msi/` - Windows installers

### Platform-Specific Builds

```bash
# Build only for current platform
npm run tauri build

# Build universal macOS binary (requires macOS with Xcode)
npm run tauri build -- --target universal-apple-darwin

# Build for specific Linux distributions
npm run tauri build -- --bundles deb
npm run tauri build -- --bundles appimage
npm run tauri build -- --bundles rpm
```

## Code Signing

### macOS
1. Obtain Apple Developer certificate
2. Update `tauri.conf.json`:
   ```json
   "macOS": {
     "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
     "hardenedRuntime": true,
     "entitlements": "path/to/entitlements.plist"
   }
   ```
3. Notarize after building (required for macOS 10.14+)

### Windows
1. Obtain code signing certificate
2. Update `tauri.conf.json`:
   ```json
   "windows": {
     "certificateThumbprint": "YOUR_CERT_THUMBPRINT",
     "digestAlgorithm": "sha256"
   }
   ```

### Linux
- No code signing required
- Distribution repositories may have their own requirements

## Icons

Icons are located in `src-tauri/icons/`:
- `icon.png` - Source icon (1024x1024 recommended)
- `icon.ico` - Windows icon (generated)
- `icon.icns` - macOS icon (generated)
- Various PNG sizes for different platforms

To regenerate icons from source:
```bash
npm run tauri icon path/to/source-icon.png
```

## Security

### Content Security Policy
The app uses a restrictive CSP:
- Scripts: Self + inline (required for Vite)
- Connections: Self + HiNotes API
- Images: Self + data URIs + blobs
- Media: Self + blobs (for audio recordings)
- WASM: Allowed for audio processing

### Hardened Runtime (macOS)
Enabled with the following entitlements:
- USB device access
- Network connections
- Audio recording (if implementing voice features)

## Dependencies

### Rust Dependencies
- `tauri` - Core framework
- `tauri-plugin-opener` - Open files/URLs
- `tauri-plugin-updater` - Auto-update functionality
- `rusb` - USB device communication
- `rusqlite` - Local database
- `reqwest` - HTTP client
- `tokio` - Async runtime
- See `Cargo.toml` for complete list

### System Dependencies

#### Linux (Debian/Ubuntu)
```bash
sudo apt install libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libusb-1.0-0-dev \
  libsqlite3-dev
```

#### Linux (Fedora/RHEL)
```bash
sudo dnf install webkit2gtk4.1-devel \
  gtk3-devel \
  libappindicator-gtk3-devel \
  librsvg2-devel \
  libusb-devel \
  sqlite-devel
```

#### macOS
```bash
brew install libusb
```

#### Windows
- Visual Studio Build Tools
- WebView2 (auto-installed)
- USB drivers (provided by Windows)

## Troubleshooting

### Build Failures

**Error: USB library not found**
```bash
# Linux
sudo apt install libusb-1.0-0-dev

# macOS
brew install libusb
```

**Error: WebKit not found (Linux)**
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**Error: Code signing failed (macOS)**
- Verify certificate is installed in Keychain
- Ensure identity matches exactly
- Check entitlements file is valid

### Runtime Issues

**USB device not detected**
- Check device permissions (Linux: udev rules)
- Verify device is connected and recognized by OS
- Check app has necessary capabilities

**Update check fails**
- Verify update endpoint URL is accessible
- Check public key matches private key used for signing
- Ensure `latest.json` is valid JSON and properly signed

**File access denied**
- Check path is within allowed scopes
- Verify app has filesystem permissions (macOS: System Preferences)
- Check CSP allows required operations

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        platform: [macos-latest, ubuntu-latest, windows-latest]
    runs-on: ${{ matrix.platform }}
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Install dependencies (Ubuntu)
        if: matrix.platform == 'ubuntu-latest'
        run: |
          sudo apt update
          sudo apt install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libusb-1.0-0-dev
      
      - name: Install frontend dependencies
        run: npm ci
      
      - name: Build app
        run: npm run tauri build
        env:
          TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
          TAURI_SIGNING_PASSWORD: ${{ secrets.TAURI_SIGNING_PASSWORD }}
      
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.platform }}-bundle
          path: src-tauri/target/release/bundle/
```

## Additional Resources

- [Tauri Documentation](https://tauri.app/v2/guides/)
- [Tauri Bundle Configuration](https://tauri.app/v2/reference/config/#bundleconfig)
- [Tauri Updater Guide](https://tauri.app/v2/guides/distribution/updater/)
- [Code Signing for macOS](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
