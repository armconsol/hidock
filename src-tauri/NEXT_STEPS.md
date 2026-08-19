# Next Steps for Build Configuration

This document outlines the remaining tasks to complete the build setup.

## ✅ Completed

1. **Bundle Configuration**
   - macOS: DMG, App bundle (universal binary: x86_64 + aarch64)
   - Linux: .deb, .AppImage, .rpm
   - Windows: .msi installer
   - All targets configured in `tauri.conf.json`

2. **App Metadata**
   - Product name: HiNotes Desktop
   - Version: Reads from `package.json` (1.0.0-alpha)
   - Description, author, license configured
   - Bundle metadata (category, copyright, homepage) set

3. **Capabilities**
   - USB device access (via rusb)
   - File system access (scoped to user directories)
   - Network access (HiNotes API endpoint)
   - Window management permissions
   - Path access permissions configured

4. **Auto-Updater**
   - Tauri updater plugin added to `Cargo.toml`
   - Configuration added to `tauri.conf.json`
   - Update endpoint template provided

5. **Icons**
   - All required icon formats present in `src-tauri/icons/`
   - Source icon available: `icon.png`

6. **Documentation**
   - `BUILD.md` created with comprehensive build guide
   - Troubleshooting section included
   - CI/CD examples provided

## ⚠️ Required Before First Build

### 1. Update GitHub Repository URL

In `tauri.conf.json`, replace placeholder URLs:
```json
"homepage": "https://github.com/yourusername/hidoc"
"endpoints": [
  "https://github.com/yourusername/hidoc/releases/latest/download/latest.json"
]
```

Replace `yourusername` with actual GitHub username.

### 2. Generate Update Signing Keys

The auto-updater requires cryptographic signing:

```bash
# Generate key pair
npm run tauri signer generate -- -w ~/.tauri/hinotes-desktop.key

# This creates two files:
# - ~/.tauri/hinotes-desktop.key (PRIVATE - keep secure, never commit)
# - ~/.tauri/hinotes-desktop.key.pub (PUBLIC - add to tauri.conf.json)
```

Then update `tauri.conf.json`:
```json
"updater": {
  "pubkey": "PASTE_PUBLIC_KEY_HERE"
}
```

### 3. Test Build

Run a test build to ensure configuration is correct:

```bash
# Development mode (faster, no signing required)
npm run tauri dev

# Production build (full bundle creation)
npm run tauri build
```

Expected output locations:
- macOS: `src-tauri/target/release/bundle/dmg/` and `src-tauri/target/release/bundle/macos/`
- Linux: `src-tauri/target/release/bundle/deb/`, `appimage/`, `rpm/`
- Windows: `src-tauri/target/release/bundle/msi/`

## 🔒 Code Signing (Optional but Recommended)

### macOS

Required for distribution outside of Mac App Store and to avoid "unidentified developer" warnings.

1. **Enroll in Apple Developer Program** ($99/year)
2. **Obtain Developer ID Certificate**
   - Log in to Apple Developer portal
   - Create "Developer ID Application" certificate
   - Download and install in Keychain

3. **Update tauri.conf.json**:
   ```json
   "macOS": {
     "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)"
   }
   ```

4. **Notarize** (macOS 10.14+):
   ```bash
   # After building, submit for notarization
   xcrun notarytool submit \
     "src-tauri/target/release/bundle/dmg/HiNotes Desktop_1.0.0-alpha_x64.dmg" \
     --apple-id "your-email@example.com" \
     --team-id "YOUR_TEAM_ID" \
     --password "app-specific-password"
   
   # Staple notarization ticket
   xcrun stapler staple "src-tauri/target/release/bundle/dmg/HiNotes Desktop_1.0.0-alpha_x64.dmg"
   ```

### Windows

Recommended for avoiding SmartScreen warnings.

1. **Obtain Code Signing Certificate**
   - Purchase from certificate authority (Sectigo, DigiCert, etc.)
   - Or use Azure Code Signing for cloud-based signing

2. **Install Certificate** (if file-based)
   ```powershell
   Import-PfxCertificate -FilePath cert.pfx -CertStoreLocation Cert:\CurrentUser\My
   ```

3. **Get Certificate Thumbprint**
   ```powershell
   Get-ChildItem -Path Cert:\CurrentUser\My | Where-Object {$_.Subject -match "Your Name"}
   ```

4. **Update tauri.conf.json**:
   ```json
   "windows": {
     "certificateThumbprint": "YOUR_CERTIFICATE_THUMBPRINT"
   }
   ```

### Linux

No code signing required. Distribution via package repositories may require GPG signing of repository metadata (separate process).

## 🚀 Release Process

### 1. Prepare Release

```bash
# Update version in package.json
npm version 1.0.0-beta

# Commit version bump
git add package.json package-lock.json
git commit -m "chore: Bump version to 1.0.0-beta"

# Create git tag
git tag -a v1.0.0-beta -m "Release v1.0.0-beta"
```

### 2. Build Release Artifacts

```bash
# Build for current platform
npm run tauri build

# macOS: Build universal binary (requires macOS)
npm run tauri build -- --target universal-apple-darwin
```

### 3. Create GitHub Release

```bash
# Push tag
git push origin v1.0.0-beta

# Create release on GitHub
gh release create v1.0.0-beta \
  --title "HiNotes Desktop v1.0.0-beta" \
  --notes "Release notes here" \
  src-tauri/target/release/bundle/**/*.{dmg,deb,AppImage,rpm,msi} \
  src-tauri/target/release/bundle/**/latest.json
```

### 4. Verify Auto-Update

After publishing release:
1. Install previous version of app
2. Launch app
3. App should detect new version and prompt to update
4. Verify update downloads and installs correctly

## 🛠️ Platform-Specific Setup

### macOS Development

```bash
# Install Xcode Command Line Tools
xcode-select --install

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install USB library
brew install libusb
```

### Linux Development (Ubuntu/Debian)

```bash
# Install system dependencies
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libusb-1.0-0-dev \
  libsqlite3-dev \
  build-essential \
  curl \
  wget \
  file

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Windows Development

1. **Install Visual Studio Build Tools**
   - Download from: https://visualstudio.microsoft.com/downloads/
   - Select "Desktop development with C++"

2. **Install Rust**
   - Download from: https://www.rust-lang.org/tools/install
   - Use `rustup-init.exe`

3. **Install Node.js**
   - Download from: https://nodejs.org/

## 📋 Testing Checklist

Before releasing:

- [ ] App launches successfully
- [ ] Window opens with correct size and title
- [ ] USB device detection works (if device available)
- [ ] File system operations work (read/write notes)
- [ ] Network requests to HiNotes API succeed
- [ ] Settings persist after restart
- [ ] Auto-update check works (doesn't crash if no update available)
- [ ] All navigation flows work correctly
- [ ] Audio playback works
- [ ] App icon appears correctly in dock/taskbar
- [ ] Installer/package installs cleanly
- [ ] Uninstaller removes all files (Windows/Linux)

## 🐛 Known Issues

### macOS

- **Universal binary size**: Building universal binaries (x86_64 + aarch64) doubles the app size. Consider separate builds if size is critical.
- **Notarization time**: Can take 5-30 minutes for Apple to notarize an app.
- **Gatekeeper**: First launch requires right-click → Open if not notarized.

### Linux

- **USB permissions**: Requires udev rules for non-root access. See `BUILD.md` for details.
- **AppImage integration**: May not automatically integrate with system (desktop file, icons). Consider distribution-specific packages.
- **WebKit version**: Different distributions ship different WebKit versions. Test on target distributions.

### Windows

- **SmartScreen**: Unsigned apps trigger SmartScreen warnings. Code signing required for clean install.
- **WebView2**: Auto-installer requires internet connection. Consider offline installer for enterprise deployment.
- **Antivirus**: Some AV software flags new/unsigned executables. Whitelist or sign the app.

## 📚 Additional Resources

- **Tauri v2 Documentation**: https://tauri.app/v2/guides/
- **Updater Guide**: https://tauri.app/v2/guides/distribution/updater/
- **Bundle Configuration**: https://tauri.app/v2/reference/config/#bundleconfig
- **Code Signing Guide**: https://tauri.app/v2/guides/distribution/sign/
- **macOS Notarization**: https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution

## 🎯 Recommended Next Steps

1. Update GitHub URLs in `tauri.conf.json`
2. Generate updater signing keys
3. Run `npm run tauri dev` to test development build
4. Run `npm run tauri build` to create production bundle
5. Test installation on clean system
6. Set up CI/CD pipeline (GitHub Actions example in `BUILD.md`)
7. Create first release with release notes
8. Test auto-update functionality

## 💡 Tips

- **Keep signing keys secure**: Store private keys in secure location, never commit to git
- **Test on clean systems**: VMs or Docker containers help catch missing dependencies
- **Monitor bundle sizes**: Large apps slow downloads. Optimize assets and dependencies.
- **Version carefully**: Semantic versioning helps users understand update significance
- **Document breaking changes**: Help users understand what changes between versions
