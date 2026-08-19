# Build Guide

This document describes how to build HiNotes Desktop for all supported platforms.

## Quick Start

### Automated Builds

#### GitHub Actions (github.com)

**Native Installer Builds:**
1. **Tag-based release:**
   ```bash
   git tag v1.0.0-beta
   git push origin v1.0.0-beta
   ```
   - Automatically builds for macOS (Universal), Linux (AppImage/deb/rpm), Windows (MSI)
   - Creates GitHub release with all installers attached

2. **Manual build:**
   - Go to Actions → Build Native Installers → Run workflow
   - Select platform (all/macos/linux/windows)
   - Download artifacts after build completes

**Monitor Progress:**
- https://github.com/YOUR_USERNAME/hinotes/actions

**Available Workflows:**
1. **Build Native Installers** (`.github/workflows/build-installers.yml`)
   - Triggered on version tags (`v*`) or manual dispatch
   - Builds universal macOS DMG, Linux packages, Windows MSI
   - Creates GitHub release automatically for tagged builds

2. **Test Workflow** (`.github/workflows/test.yml`)
   - Runs on push to main and PRs
   - Backend tests (Rust) + Frontend tests (React/Vitest)

3. **PR Review Workflow** (`.github/workflows/pr-review.yml`)
   - Comprehensive PR validation
   - Code quality, tests, integration build, security audit

#### Gitea Actions (gogs.tftsr.com)

**Status:** Testing only (no native installer builds)

**Available Workflows:**
1. **Test Workflow** (`.gitea/workflows/test.yml`) - Backend + Frontend tests
2. **PR Review Workflow** (`.gitea/workflows/pr-review.yml`) - Full PR validation

**Monitor Progress:**
- https://gogs.tftsr.com/sarman/hinotes/actions

**Note:** Gitea workflows use container-based runners and cannot build native installers. Use GitHub Actions for installer builds.

---

## Local Builds

### Prerequisites

**All Platforms:**
- Node.js 18+ with npm
- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Git

**macOS:**
```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add targets for universal binary
rustup target add aarch64-apple-darwin x86_64-apple-darwin
```

**Linux (Ubuntu/Debian):**
```bash
# Install system dependencies
sudo apt-get update
sudo apt-get install -y \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  pkg-config \
  build-essential \
  curl \
  wget \
  file

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**Windows:**
```powershell
# Install Node.js from https://nodejs.org/
# Install Rust from https://rustup.rs/

# Install Visual Studio Build Tools (required)
# Download from: https://visualstudio.microsoft.com/downloads/
# Select: "Desktop development with C++"
```

### Build Steps

**1. Clone and Install:**
```bash
git clone https://gogs.tftsr.com/sarman/hinotes.git
cd hinotes
npm install
```

**2. Configure Environment (Optional):**
```bash
cp .env.example .env
# Edit .env and add OAuth credentials if needed
```

**3. Build for Current Platform:**
```bash
# Development build (fast)
npm run tauri dev

# Production build (optimized)
npm run tauri build
```

**4. Platform-Specific Builds:**

**macOS Universal Binary:**
```bash
npm run tauri build -- --target universal-apple-darwin
# Output: src-tauri/target/universal-apple-darwin/release/bundle/dmg/
```

**Linux All Formats:**
```bash
npm run tauri build
# Outputs:
# - src-tauri/target/release/bundle/appimage/
# - src-tauri/target/release/bundle/deb/
# - src-tauri/target/release/bundle/rpm/
```

**Windows MSI:**
```bash
npm run tauri build
# Output: src-tauri\target\release\bundle\msi\
```

---

## Build Scripts

Convenience scripts for platform-specific builds:

**macOS:**
```bash
./scripts/build-mac.sh
```

**Linux:**
```bash
./scripts/build-linux.sh
```

**Windows:**
```powershell
.\scripts\build-windows.ps1
```

---

## Code Signing

### macOS

**Requirements:**
- Apple Developer account ($99/year)
- Developer ID Application certificate
- App-specific password for notarization

**Setup:**
```bash
# Export certificates from Keychain
security find-identity -v -p codesigning

# Set environment variables
export APPLE_CERTIFICATE="<base64-encoded-p12>"
export APPLE_CERTIFICATE_PASSWORD="<password>"
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
export APPLE_ID="your-email@example.com"
export APPLE_PASSWORD="<app-specific-password>"
export APPLE_TEAM_ID="<team-id>"

# Build with signing
npm run tauri build -- --target universal-apple-darwin
```

**Notarization:**
Automatic via Tauri if credentials are set. Manual notarization:
```bash
xcrun notarytool submit HiNotes-Desktop.dmg \
  --apple-id "your-email@example.com" \
  --password "<app-specific-password>" \
  --team-id "<team-id>" \
  --wait

xcrun stapler staple HiNotes-Desktop.dmg
```

### Windows

**Requirements:**
- Code signing certificate (EV or OV)
- signtool.exe (included in Windows SDK)

**Setup:**
```powershell
# Import certificate to Windows Certificate Store
# Or use PFX file

# Set environment variables
$env:WINDOWS_CERTIFICATE = "<base64-pfx>"
$env:WINDOWS_CERTIFICATE_PASSWORD = "<password>"

# Build with signing
npm run tauri build
```

### Linux

Linux packages typically don't require code signing for distribution via GitHub/Gitea releases. For official repos (apt/yum), signing is repository-specific.

---

## Troubleshooting

### Build Failures

**"cargo: command not found"**
```bash
# Ensure Rust is in PATH
source $HOME/.cargo/env
```

**"error: linking with \`cc\` failed"**
```bash
# Linux: Install build essentials
sudo apt-get install build-essential

# macOS: Install Xcode Command Line Tools
xcode-select --install
```

**"Failed to bundle project"**
```bash
# Clear build cache
rm -rf src-tauri/target
npm run tauri build
```

**FFmpeg missing (Linux)**
```bash
# Install FFmpeg
sudo apt-get install ffmpeg  # Debian/Ubuntu
sudo dnf install ffmpeg      # Fedora
sudo pacman -S ffmpeg        # Arch
```

### Performance Issues

**Slow builds:**
```bash
# Use release mode with optimizations
npm run tauri build

# Enable parallel compilation
export CARGO_BUILD_JOBS=8
```

**Large binary size:**
```bash
# Strip debug symbols (Linux/macOS)
strip src-tauri/target/release/hinotes-desktop

# Use UPX compression (optional)
upx --best src-tauri/target/release/hinotes-desktop
```

---

## CI/CD Integration

### Gitea Actions

The project includes two workflows that run on `ubuntu-latest` runners with Docker containers:

**1. Test Workflow (`.gitea/workflows/test.yml`)**
- Triggers on push to main and pull requests
- Uses rustlang/rust:nightly and node:20 containers
- Runs backend (cargo test) and frontend (npm test) tests
- Manual git checkout pattern with GITHUB_TOKEN authentication

**2. PR Review Workflow (`.gitea/workflows/pr-review.yml`)**
- Triggers on all PRs to main/develop
- Comprehensive validation:
  - Code quality (rustfmt, clippy, placeholder check)
  - Backend tests (Rust)
  - Frontend tests (React/Vitest)
  - Integration tests (full Tauri build)
  - Security audit (cargo-audit, npm audit)
- Generates summary report

**Workflow Pattern:**
- Uses public container images (rustlang/rust:nightly, node:20, alpine)
- Manual git checkout with token authentication
- No dependency on GitHub Actions marketplace actions
- Compatible with self-hosted Gitea Actions runners

### Mirroring Between Gitea and GitHub

The project supports both Gitea Actions (gogs.tftsr.com) and GitHub Actions (github.com):

- **Gitea Actions**: Container-based, testing only
- **GitHub Actions**: Native runners, includes installer builds

**Workflow files:**
- `.gitea/workflows/` - For Gitea Actions (container-based, manual git checkout)
- `.github/workflows/` - For GitHub Actions (uses actions/checkout, includes build-installers.yml)

Both sets work independently when the repository is mirrored to both platforms.

---

## Release Process

**1. Update Version:**
```bash
# Edit package.json
vim package.json  # Set version to 1.0.0-beta

# Update CHANGELOG.md
vim CHANGELOG.md  # Add release notes

# Commit
git add package.json CHANGELOG.md
git commit -m "chore: Bump version to 1.0.0-beta"
git push
```

**2. Create Tag:**
```bash
git tag -a v1.0.0-beta -m "Release v1.0.0-beta"
git push origin v1.0.0-beta
```

**3. Monitor Build:**
- https://gogs.tftsr.com/sarman/hinotes/actions

**4. Download Artifacts:**
- macOS: `HiNotes-Desktop_1.0.0-beta_universal.dmg`
- Linux: AppImage, deb, rpm
- Windows: `HiNotes-Desktop_1.0.0-beta_x64.msi`

**5. Test Installers:**
- Install on clean systems
- Verify OAuth flows work
- Test offline functionality
- Check auto-update (if configured)

**6. Publish Release:**
- Download artifacts from Actions
- Create release on Gitea: https://gogs.tftsr.com/sarman/hinotes/releases
- Upload installers
- Publish release notes

---

## Build Artifacts

### macOS
- **DMG**: Drag-to-Applications installer
- **Size**: ~80-100 MB (universal binary)
- **Architectures**: ARM64 + x86_64
- **Minimum OS**: macOS 11.0 (Big Sur)

### Linux
- **AppImage**: Portable, no installation needed
- **deb**: For Debian/Ubuntu systems
- **rpm**: For Fedora/RHEL systems
- **Size**: ~60-80 MB
- **Architectures**: x86_64 (amd64)

### Windows
- **MSI**: Standard Windows installer
- **Size**: ~70-90 MB
- **Architectures**: x64
- **Minimum OS**: Windows 10

---

## Development Builds

For quick development testing:

```bash
# Run in development mode (hot reload)
npm run tauri dev

# Build debug binary (faster, larger)
npm run tauri build -- --debug

# Run tests
npm test                   # Frontend
cargo test --manifest-path=src-tauri/Cargo.toml  # Backend
```

---

## Cross-Compilation

**macOS → Linux/Windows**: Not supported (use Gitea Actions)
**Linux → macOS/Windows**: Not supported (use Gitea Actions)
**Windows → macOS/Linux**: Not supported (use Gitea Actions)

**Recommended**: Use Gitea Actions for multi-platform builds.

---

## Build Cache

**Cargo (Rust):**
```bash
# Cache location
~/.cargo/

# Clear cache
cargo clean
rm -rf ~/.cargo/registry/cache
```

**npm:**
```bash
# Cache location
~/.npm/

# Clear cache
npm cache clean --force
```

**Tauri:**
```bash
# Clear Tauri build cache
rm -rf src-tauri/target
```

---

## Support

- **Build Issues**: https://gogs.tftsr.com/sarman/hinotes/issues
- **Tauri Docs**: https://tauri.app/
- **Rust Docs**: https://doc.rust-lang.org/
