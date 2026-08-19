#!/usr/bin/env bash

################################################################################
# macOS Build Script for HiNotes Desktop (Tauri App)
################################################################################
#
# This script builds a universal macOS application bundle (.app) and DMG
# installer for the HiNotes Desktop Tauri application.
#
# Features:
# - Universal binary (x86_64 + aarch64)
# - DMG creation with proper layout
# - Code signing (if certificates available)
# - Notarization for Gatekeeper (if credentials available)
# - Comprehensive error handling and logging
#
# Requirements:
# - Xcode Command Line Tools
# - Rust toolchain with aarch64 and x86_64 targets
# - Node.js and npm
# - Tauri CLI (@tauri-apps/cli)
#
# Optional (for signing/notarization):
# - Apple Developer certificate in Keychain
# - APPLE_ID environment variable
# - APPLE_ID_PASSWORD environment variable (app-specific password)
# - APPLE_TEAM_ID environment variable
#
# Usage:
#   ./scripts/build-mac.sh [--sign] [--notarize] [--universal] [--clean]
#
################################################################################

set -euo pipefail

################################################################################
# Configuration
################################################################################

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_DIR="${PROJECT_ROOT}/target/release"
BUNDLE_DIR="${BUILD_DIR}/bundle"
DIST_DIR="${PROJECT_ROOT}/dist-artifacts"
LOG_FILE="${PROJECT_ROOT}/build-mac.log"

# Application details (read from package.json)
APP_NAME="HiNotes Desktop"
APP_IDENTIFIER="com.sarman.hinotes-desktop"

# Build options (can be overridden by command line args)
DO_SIGN=false
DO_NOTARIZE=false
BUILD_UNIVERSAL=true
DO_CLEAN=false
VERBOSE=false

################################################################################
# Color output
################################################################################

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

################################################################################
# Logging functions
################################################################################

log() {
    local level="$1"
    shift
    local message="$*"
    local timestamp
    timestamp="$(date '+%Y-%m-%d %H:%M:%S')"

    echo "[${timestamp}] [${level}] ${message}" | tee -a "${LOG_FILE}"
}

log_info() {
    echo -e "${BLUE}ℹ ${NC}$*" | tee -a "${LOG_FILE}"
}

log_success() {
    echo -e "${GREEN}✓${NC} $*" | tee -a "${LOG_FILE}"
}

log_warning() {
    echo -e "${YELLOW}⚠${NC} $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo -e "${RED}✗${NC} $*" | tee -a "${LOG_FILE}"
}

################################################################################
# Error handling
################################################################################

error_exit() {
    log_error "$1"
    log_error "Build failed! Check ${LOG_FILE} for details."
    exit 1
}

cleanup_on_error() {
    log_warning "Build interrupted. Cleaning up..."
    # Add any cleanup tasks here if needed
    exit 1
}

trap cleanup_on_error INT TERM

################################################################################
# Dependency checks
################################################################################

check_dependencies() {
    log_info "Checking build dependencies..."

    # Check for required commands
    local deps=("rustc" "cargo" "node" "npm")
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            error_exit "Required dependency '${dep}' not found in PATH"
        fi
    done

    # Check Rust targets for universal binary
    if [ "$BUILD_UNIVERSAL" = true ]; then
        log_info "Checking Rust targets for universal binary..."

        if ! rustup target list --installed | grep -q "aarch64-apple-darwin"; then
            log_info "Installing aarch64-apple-darwin target..."
            rustup target add aarch64-apple-darwin || error_exit "Failed to add aarch64 target"
        fi

        if ! rustup target list --installed | grep -q "x86_64-apple-darwin"; then
            log_info "Installing x86_64-apple-darwin target..."
            rustup target add x86_64-apple-darwin || error_exit "Failed to add x86_64 target"
        fi
    fi

    # Check Tauri CLI
    if ! command -v cargo-tauri &> /dev/null; then
        log_info "Tauri CLI not found, checking npm installation..."
        if ! npm list -g @tauri-apps/cli &> /dev/null; then
            error_exit "Tauri CLI not found. Install with: npm install -g @tauri-apps/cli"
        fi
    fi

    # Check for signing requirements
    if [ "$DO_SIGN" = true ]; then
        log_info "Checking code signing requirements..."

        if ! command -v codesign &> /dev/null; then
            error_exit "codesign command not found. Install Xcode Command Line Tools."
        fi

        # Check for valid signing identity
        local signing_identities
        signing_identities=$(security find-identity -v -p codesigning 2>/dev/null | grep "Developer ID Application" || true)

        if [ -z "$signing_identities" ]; then
            log_warning "No 'Developer ID Application' certificate found in Keychain."
            log_warning "Continuing without code signing. Build will not be notarized."
            DO_SIGN=false
            DO_NOTARIZE=false
        else
            log_success "Found code signing certificate(s):"
            echo "$signing_identities" | tee -a "${LOG_FILE}"
        fi
    fi

    # Check for notarization requirements
    if [ "$DO_NOTARIZE" = true ]; then
        if [ "$DO_SIGN" != true ]; then
            log_warning "Notarization requires code signing. Enabling signing."
            DO_SIGN=true
        fi

        if [ -z "${APPLE_ID:-}" ]; then
            log_warning "APPLE_ID not set. Skipping notarization."
            DO_NOTARIZE=false
        fi

        if [ -z "${APPLE_ID_PASSWORD:-}" ]; then
            log_warning "APPLE_ID_PASSWORD not set. Skipping notarization."
            DO_NOTARIZE=false
        fi

        if [ -z "${APPLE_TEAM_ID:-}" ]; then
            log_warning "APPLE_TEAM_ID not set. Skipping notarization."
            DO_NOTARIZE=false
        fi

        if [ "$DO_NOTARIZE" = true ]; then
            if ! command -v xcrun &> /dev/null; then
                error_exit "xcrun command not found. Install Xcode Command Line Tools."
            fi
        fi
    fi

    log_success "All dependencies satisfied"
}

################################################################################
# Clean build artifacts
################################################################################

clean_build() {
    if [ "$DO_CLEAN" = true ]; then
        log_info "Cleaning previous build artifacts..."

        cd "${PROJECT_ROOT}"

        # Clean frontend
        if [ -d "dist" ]; then
            rm -rf dist
            log_success "Removed frontend dist directory"
        fi

        # Clean Rust target
        if [ -d "target" ]; then
            cargo clean 2>&1 | tee -a "${LOG_FILE}" || log_warning "cargo clean failed"
            log_success "Cleaned Rust target directory"
        fi

        # Clean node_modules (optional, uncomment if needed)
        # if [ -d "node_modules" ]; then
        #     rm -rf node_modules
        #     log_success "Removed node_modules"
        # fi
    fi
}

################################################################################
# Install dependencies
################################################################################

install_dependencies() {
    log_info "Installing frontend dependencies..."
    cd "${PROJECT_ROOT}"

    npm ci 2>&1 | tee -a "${LOG_FILE}" || error_exit "npm ci failed"

    log_success "Frontend dependencies installed"
}

################################################################################
# Build frontend
################################################################################

build_frontend() {
    log_info "Building frontend (TypeScript + Vite)..."
    cd "${PROJECT_ROOT}"

    npm run build 2>&1 | tee -a "${LOG_FILE}" || error_exit "Frontend build failed"

    if [ ! -d "dist" ]; then
        error_exit "Frontend dist directory not created"
    fi

    log_success "Frontend built successfully"
}

################################################################################
# Build Tauri application
################################################################################

build_tauri() {
    log_info "Building Tauri application..."
    cd "${PROJECT_ROOT}"

    local build_args=()

    if [ "$BUILD_UNIVERSAL" = true ]; then
        log_info "Building universal binary (x86_64 + aarch64)..."
        build_args+=("--target" "universal-apple-darwin")
    fi

    if [ "$VERBOSE" = true ]; then
        build_args+=("--verbose")
    fi

    # Build using Tauri CLI
    if command -v cargo-tauri &> /dev/null; then
        cargo tauri build "${build_args[@]}" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Tauri build failed"
    else
        npm run tauri build -- "${build_args[@]}" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Tauri build failed"
    fi

    log_success "Tauri application built successfully"
}

################################################################################
# Sign application bundle
################################################################################

sign_app() {
    if [ "$DO_SIGN" != true ]; then
        log_info "Skipping code signing (not requested)"
        return 0
    fi

    log_info "Code signing application bundle..."

    # Find the .app bundle
    local app_bundle
    if [ "$BUILD_UNIVERSAL" = true ]; then
        app_bundle="${BUNDLE_DIR}/macos/${APP_NAME}.app"
    else
        app_bundle="${BUNDLE_DIR}/macos/${APP_NAME}.app"
    fi

    if [ ! -d "$app_bundle" ]; then
        error_exit "Application bundle not found at: ${app_bundle}"
    fi

    # Get signing identity
    local signing_identity
    signing_identity=$(security find-identity -v -p codesigning | grep "Developer ID Application" | head -n 1 | awk '{print $2}' || true)

    if [ -z "$signing_identity" ]; then
        error_exit "No Developer ID Application certificate found"
    fi

    log_info "Using signing identity: ${signing_identity}"

    # Sign with hardened runtime and secure timestamp
    codesign --force \
        --options runtime \
        --sign "${signing_identity}" \
        --timestamp \
        --deep \
        --verbose \
        "$app_bundle" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Code signing failed"

    # Verify signature
    log_info "Verifying code signature..."
    codesign --verify --deep --strict --verbose=2 "$app_bundle" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Code signature verification failed"

    log_success "Application signed successfully"
}

################################################################################
# Create DMG installer
################################################################################

create_dmg() {
    log_info "DMG creation handled by Tauri bundler..."

    local dmg_path
    if [ "$BUILD_UNIVERSAL" = true ]; then
        dmg_path="${BUNDLE_DIR}/dmg/${APP_NAME}_*_universal.dmg"
    else
        dmg_path="${BUNDLE_DIR}/dmg/${APP_NAME}_*.dmg"
    fi

    # Find the generated DMG
    local dmg_file
    dmg_file=$(find "${BUNDLE_DIR}/dmg" -name "*.dmg" -type f 2>/dev/null | head -n 1 || true)

    if [ -z "$dmg_file" ]; then
        log_warning "DMG not found in expected location. Checking alternative paths..."
        dmg_file=$(find "${PROJECT_ROOT}/target" -name "*.dmg" -type f 2>/dev/null | head -n 1 || true)
    fi

    if [ -n "$dmg_file" ]; then
        log_success "DMG created: ${dmg_file}"
        echo "DMG_PATH=${dmg_file}" >> "${LOG_FILE}"
    else
        log_warning "DMG file not found. May have been created in non-standard location."
    fi
}

################################################################################
# Notarize application
################################################################################

notarize_app() {
    if [ "$DO_NOTARIZE" != true ]; then
        log_info "Skipping notarization (not requested)"
        return 0
    fi

    log_info "Notarizing application with Apple..."

    # Find the DMG to notarize
    local dmg_file
    dmg_file=$(find "${BUNDLE_DIR}/dmg" -name "*.dmg" -type f 2>/dev/null | head -n 1 || true)

    if [ -z "$dmg_file" ]; then
        dmg_file=$(find "${PROJECT_ROOT}/target" -name "*.dmg" -type f 2>/dev/null | head -n 1 || true)
    fi

    if [ -z "$dmg_file" ]; then
        error_exit "Cannot find DMG file to notarize"
    fi

    log_info "Submitting ${dmg_file} for notarization..."

    # Submit for notarization (using notarytool, available in Xcode 13+)
    local request_uuid
    request_uuid=$(xcrun notarytool submit "$dmg_file" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_ID_PASSWORD}" \
        --team-id "${APPLE_TEAM_ID}" \
        --wait \
        2>&1 | tee -a "${LOG_FILE}" | grep "id:" | awk '{print $2}' || true)

    if [ -z "$request_uuid" ]; then
        error_exit "Notarization submission failed"
    fi

    log_info "Notarization request ID: ${request_uuid}"

    # Check notarization status
    log_info "Waiting for notarization to complete..."

    xcrun notarytool wait "$request_uuid" \
        --apple-id "${APPLE_ID}" \
        --password "${APPLE_ID_PASSWORD}" \
        --team-id "${APPLE_TEAM_ID}" \
        2>&1 | tee -a "${LOG_FILE}" || error_exit "Notarization failed"

    # Staple the notarization ticket
    log_info "Stapling notarization ticket to DMG..."
    xcrun stapler staple "$dmg_file" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Stapling failed"

    # Verify stapling
    log_info "Verifying stapled ticket..."
    xcrun stapler validate "$dmg_file" 2>&1 | tee -a "${LOG_FILE}" || error_exit "Staple verification failed"

    log_success "Application notarized successfully"
}

################################################################################
# Copy artifacts to distribution directory
################################################################################

copy_artifacts() {
    log_info "Copying build artifacts to ${DIST_DIR}..."

    mkdir -p "${DIST_DIR}"

    # Copy DMG
    local dmg_file
    dmg_file=$(find "${PROJECT_ROOT}/target" -name "*.dmg" -type f 2>/dev/null | head -n 1 || true)

    if [ -n "$dmg_file" ]; then
        cp "$dmg_file" "${DIST_DIR}/"
        log_success "Copied DMG: $(basename "$dmg_file")"
    fi

    # Copy .app bundle
    local app_bundle
    app_bundle=$(find "${PROJECT_ROOT}/target" -name "${APP_NAME}.app" -type d 2>/dev/null | head -n 1 || true)

    if [ -n "$app_bundle" ]; then
        cp -R "$app_bundle" "${DIST_DIR}/"
        log_success "Copied app bundle: $(basename "$app_bundle")"
    fi

    # Copy updater artifacts if they exist
    local updater_json
    updater_json=$(find "${PROJECT_ROOT}/target" -name "latest.json" -type f 2>/dev/null | head -n 1 || true)

    if [ -n "$updater_json" ]; then
        cp "$updater_json" "${DIST_DIR}/"
        log_success "Copied updater manifest: latest.json"
    fi

    # List all artifacts
    log_info "Build artifacts:"
    ls -lh "${DIST_DIR}" | tee -a "${LOG_FILE}"
}

################################################################################
# Generate build summary
################################################################################

generate_summary() {
    log_info "Build Summary"
    echo "========================================" | tee -a "${LOG_FILE}"
    echo "Build Date: $(date)" | tee -a "${LOG_FILE}"
    echo "Build Type: $([ "$BUILD_UNIVERSAL" = true ] && echo "Universal (x86_64 + aarch64)" || echo "Native")" | tee -a "${LOG_FILE}"
    echo "Code Signed: $([ "$DO_SIGN" = true ] && echo "Yes" || echo "No")" | tee -a "${LOG_FILE}"
    echo "Notarized: $([ "$DO_NOTARIZE" = true ] && echo "Yes" || echo "No")" | tee -a "${LOG_FILE}"
    echo "Artifacts Directory: ${DIST_DIR}" | tee -a "${LOG_FILE}"

    # List artifacts with sizes
    if [ -d "${DIST_DIR}" ]; then
        echo "" | tee -a "${LOG_FILE}"
        echo "Artifacts:" | tee -a "${LOG_FILE}"
        find "${DIST_DIR}" -type f -exec ls -lh {} \; | awk '{print "  " $9 " (" $5 ")"}' | tee -a "${LOG_FILE}"
    fi

    echo "========================================" | tee -a "${LOG_FILE}"
    echo "" | tee -a "${LOG_FILE}"

    log_success "Build completed successfully!"
    echo "" | tee -a "${LOG_FILE}"
    echo "Artifacts available in: ${DIST_DIR}" | tee -a "${LOG_FILE}"
    echo "Build log available at: ${LOG_FILE}" | tee -a "${LOG_FILE}"
}

################################################################################
# Parse command line arguments
################################################################################

parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --sign)
                DO_SIGN=true
                shift
                ;;
            --notarize)
                DO_NOTARIZE=true
                DO_SIGN=true  # Notarization requires signing
                shift
                ;;
            --universal)
                BUILD_UNIVERSAL=true
                shift
                ;;
            --no-universal)
                BUILD_UNIVERSAL=false
                shift
                ;;
            --clean)
                DO_CLEAN=true
                shift
                ;;
            --verbose)
                VERBOSE=true
                shift
                ;;
            --help|-h)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Build HiNotes Desktop for macOS"
                echo ""
                echo "OPTIONS:"
                echo "  --sign          Code sign the application (requires certificate)"
                echo "  --notarize      Notarize the application (implies --sign, requires credentials)"
                echo "  --universal     Build universal binary for x86_64 + aarch64 (default)"
                echo "  --no-universal  Build for current architecture only"
                echo "  --clean         Clean build artifacts before building"
                echo "  --verbose       Enable verbose output"
                echo "  --help, -h      Show this help message"
                echo ""
                echo "ENVIRONMENT VARIABLES (for notarization):"
                echo "  APPLE_ID            Apple ID email"
                echo "  APPLE_ID_PASSWORD   App-specific password"
                echo "  APPLE_TEAM_ID       Apple Developer Team ID"
                echo ""
                exit 0
                ;;
            *)
                error_exit "Unknown option: $1. Use --help for usage information."
                ;;
        esac
    done
}

################################################################################
# Main execution
################################################################################

main() {
    echo "╔════════════════════════════════════════════════════════════════╗"
    echo "║         HiNotes Desktop - macOS Build Script                  ║"
    echo "╚════════════════════════════════════════════════════════════════╝"
    echo ""

    # Initialize log file
    echo "Build started at $(date)" > "${LOG_FILE}"
    echo "========================================" >> "${LOG_FILE}"

    # Parse command line arguments
    parse_args "$@"

    # Display build configuration
    log_info "Build Configuration:"
    echo "  Project: ${APP_NAME}" | tee -a "${LOG_FILE}"
    echo "  Identifier: ${APP_IDENTIFIER}" | tee -a "${LOG_FILE}"
    echo "  Universal Binary: ${BUILD_UNIVERSAL}" | tee -a "${LOG_FILE}"
    echo "  Code Signing: ${DO_SIGN}" | tee -a "${LOG_FILE}"
    echo "  Notarization: ${DO_NOTARIZE}" | tee -a "${LOG_FILE}"
    echo "  Clean Build: ${DO_CLEAN}" | tee -a "${LOG_FILE}"
    echo "" | tee -a "${LOG_FILE}"

    # Execute build steps
    check_dependencies
    clean_build
    install_dependencies
    build_frontend
    build_tauri
    sign_app
    create_dmg
    notarize_app
    copy_artifacts
    generate_summary
}

# Run main function with all arguments
main "$@"
