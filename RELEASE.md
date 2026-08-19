# Release and Code Signing Guide

This document provides comprehensive guidance for building, code signing, and releasing the HiDoc P1 USB monitor application across macOS, Windows, and Linux platforms.

## Table of Contents

- [Overview](#overview)
- [macOS Code Signing](#macos-code-signing)
  - [Prerequisites](#macos-prerequisites)
  - [Obtaining Developer ID Certificate](#obtaining-developer-id-certificate)
  - [Code Signing Process](#macos-code-signing-process)
  - [Notarization](#notarization)
  - [CI/CD Configuration](#macos-cicd-configuration)
- [Windows Code Signing](#windows-code-signing)
  - [Certificate Options](#windows-certificate-options)
  - [Signing Process](#windows-signing-process)
  - [CI/CD Configuration](#windows-cicd-configuration)
- [Linux Code Signing](#linux-code-signing)
  - [AppImage Signing](#appimage-signing)
  - [Package Repository Signing](#package-repository-signing)
- [Troubleshooting](#troubleshooting)
  - [macOS Issues](#macos-troubleshooting)
  - [Windows Issues](#windows-troubleshooting)
  - [Linux Issues](#linux-troubleshooting)
- [Release Checklist](#release-checklist)

---

## Overview

Code signing is essential for distributing applications without triggering security warnings. This guide covers:

- **macOS**: Developer ID certificates and notarization (required)
- **Windows**: Code signing certificates (optional but recommended)
- **Linux**: AppImage and package repository signing (optional)

---

## macOS Code Signing

### macOS Prerequisites

1. **Apple Developer Account** ($99/year)
2. **Xcode Command Line Tools** installed
3. **Developer ID Application Certificate** in Keychain
4. **App-Specific Password** for notarization

### Obtaining Developer ID Certificate

#### Step 1: Enroll in Apple Developer Program

1. Visit [developer.apple.com](https://developer.apple.com)
2. Enroll in the Apple Developer Program ($99/year)
3. Wait for enrollment confirmation (may take 24-48 hours)

#### Step 2: Create Certificate Signing Request (CSR)

```bash
# Open Keychain Access
open "/Applications/Utilities/Keychain Access.app"

# Menu: Keychain Access > Certificate Assistant > Request a Certificate from a Certificate Authority
# - User Email Address: your@email.com
# - Common Name: Your Name
# - CA Email Address: (leave empty)
# - Request: "Saved to disk"
# - Key Pair Information: RSA, 2048 bits
```

This creates `CertificateSigningRequest.certSigningRequest` on your Desktop.

#### Step 3: Download Developer ID Certificate

1. Log in to [developer.apple.com/account/resources/certificates](https://developer.apple.com/account/resources/certificates)
2. Click "+" to create a new certificate
3. Select "Developer ID Application" (for distributing apps outside the Mac App Store)
4. Upload your CSR file
5. Download the certificate (`developerID_application.cer`)
6. Double-click to install in Keychain Access

#### Step 4: Verify Certificate Installation

```bash
# List all Developer ID certificates
security find-identity -v -p codesigning

# Expected output:
# 1) ABCDEF1234567890... "Developer ID Application: Your Name (TEAM_ID)"
```

### macOS Code Signing Process

#### Using Tauri CLI (Recommended)

Tauri automatically signs the app bundle during build:

```bash
# Build and sign for production
npm run tauri build

# Tauri will use the certificate from your keychain
# Configuration in tauri.conf.json:
```

**tauri.conf.json snippet:**

```json
{
  "tauri": {
    "bundle": {
      "macOS": {
        "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
        "entitlements": "entitlements.plist",
        "exceptionDomain": null
      }
    }
  }
}
```

#### Manual Code Signing

If you need to sign manually or re-sign:

```bash
# Sign the app bundle
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --force \
  --options runtime \
  --entitlements entitlements.plist \
  --timestamp \
  --deep \
  src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app

# Verify signature
codesign --verify --verbose=4 src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app

# Display signature details
codesign -dvv src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app

# Check if hardened runtime is enabled
codesign -dvvv src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app 2>&1 | grep -i runtime
```

#### Entitlements File

Create `entitlements.plist` in `src-tauri/`:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <!-- Required for hardened runtime -->
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <false/>
    
    <!-- USB device access -->
    <key>com.apple.security.device.usb</key>
    <true/>
    
    <!-- Network access (for WebUSB API) -->
    <key>com.apple.security.network.client</key>
    <true/>
    
    <!-- Disable library validation (if needed for third-party libs) -->
    <key>com.apple.security.cs.disable-library-validation</key>
    <false/>
</dict>
</plist>
```

### Notarization

Notarization is required for apps distributed outside the Mac App Store (macOS 10.15+).

#### Step 1: Create App-Specific Password

1. Visit [appleid.apple.com](https://appleid.apple.com)
2. Sign in with your Apple ID
3. Go to "Security" > "App-Specific Passwords"
4. Click "Generate an app-specific password"
5. Label it "Notarization" and copy the password

#### Step 2: Store Credentials in Keychain

```bash
# Store notarization credentials
xcrun notarytool store-credentials "notarization-profile" \
  --apple-id "your@email.com" \
  --team-id "YOUR_TEAM_ID" \
  --password "xxxx-xxxx-xxxx-xxxx"

# Verify stored credentials
xcrun notarytool history --keychain-profile "notarization-profile"
```

#### Step 3: Create DMG for Distribution

```bash
# Create a DMG from the app bundle
hdiutil create -volname "HiDoc P1 Monitor" \
  -srcfolder src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app \
  -ov -format UDZO \
  HiDoc-P1-Monitor.dmg

# Sign the DMG
codesign --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --timestamp \
  HiDoc-P1-Monitor.dmg
```

#### Step 4: Submit for Notarization

```bash
# Submit DMG for notarization
xcrun notarytool submit HiDoc-P1-Monitor.dmg \
  --keychain-profile "notarization-profile" \
  --wait

# Expected output:
# Conducting pre-submission checks for HiDoc-P1-Monitor.dmg and initiating connection to the Apple notary service...
# Submission ID: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
# Successfully uploaded file
# ...
# Processing complete
#   id: xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx
#   status: Accepted
```

#### Step 5: Staple Notarization Ticket

```bash
# Staple the notarization ticket to the DMG
xcrun stapler staple HiDoc-P1-Monitor.dmg

# Verify stapling
xcrun stapler validate HiDoc-P1-Monitor.dmg

# Expected output:
# The validate action worked!
```

#### Check Notarization Status

```bash
# Get notarization info by submission ID
xcrun notarytool info SUBMISSION_ID --keychain-profile "notarization-profile"

# View notarization log (if rejected)
xcrun notarytool log SUBMISSION_ID --keychain-profile "notarization-profile"
```

### macOS CI/CD Configuration

#### GitHub Actions Secrets

Add these secrets to your repository (Settings > Secrets and variables > Actions):

- `APPLE_CERTIFICATE_BASE64`: Base64-encoded .p12 certificate
- `APPLE_CERTIFICATE_PASSWORD`: Password for .p12 file
- `APPLE_ID`: Apple ID email for notarization
- `APPLE_TEAM_ID`: Team ID from developer.apple.com
- `APPLE_APP_SPECIFIC_PASSWORD`: App-specific password for notarization

#### Export Certificate as P12

```bash
# Export from Keychain as .p12
# 1. Open Keychain Access
# 2. Find "Developer ID Application: Your Name"
# 3. Right-click > Export "Developer ID Application: Your Name"
# 4. Save as .p12 with a strong password

# Convert to base64 for GitHub Secrets
base64 -i DeveloperID.p12 | pbcopy

# Paste into GitHub Secrets as APPLE_CERTIFICATE_BASE64
```

#### GitHub Actions Workflow Example

```yaml
name: Release macOS

on:
  push:
    tags:
      - 'v*'

jobs:
  build-macos:
    runs-on: macos-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          
      - name: Install dependencies
        run: npm ci
      
      - name: Import Code Signing Certificate
        env:
          APPLE_CERTIFICATE_BASE64: ${{ secrets.APPLE_CERTIFICATE_BASE64 }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          # Create temporary keychain
          KEYCHAIN_PATH=$RUNNER_TEMP/build.keychain
          KEYCHAIN_PASSWORD=$(openssl rand -base64 32)
          
          security create-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          security set-keychain-settings -lut 21600 "$KEYCHAIN_PATH"
          security unlock-keychain -p "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          
          # Import certificate
          CERT_PATH=$RUNNER_TEMP/certificate.p12
          echo "$APPLE_CERTIFICATE_BASE64" | base64 --decode > "$CERT_PATH"
          security import "$CERT_PATH" -k "$KEYCHAIN_PATH" -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          
          # Allow codesign to access certificate
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "$KEYCHAIN_PASSWORD" "$KEYCHAIN_PATH"
          
          # Add keychain to search list
          security list-keychain -d user -s "$KEYCHAIN_PATH" $(security list-keychain -d user | sed s/\"//g)
      
      - name: Build Tauri App
        env:
          APPLE_SIGNING_IDENTITY: "Developer ID Application"
        run: npm run tauri build
      
      - name: Create DMG
        run: |
          hdiutil create -volname "HiDoc P1 Monitor" \
            -srcfolder src-tauri/target/release/bundle/macos/HiDoc\ P1\ Monitor.app \
            -ov -format UDZO \
            HiDoc-P1-Monitor.dmg
          
          codesign --sign "Developer ID Application" --timestamp HiDoc-P1-Monitor.dmg
      
      - name: Notarize DMG
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
          APPLE_APP_SPECIFIC_PASSWORD: ${{ secrets.APPLE_APP_SPECIFIC_PASSWORD }}
        run: |
          # Store credentials temporarily
          xcrun notarytool store-credentials "ci-profile" \
            --apple-id "$APPLE_ID" \
            --team-id "$APPLE_TEAM_ID" \
            --password "$APPLE_APP_SPECIFIC_PASSWORD"
          
          # Submit for notarization
          xcrun notarytool submit HiDoc-P1-Monitor.dmg \
            --keychain-profile "ci-profile" \
            --wait
          
          # Staple ticket
          xcrun stapler staple HiDoc-P1-Monitor.dmg
      
      - name: Upload Release Asset
        uses: actions/upload-artifact@v3
        with:
          name: HiDoc-P1-Monitor-macOS
          path: HiDoc-P1-Monitor.dmg
```

---

## Windows Code Signing

### Windows Certificate Options

Windows code signing is **optional** but highly recommended to avoid SmartScreen warnings.

#### Certificate Providers

1. **DigiCert** - Industry standard, trusted by all platforms ($300-500/year)
2. **Sectigo (Comodo)** - Affordable option ($100-200/year)
3. **SSL.com** - Budget-friendly ($100-150/year)
4. **GlobalSign** - Enterprise option ($300-400/year)

#### Certificate Types

- **OV (Organization Validation)**: Requires business verification
- **EV (Extended Validation)**: Highest trust, requires hardware token, immediately trusted by SmartScreen

#### Self-Signed Certificates (Testing Only)

```powershell
# Create self-signed certificate (Windows PowerShell as Administrator)
$cert = New-SelfSignedCertificate `
  -Type CodeSigningCert `
  -Subject "CN=Your Company Name" `
  -KeyUsage DigitalSignature `
  -FriendlyName "Code Signing Certificate" `
  -CertStoreLocation "Cert:\CurrentUser\My" `
  -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3", "2.5.29.19={text}")

# Export certificate
$password = ConvertTo-SecureString -String "YourPassword" -Force -AsPlainText
Export-PfxCertificate -Cert $cert -FilePath "CodeSignCert.pfx" -Password $password

# Import to Trusted Root (required for self-signed)
Import-Certificate -FilePath "CodeSignCert.cer" -CertStoreLocation "Cert:\LocalMachine\Root"
```

**Note**: Self-signed certificates trigger security warnings. Only use for internal testing.

### Windows Signing Process

#### Using Tauri CLI (Recommended)

Tauri can automatically sign Windows binaries if a certificate is configured.

**tauri.conf.json snippet:**

```json
{
  "tauri": {
    "bundle": {
      "windows": {
        "certificateThumbprint": "ABCDEF1234567890...",
        "digestAlgorithm": "sha256",
        "timestampUrl": "http://timestamp.digicert.com"
      }
    }
  }
}
```

#### Manual Signing with signtool.exe

```powershell
# Sign EXE with PFX file
signtool sign /f "CodeSignCert.pfx" /p "YourPassword" /t http://timestamp.digicert.com /fd sha256 "HiDoc P1 Monitor.exe"

# Sign with certificate from Windows Certificate Store (by thumbprint)
signtool sign /sha1 ABCDEF1234567890... /t http://timestamp.digicert.com /fd sha256 "HiDoc P1 Monitor.exe"

# Sign with EV certificate (USB token)
signtool sign /n "Your Company Name" /t http://timestamp.digicert.com /fd sha256 "HiDoc P1 Monitor.exe"

# Verify signature
signtool verify /pa /v "HiDoc P1 Monitor.exe"
```

#### Timestamp Servers

Always timestamp your signatures (allows signature to remain valid after certificate expiration):

- DigiCert: `http://timestamp.digicert.com`
- Sectigo: `http://timestamp.sectigo.com`
- GlobalSign: `http://timestamp.globalsign.com`

#### Sign MSI Installer

```powershell
# Sign MSI package
signtool sign /f "CodeSignCert.pfx" /p "YourPassword" /t http://timestamp.digicert.com /fd sha256 "HiDoc P1 Monitor_x64.msi"

# Verify MSI signature
signtool verify /pa /v "HiDoc P1 Monitor_x64.msi"
```

### Windows CI/CD Configuration

#### GitHub Actions Secrets

- `WINDOWS_CERTIFICATE_BASE64`: Base64-encoded .pfx certificate
- `WINDOWS_CERTIFICATE_PASSWORD`: Certificate password

#### Export Certificate

```powershell
# Convert PFX to base64
$bytes = [System.IO.File]::ReadAllBytes("CodeSignCert.pfx")
[System.Convert]::ToBase64String($bytes) | Set-Clipboard

# Paste into GitHub Secrets as WINDOWS_CERTIFICATE_BASE64
```

#### GitHub Actions Workflow Example

```yaml
name: Release Windows

on:
  push:
    tags:
      - 'v*'

jobs:
  build-windows:
    runs-on: windows-latest
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install dependencies
        run: npm ci
      
      - name: Decode Certificate
        if: ${{ secrets.WINDOWS_CERTIFICATE_BASE64 != '' }}
        run: |
          $certBytes = [System.Convert]::FromBase64String("${{ secrets.WINDOWS_CERTIFICATE_BASE64 }}")
          [System.IO.File]::WriteAllBytes("$env:TEMP\cert.pfx", $certBytes)
        shell: pwsh
      
      - name: Build Tauri App
        run: npm run tauri build
      
      - name: Sign Executables
        if: ${{ secrets.WINDOWS_CERTIFICATE_BASE64 != '' }}
        env:
          CERT_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          $signtool = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64\signtool.exe"
          
          # Sign EXE
          & $signtool sign /f "$env:TEMP\cert.pfx" /p "$env:CERT_PASSWORD" /t http://timestamp.digicert.com /fd sha256 "src-tauri\target\release\HiDoc P1 Monitor.exe"
          
          # Sign MSI
          & $signtool sign /f "$env:TEMP\cert.pfx" /p "$env:CERT_PASSWORD" /t http://timestamp.digicert.com /fd sha256 "src-tauri\target\release\bundle\msi\HiDoc P1 Monitor_*_x64.msi"
          
          # Verify signatures
          & $signtool verify /pa /v "src-tauri\target\release\HiDoc P1 Monitor.exe"
        shell: pwsh
      
      - name: Upload Release Asset
        uses: actions/upload-artifact@v3
        with:
          name: HiDoc-P1-Monitor-Windows
          path: src-tauri/target/release/bundle/msi/*.msi
```

---

## Linux Code Signing

Code signing on Linux is **optional** but recommended for AppImages and package repositories.

### AppImage Signing

#### Prerequisites

```bash
# Install signing tools
sudo apt-get install gnupg2

# Generate GPG key (if you don't have one)
gpg --full-generate-key
# Select: (1) RSA and RSA
# Key size: 4096
# Expiration: 2y
# Name, email, passphrase
```

#### Sign AppImage

```bash
# Sign the AppImage
gpg --detach-sign --armor HiDoc-P1-Monitor.AppImage

# This creates HiDoc-P1-Monitor.AppImage.asc

# Verify signature
gpg --verify HiDoc-P1-Monitor.AppImage.asc HiDoc-P1-Monitor.AppImage

# Export public key for distribution
gpg --export --armor your@email.com > public-key.asc
```

#### Embed Signature in AppImage

```bash
# Install appimagetool
wget https://github.com/AppImage/AppImageKit/releases/download/continuous/appimagetool-x86_64.AppImage
chmod +x appimagetool-x86_64.AppImage

# Sign during AppImage creation
./appimagetool-x86_64.AppImage --sign \
  --sign-key YOUR_GPG_KEY_ID \
  HiDoc-P1-Monitor.AppDir \
  HiDoc-P1-Monitor.AppImage
```

### Package Repository Signing

#### Debian/Ubuntu (.deb)

```bash
# Sign .deb package
dpkg-sig --sign builder HiDoc-P1-Monitor_1.0.0_amd64.deb

# Verify signature
dpkg-sig --verify HiDoc-P1-Monitor_1.0.0_amd64.deb
```

#### RPM-based (.rpm)

```bash
# Sign RPM package
rpm --addsign HiDoc-P1-Monitor-1.0.0-1.x86_64.rpm

# Verify signature
rpm --checksig HiDoc-P1-Monitor-1.0.0-1.x86_64.rpm
```

#### Repository Signing (APT)

```bash
# Create repository
mkdir -p deb-repo/pool/main

# Copy .deb files
cp *.deb deb-repo/pool/main/

# Generate Packages file
cd deb-repo
dpkg-scanpackages pool/main /dev/null | gzip -9c > dists/stable/main/binary-amd64/Packages.gz

# Sign repository
gpg --clearsign -o dists/stable/InRelease dists/stable/Release

# Export public key for users
gpg --export --armor your@email.com > deb-repo/public-key.asc
```

Users can then add your repository:

```bash
# Add GPG key
curl -fsSL https://your-repo.com/public-key.asc | sudo gpg --dearmor -o /usr/share/keyrings/hidoc-archive-keyring.gpg

# Add repository
echo "deb [signed-by=/usr/share/keyrings/hidoc-archive-keyring.gpg] https://your-repo.com/deb-repo stable main" | sudo tee /etc/apt/sources.list.d/hidoc.list

# Install
sudo apt update
sudo apt install hidoc-p1-monitor
```

---

## Troubleshooting

### macOS Troubleshooting

#### Issue: "Developer ID Application" not found in keychain

**Solution:**

```bash
# List all identities
security find-identity -v -p codesigning

# If empty, certificate may be in a different keychain
security list-keychains

# Add login keychain
security list-keychains -d user -s login.keychain

# Re-import certificate
open DeveloperID_Application.cer
```

#### Issue: "errSecInternalComponent" during codesign

**Solution:**

```bash
# Unlock keychain
security unlock-keychain login.keychain

# Allow codesign access
security set-key-partition-list -S apple-tool:,apple: -s -k "keychain_password" login.keychain
```

#### Issue: Notarization fails with "Invalid Code Signature"

**Solution:**

```bash
# Ensure hardened runtime is enabled
codesign --sign "Developer ID Application: Your Name" \
  --force \
  --options runtime \
  --entitlements entitlements.plist \
  --timestamp \
  --deep \
  YourApp.app

# Verify runtime flag
codesign -dvvv YourApp.app 2>&1 | grep -i runtime
# Should show: flags=0x10000(runtime)
```

#### Issue: "The specified item could not be found in the keychain"

**Solution:**

```bash
# Verify certificate is valid (not expired)
security find-certificate -c "Developer ID Application" -p | openssl x509 -text | grep "Not After"

# If expired, renew certificate from developer.apple.com
```

#### Issue: Notarization stuck at "In Progress"

**Solution:**

```bash
# Usually completes in 1-5 minutes, but can take up to 1 hour
# Check status
xcrun notarytool info SUBMISSION_ID --keychain-profile "notarization-profile"

# If > 1 hour, resubmit
xcrun notarytool submit YourApp.dmg --keychain-profile "notarization-profile" --wait
```

#### Issue: "Application is damaged and can't be opened"

**Cause**: Quarantine attribute set by macOS Gatekeeper.

**Solution (for testing only):**

```bash
# Remove quarantine attribute
xattr -cr /Applications/YourApp.app

# Never distribute unsigned apps with this workaround
```

### Windows Troubleshooting

#### Issue: "signtool.exe not found"

**Solution:**

```powershell
# Install Windows SDK
# Download from: https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

# Or install via chocolatey
choco install windows-sdk-10.1

# Add to PATH (adjust version)
$env:Path += ";C:\Program Files (x86)\Windows Kits\10\bin\10.0.22621.0\x64"
```

#### Issue: "The specified certificate is not suitable for signing"

**Solution:**

```powershell
# Ensure certificate has "Code Signing" enhanced key usage
certutil -store My

# Look for "Enhanced Key Usage" section:
# Code Signing (1.3.6.1.5.5.7.3.3)
```

#### Issue: "Unable to use the certificate. The private key is not available"

**Solution:**

```powershell
# Ensure certificate was imported with private key
# Re-import PFX with "Mark this key as exportable" checked

# Verify private key exists
certutil -user -store My THUMBPRINT

# Should show: Private key is NOT exportable (or exportable)
```

#### Issue: SmartScreen warning even with valid signature

**Cause**: New certificates need to build reputation with Microsoft SmartScreen.

**Solution:**

- **EV Certificates**: Immediate reputation (recommended)
- **OV Certificates**: Requires 2-4 weeks of downloads before SmartScreen trusts
- **Workaround**: Submit your binary to Microsoft for analysis at [Microsoft Security Intelligence](https://www.microsoft.com/en-us/wdsi/filesubmission)

#### Issue: Timestamp server unavailable

**Solution:**

```powershell
# Retry with different timestamp server
# Try these in order:
signtool sign /f cert.pfx /p password /t http://timestamp.digicert.com /fd sha256 app.exe
signtool sign /f cert.pfx /p password /tr http://timestamp.sectigo.com /td sha256 /fd sha256 app.exe
signtool sign /f cert.pfx /p password /tr http://timestamp.globalsign.com/tsa/r6advanced1 /td sha256 /fd sha256 app.exe
```

### Linux Troubleshooting

#### Issue: GPG signing fails in CI

**Solution:**

```bash
# Export GPG key for CI
gpg --export-secret-keys --armor YOUR_KEY_ID > private-key.asc

# In CI, import key
echo "$GPG_PRIVATE_KEY" | gpg --import

# Configure git to use GPG key
git config --global user.signingkey YOUR_KEY_ID
```

#### Issue: AppImage signature verification fails

**Solution:**

```bash
# Ensure public key is imported
gpg --import public-key.asc

# Trust the key
gpg --edit-key your@email.com
# Type: trust
# Select: 5 (I trust ultimately)
# Type: quit

# Verify again
gpg --verify HiDoc-P1-Monitor.AppImage.asc HiDoc-P1-Monitor.AppImage
```

---

## Release Checklist

### Pre-Release

- [ ] Update version in `package.json`
- [ ] Update version in `src-tauri/Cargo.toml`
- [ ] Update `CHANGELOG.md` with release notes
- [ ] Test app on all platforms (macOS, Windows, Linux)
- [ ] Run security scan (`npm audit`, `cargo audit`)
- [ ] Update documentation if API changes

### Code Signing Preparation

#### macOS
- [ ] Verify Developer ID certificate is valid (not expired)
- [ ] Test signing locally: `codesign --verify --verbose=4 YourApp.app`
- [ ] Test notarization with `xcrun notarytool`
- [ ] Verify entitlements.plist includes necessary permissions

#### Windows
- [ ] Verify code signing certificate is valid
- [ ] Test signing locally: `signtool verify /pa /v YourApp.exe`
- [ ] Ensure timestamp server is accessible

#### Linux
- [ ] Verify GPG key is valid
- [ ] Test AppImage signature: `gpg --verify`

### Build & Sign

- [ ] Build for all platforms: `npm run tauri build`
- [ ] Sign macOS app and create DMG
- [ ] Notarize macOS DMG
- [ ] Staple notarization ticket to DMG
- [ ] Sign Windows EXE and MSI
- [ ] Sign Linux AppImage (optional)
- [ ] Generate checksums (SHA-256) for all artifacts

```bash
# Generate SHA-256 checksums
shasum -a 256 HiDoc-P1-Monitor.dmg > checksums.txt
certutil -hashfile HiDoc-P1-Monitor.msi SHA256 >> checksums.txt
sha256sum HiDoc-P1-Monitor.AppImage >> checksums.txt
```

### Verification

- [ ] Test install on clean macOS machine
- [ ] Test install on clean Windows machine
- [ ] Test install on clean Linux machine
- [ ] Verify no security warnings appear
- [ ] Test USB device detection on all platforms

### Distribution

- [ ] Create GitHub release with tag (e.g., `v1.0.0`)
- [ ] Upload signed binaries to GitHub release
- [ ] Upload checksums.txt
- [ ] Include release notes in GitHub release description
- [ ] Update website download links (if applicable)
- [ ] Announce release (Discord, Twitter, blog, etc.)

### Post-Release

- [ ] Monitor GitHub issues for installation problems
- [ ] Check crash reports (if telemetry enabled)
- [ ] Update project documentation if needed
- [ ] Plan next release milestones

---

## Additional Resources

### macOS
- [Apple Developer Documentation - Notarizing macOS Software](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Code Signing Guide](https://developer.apple.com/library/archive/documentation/Security/Conceptual/CodeSigningGuide/Introduction/Introduction.html)
- [Hardened Runtime](https://developer.apple.com/documentation/security/hardened_runtime)

### Windows
- [Microsoft Code Signing Overview](https://docs.microsoft.com/en-us/windows-hardware/drivers/dashboard/code-signing-attestation)
- [SignTool Documentation](https://docs.microsoft.com/en-us/windows/win32/seccrypto/signtool)

### Linux
- [AppImage Documentation](https://docs.appimage.org/)
- [Debian Package Signing](https://wiki.debian.org/SecureApt)
- [GPG Signing Guide](https://www.gnupg.org/gph/en/manual.html)

### Tauri
- [Tauri Code Signing Guide](https://tauri.app/v1/guides/distribution/sign-your-app)
- [Tauri Bundle Configuration](https://tauri.app/v1/api/config/#bundleconfig)

---

**Last Updated**: 2026-08-18
