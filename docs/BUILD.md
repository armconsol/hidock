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

**Status:** Full native installer builds, gated behind Rust/frontend checks.

**Available Workflows:**
1. **Build Native Installers** (`.gitea/workflows/build-installers.yml`) -
   Runs `rust-fmt-check`, `rust-clippy`, `rust-tests`, `frontend-typecheck`,
   `frontend-tests` first; `build-macos`/`build-linux`/`build-windows` all
   `needs:` those checks and only run if they pass.
2. **Test Workflow** (`.gitea/workflows/test.yml`) - Same checks, PR-only
   (pushes to main are covered by the gating jobs above).
3. **PR Review Workflow** (`.gitea/workflows/pr-review.yml`) - Full PR
   validation (code quality, tests, integration build, security audit).

**Monitor Progress:**
- https://gogs.tftsr.com/sarman/hinotes/actions

**Note:** See "CI/CD Integration" below for why Gitea workflows here use
`linux-amd64`/`macos-arm64` runner labels and Harbor container images instead
of `ubuntu-latest`/`macos-latest`/`windows-latest` and `actions/checkout@v4`.

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

Workflows run on this Gitea instance's self-hosted runners, using the same
runner labels and pre-baked Harbor images as the `tftsr-devops_investigation`
(TRCAA) project:

- `runs-on: linux-amd64` with `container: harbor.tftsr.com/tftsr/tftsr-linux-amd64:rust1.89-node22`
  for Rust jobs (Rust + Node.js + webkit/gtk build deps preinstalled).
- `runs-on: linux-amd64` with `container: harbor.tftsr.com/tftsr/tftsr-windows-cross:rust1.89-node22`
  for Windows builds (mingw cross-compilation, no native Windows runner exists).
- `runs-on: macos-arm64` (native, no container) for macOS builds — only an
  arm64 Mac runner is registered, so macOS builds are `aarch64-apple-darwin`
  only (no universal binary).
- `runs-on: ubuntu-latest` with `container: node:22-alpine` for frontend-only
  jobs.

**If `build-macos` sits in `queued` forever with no runner picking it up**,
the macOS runner host (`172.0.1.54`, hostname `mac-arm64-runner-2` in
`/admin/actions/runners`) uses a local relay chain to reach Gitea, since
headless launchd processes can't resolve `gogs.tftsr.com` on macOS's Local
Network otherwise: `/etc/hosts` maps `gogs.tftsr.com` → `127.0.0.1`, a
root-owned `forward443` LaunchDaemon listens on `127.0.0.1:443` and forwards
to a `GiteaRunnerRelay.app` LaunchAgent (listening on `127.0.0.1:8443`), which
proxies to the real Gitea at `172.0.0.70:443` with the correct TLS SNI. If
either link in that chain hangs, `act_runner`'s log
(`~/gitea-runner/runner.err.log` on that host) shows `Your Gitea version is
too old to support runner declare` / `unimplemented: unary response has zero
messages` even though Gitea itself is fine — check with `curl -sk
https://gogs.tftsr.com/api/v1/version` on that host; a `SSL_ERROR_SYSCALL`
there means the relay chain is broken, not Gitea. Restart with `sudo
launchctl kickstart -k system/com.tftsr.gitea-runner-portforward` and (as
the `sarman` user) `launchctl kickstart -k
gui/$(id -u)/com.tftsr.gitea-runner-relay`.

**Do not use `actions/checkout@v4`, `actions/setup-node@v4`, or
`runs-on: ubuntu-latest`/`macos-latest`/`windows-latest` without a `container:`
image on this Gitea instance.** The Docker-based act_runner here does not
reliably support `actions/checkout`'s Node-based execution when the job
container has no Node.js preinstalled (checkout fails with `exec: "node":
executable file not found in $PATH`), and there is no runner registered under
the `macos-latest`/`windows-latest` labels — jobs using them queue forever.
Use the manual `git init && git remote add && git fetch --depth=1 && git
checkout FETCH_HEAD` pattern (see any job in `.gitea/workflows/`) with
`secrets.GITHUB_TOKEN` instead.

**Do not use `actions/upload-artifact` or `actions/download-artifact` on this
Gitea instance either** — confirmed failing with `GHESNotSupportedError:
@actions/artifact v2.0.0+, upload-artifact@v4+ and download-artifact@v4+ are
not currently supported on GHES`. Each build job in `build-installers.yml`
instead `curl`s its installer directly to a Gitea release on version-tag
pushes, using a `RELEASE_TOKEN` secret (a Gitea personal access token with
repo write access) — same pattern as `tftsr-devops_investigation`'s
`release-beta.yml`. `actions/cache@v4` is fine and used throughout; only the
artifact upload/download actions are broken here.

**1. Build Installers Workflow (`.gitea/workflows/build-installers.yml`)**
- Triggers on push to main and version tags
- Gating checks first: `rust-fmt-check`, `rust-clippy`, `rust-tests` (Harbor
  Rust image), `frontend-typecheck`, `frontend-tests` (`node:22-alpine`)
- `build-macos`, `build-linux`, `build-windows` all `needs:` every check job
  above and only start once they all pass
- Builds Linux (deb/rpm/AppImage) on `linux-amd64`, Windows NSIS installer
  (cross-compiled via mingw) on `linux-amd64`, and macOS arm64 DMG on
  `macos-arm64`
- On a `v*` tag push, each build job uploads its installer to a Gitea
  release via `RELEASE_TOKEN` (creating the release if it doesn't exist yet)
- Windows bundle target is `nsis`, not `msi` — WiX/MSI cannot be
  cross-compiled from Linux; NSIS can (see `bundle.targets` in
  `src-tauri/tauri.conf.json`)
- The Linux job installs `xdg-utils` — Tauri's AppImage bundler shells out to
  `xdg-open` and fails the whole build (`xdg-open binary not found`) without
  it; the Harbor `tftsr-linux-amd64` image doesn't include it by default
- Gitea Actions has no reliable cross-workflow gating (no `workflow_run`
  equivalent — see docs.gitea.com/usage/actions/comparison), so the checks
  live in this same workflow file rather than a separate `test.yml` that
  build jobs can depend on

**2. Test Workflow (`.gitea/workflows/test.yml`)**
- Same checks as above, but triggers on pull requests only — pushes to main
  are already covered by the gating jobs in `build-installers.yml`

**3. PR Review Workflow (`.gitea/workflows/pr-review.yml`)**
- Triggers on all PRs to main/develop
- Comprehensive validation:
  - Code quality (rustfmt, clippy, placeholder check)
  - Backend tests (Rust)
  - Frontend tests (React/Vitest) + frontend build
  - Integration tests (full Tauri build)
  - Security audit (cargo-audit, npm audit)
- Generates summary report

### GitHub Actions mirror

`.github/workflows/` contains an equivalent set of workflows for if/when this
repo is mirrored to github.com, using GitHub-hosted runners
(`ubuntu-latest`/`macos-latest`/`windows-latest`) and `actions/checkout@v4`
directly — those work fine there since GitHub-hosted runners have Node.js
preinstalled on the VM. The two directories are intentionally different and
should not be kept byte-for-byte identical.

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
