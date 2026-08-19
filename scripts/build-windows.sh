#!/usr/bin/env bash

################################################################################
# Windows Build Script for HiNotes Desktop (Tauri Application)
################################################################################
#
# This script cross-compiles the HiNotes Desktop Tauri application for Windows
# from macOS or Linux. It produces MSI installers and optionally signs them.
#
# PREREQUISITES:
#   - Rust toolchain with windows target: rustup target add x86_64-pc-windows-msvc
#   - Cross-compilation tools: cargo install cargo-xwin
#   - Node.js and npm for frontend build
#   - WiX Toolset (for MSI creation - handled by Tauri)
#
# USAGE:
#   ./scripts/build-windows.sh [OPTIONS]
#
# OPTIONS:
#   --sign              Enable code signing (requires certificate)
#   --cert-path PATH    Path to code signing certificate (.pfx)
#   --cert-pass PASS    Certificate password (or set CERT_PASSWORD env var)
#   --clean             Clean build directories before building
#   --debug             Build in debug mode instead of release
#   --output-dir PATH   Custom output directory for artifacts (default: ./target/release/bundle)
#   --help              Show this help message
#
# ENVIRONMENT VARIABLES:
#   CERT_PASSWORD       Certificate password for code signing
#   TAURI_SIGNING_PRIVATE_KEY  Private key for Tauri updater signing
#
################################################################################

set -euo pipefail

# ============================================================================
# Configuration & Default Values
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BUILD_MODE="release"
ENABLE_SIGNING=false
CERT_PATH=""
CERT_PASSWORD="${CERT_PASSWORD:-}"
CLEAN_BUILD=false
OUTPUT_DIR=""
TIMESTAMP_URL="http://timestamp.digicert.com"
TARGET_TRIPLE="x86_64-pc-windows-msvc"

# Build artifacts
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_DIR="${PROJECT_ROOT}/build-logs"
LOG_FILE="${LOG_DIR}/windows-build-${TIMESTAMP}.log"

# ============================================================================
# Color output helpers
# ============================================================================

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $*" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "$LOG_FILE"
}

# ============================================================================
# Usage & Help
# ============================================================================

show_help() {
    sed -n '/^# USAGE:/,/^###/p' "$0" | sed 's/^# //; s/^#//'
    exit 0
}

# ============================================================================
# Parse Command Line Arguments
# ============================================================================

while [[ $# -gt 0 ]]; do
    case $1 in
        --sign)
            ENABLE_SIGNING=true
            shift
            ;;
        --cert-path)
            CERT_PATH="$2"
            shift 2
            ;;
        --cert-pass)
            CERT_PASSWORD="$2"
            shift 2
            ;;
        --clean)
            CLEAN_BUILD=true
            shift
            ;;
        --debug)
            BUILD_MODE="debug"
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --help)
            show_help
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            ;;
    esac
done

# ============================================================================
# Setup & Validation
# ============================================================================

setup_environment() {
    log_info "Setting up build environment..."

    # Create log directory
    mkdir -p "$LOG_DIR"

    # Change to project root
    cd "$PROJECT_ROOT"

    # Validate we're in a Tauri project
    if [[ ! -f "src-tauri/Cargo.toml" ]] || [[ ! -f "package.json" ]]; then
        log_error "Not a valid Tauri project directory"
        exit 1
    fi

    # Check for required tools
    log_info "Checking prerequisites..."

    if ! command -v rustc &> /dev/null; then
        log_error "Rust is not installed. Install from https://rustup.rs/"
        exit 1
    fi

    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is not installed. Install Rust toolchain from https://rustup.rs/"
        exit 1
    fi

    if ! command -v node &> /dev/null; then
        log_error "Node.js is not installed. Install from https://nodejs.org/"
        exit 1
    fi

    if ! command -v npm &> /dev/null; then
        log_error "npm is not installed. Install Node.js from https://nodejs.org/"
        exit 1
    fi

    # Check if Windows target is installed
    if ! rustup target list --installed | grep -q "$TARGET_TRIPLE"; then
        log_warning "Windows target not installed. Installing..."
        rustup target add "$TARGET_TRIPLE" 2>&1 | tee -a "$LOG_FILE"
    fi

    # Check for Tauri CLI
    if ! command -v cargo-tauri &> /dev/null && ! cargo tauri --version &> /dev/null; then
        log_warning "Tauri CLI not found. Installing via npm..."
        npm install -g @tauri-apps/cli 2>&1 | tee -a "$LOG_FILE"
    fi

    log_success "Prerequisites validated"
}

# ============================================================================
# Validate Signing Configuration
# ============================================================================

validate_signing() {
    if [[ "$ENABLE_SIGNING" == true ]]; then
        log_info "Validating code signing configuration..."

        if [[ -z "$CERT_PATH" ]]; then
            log_error "Code signing enabled but --cert-path not provided"
            exit 1
        fi

        if [[ ! -f "$CERT_PATH" ]]; then
            log_error "Certificate file not found: $CERT_PATH"
            exit 1
        fi

        if [[ -z "$CERT_PASSWORD" ]]; then
            log_error "Certificate password not provided. Use --cert-pass or set CERT_PASSWORD env var"
            exit 1
        fi

        # Check for signtool (usually requires Windows or Wine)
        if ! command -v osslsigncode &> /dev/null; then
            log_warning "osslsigncode not found. Code signing may not work."
            log_warning "Install with: brew install osslsigncode (macOS) or apt install osslsigncode (Linux)"
        fi

        log_success "Signing configuration validated"
    else
        log_info "Code signing disabled (use --sign to enable)"
    fi
}

# ============================================================================
# Clean Build Directories
# ============================================================================

clean_build_directories() {
    if [[ "$CLEAN_BUILD" == true ]]; then
        log_info "Cleaning build directories..."

        # Clean Rust build artifacts
        if [[ -d "src-tauri/target" ]]; then
            log_info "Removing src-tauri/target..."
            rm -rf src-tauri/target
        fi

        # Clean frontend build artifacts
        if [[ -d "dist" ]]; then
            log_info "Removing dist..."
            rm -rf dist
        fi

        # Clean node_modules (optional, commented out by default)
        # if [[ -d "node_modules" ]]; then
        #     log_info "Removing node_modules..."
        #     rm -rf node_modules
        # fi

        log_success "Build directories cleaned"
    fi
}

# ============================================================================
# Install Dependencies
# ============================================================================

install_dependencies() {
    log_info "Installing Node.js dependencies..."
    npm install 2>&1 | tee -a "$LOG_FILE"

    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        log_error "Failed to install Node.js dependencies"
        exit 1
    fi

    log_success "Dependencies installed"
}

# ============================================================================
# Build Frontend
# ============================================================================

build_frontend() {
    log_info "Building frontend (TypeScript + Vite)..."

    npm run build 2>&1 | tee -a "$LOG_FILE"

    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        log_error "Frontend build failed"
        exit 1
    fi

    # Verify dist directory was created
    if [[ ! -d "dist" ]]; then
        log_error "Frontend build succeeded but dist directory not found"
        exit 1
    fi

    log_success "Frontend build completed"
}

# ============================================================================
# Configure Tauri for Windows
# ============================================================================

configure_tauri() {
    log_info "Configuring Tauri for Windows build..."

    local tauri_conf="src-tauri/tauri.conf.json"

    # Backup original config
    cp "$tauri_conf" "${tauri_conf}.bak"

    # Update bundle targets to only include Windows formats
    if command -v jq &> /dev/null; then
        jq '.bundle.targets = ["msi", "nsis"]' "$tauri_conf" > "${tauri_conf}.tmp" && mv "${tauri_conf}.tmp" "$tauri_conf"
    else
        log_warning "jq not found, skipping bundle target configuration"
    fi

    # Set signing configuration if enabled
    if [[ "$ENABLE_SIGNING" == true ]]; then
        if command -v jq &> /dev/null; then
            jq --arg thumbprint "$(get_cert_thumbprint)" \
               --arg timestamp "$TIMESTAMP_URL" \
               '.bundle.windows.certificateThumbprint = $thumbprint |
                .bundle.windows.timestampUrl = $timestamp' \
               "$tauri_conf" > "${tauri_conf}.tmp" && mv "${tauri_conf}.tmp" "$tauri_conf"
        fi
    fi

    log_success "Tauri configuration updated"
}

# ============================================================================
# Get Certificate Thumbprint (if signing)
# ============================================================================

get_cert_thumbprint() {
    if [[ -n "$CERT_PATH" ]] && command -v openssl &> /dev/null; then
        openssl pkcs12 -in "$CERT_PATH" -passin pass:"$CERT_PASSWORD" -nokeys -nodes 2>/dev/null | \
            openssl x509 -noout -fingerprint -sha1 | \
            cut -d= -f2 | \
            tr -d ':'
    else
        echo ""
    fi
}

# ============================================================================
# Build Tauri Application for Windows
# ============================================================================

build_tauri() {
    log_info "Building Tauri application for Windows ($BUILD_MODE mode)..."

    local build_cmd="cargo tauri build --target $TARGET_TRIPLE"

    if [[ "$BUILD_MODE" == "debug" ]]; then
        build_cmd="$build_cmd --debug"
    fi

    # Set environment variables for build
    export TAURI_SKIP_UPDATE_CHECK=true

    log_info "Running: $build_cmd"

    # Execute build
    eval "$build_cmd" 2>&1 | tee -a "$LOG_FILE"

    if [[ ${PIPESTATUS[0]} -ne 0 ]]; then
        log_error "Tauri build failed. Check log: $LOG_FILE"
        restore_tauri_config
        exit 1
    fi

    log_success "Tauri build completed"
}

# ============================================================================
# Sign Windows Binaries
# ============================================================================

sign_binaries() {
    if [[ "$ENABLE_SIGNING" == false ]]; then
        log_info "Skipping code signing (disabled)"
        return 0
    fi

    log_info "Signing Windows binaries..."

    local bundle_dir="src-tauri/target/${TARGET_TRIPLE}/${BUILD_MODE}/bundle"

    # Find MSI installers
    local msi_files
    mapfile -t msi_files < <(find "$bundle_dir" -name "*.msi" -type f)

    if [[ ${#msi_files[@]} -eq 0 ]]; then
        log_warning "No MSI files found to sign"
        return 0
    fi

    for msi_file in "${msi_files[@]}"; do
        log_info "Signing: $(basename "$msi_file")"

        if command -v osslsigncode &> /dev/null; then
            osslsigncode sign \
                -pkcs12 "$CERT_PATH" \
                -pass "$CERT_PASSWORD" \
                -n "HiNotes Desktop" \
                -i "https://github.com/yourusername/hidoc" \
                -t "$TIMESTAMP_URL" \
                -in "$msi_file" \
                -out "${msi_file}.signed" 2>&1 | tee -a "$LOG_FILE"

            if [[ ${PIPESTATUS[0]} -eq 0 ]]; then
                mv "${msi_file}.signed" "$msi_file"
                log_success "Signed: $(basename "$msi_file")"
            else
                log_error "Failed to sign: $(basename "$msi_file")"
                rm -f "${msi_file}.signed"
            fi
        else
            log_error "osslsigncode not found. Cannot sign binaries."
            return 1
        fi
    done

    log_success "Code signing completed"
}

# ============================================================================
# Copy Artifacts to Output Directory
# ============================================================================

copy_artifacts() {
    log_info "Copying build artifacts..."

    local bundle_dir="src-tauri/target/${TARGET_TRIPLE}/${BUILD_MODE}/bundle"
    local output_path="${OUTPUT_DIR:-${PROJECT_ROOT}/dist-windows}"

    # Create output directory
    mkdir -p "$output_path"

    # Find and copy all installer artifacts
    local artifact_count=0

    # MSI installers
    while IFS= read -r -d '' file; do
        cp "$file" "$output_path/"
        log_info "Copied: $(basename "$file")"
        ((artifact_count++))
    done < <(find "$bundle_dir" -name "*.msi" -type f -print0 2>/dev/null || true)

    # NSIS installers (if any)
    while IFS= read -r -d '' file; do
        cp "$file" "$output_path/"
        log_info "Copied: $(basename "$file")"
        ((artifact_count++))
    done < <(find "$bundle_dir" -name "*.exe" -type f -print0 2>/dev/null || true)

    # Updater artifacts (JSON + signature)
    while IFS= read -r -d '' file; do
        cp "$file" "$output_path/"
        log_info "Copied: $(basename "$file")"
        ((artifact_count++))
    done < <(find "$bundle_dir" \( -name "*.json" -o -name "*.sig" \) -type f -print0 2>/dev/null || true)

    if [[ $artifact_count -eq 0 ]]; then
        log_error "No artifacts found in: $bundle_dir"
        log_error "Build may have failed. Check: $LOG_FILE"
        return 1
    fi

    log_success "Copied $artifact_count artifact(s) to: $output_path"
    echo "$output_path" > "${PROJECT_ROOT}/.last-windows-build-output"
}

# ============================================================================
# Restore Tauri Configuration
# ============================================================================

restore_tauri_config() {
    local tauri_conf="src-tauri/tauri.conf.json"
    if [[ -f "${tauri_conf}.bak" ]]; then
        mv "${tauri_conf}.bak" "$tauri_conf"
        log_info "Restored original tauri.conf.json"
    fi
}

# ============================================================================
# Generate Build Report
# ============================================================================

generate_build_report() {
    log_info "Generating build report..."

    local output_path="${OUTPUT_DIR:-${PROJECT_ROOT}/dist-windows}"
    local report_file="${output_path}/BUILD_REPORT.txt"

    {
        echo "=============================================="
        echo "HiNotes Desktop - Windows Build Report"
        echo "=============================================="
        echo ""
        echo "Build Date: $(date)"
        echo "Build Mode: $BUILD_MODE"
        echo "Target: $TARGET_TRIPLE"
        echo "Code Signing: $ENABLE_SIGNING"
        echo ""
        echo "----------------------------------------------"
        echo "Build Artifacts:"
        echo "----------------------------------------------"

        if [[ -d "$output_path" ]]; then
            ls -lh "$output_path"/*.{msi,exe,json,sig} 2>/dev/null || echo "No artifacts found"
        fi

        echo ""
        echo "----------------------------------------------"
        echo "File Hashes (SHA256):"
        echo "----------------------------------------------"

        if command -v shasum &> /dev/null; then
            cd "$output_path"
            shasum -a 256 *.{msi,exe} 2>/dev/null || echo "No files to hash"
            cd - > /dev/null
        fi

        echo ""
        echo "----------------------------------------------"
        echo "Installation Instructions:"
        echo "----------------------------------------------"
        echo "1. Download the .msi or .exe installer"
        echo "2. Run the installer (may require admin privileges)"
        echo "3. Follow the on-screen instructions"
        echo "4. Launch HiNotes Desktop from Start Menu"
        echo ""
        echo "For silent installation (MSI):"
        echo "  msiexec /i HiNotes-Desktop.msi /quiet /qn"
        echo ""
        echo "----------------------------------------------"
        echo "System Requirements:"
        echo "----------------------------------------------"
        echo "- Windows 10 (1809+) or Windows 11"
        echo "- x64 processor"
        echo "- 200 MB disk space"
        echo "- WebView2 runtime (auto-installed if missing)"
        echo ""
        echo "=============================================="

    } > "$report_file"

    log_success "Build report generated: $report_file"
}

# ============================================================================
# Cleanup
# ============================================================================

cleanup() {
    log_info "Cleaning up..."
    restore_tauri_config
}

# ============================================================================
# Main Build Flow
# ============================================================================

main() {
    log_info "Starting Windows build for HiNotes Desktop"
    log_info "Build log: $LOG_FILE"

    # Setup trap for cleanup on exit
    trap cleanup EXIT

    # Execute build steps
    setup_environment
    validate_signing
    clean_build_directories
    install_dependencies
    build_frontend
    configure_tauri
    build_tauri
    sign_binaries
    copy_artifacts
    generate_build_report

    # Final output
    local output_path="${OUTPUT_DIR:-${PROJECT_ROOT}/dist-windows}"

    echo ""
    log_success "╔════════════════════════════════════════════════════════════╗"
    log_success "║   Windows Build Completed Successfully!                   ║"
    log_success "╚════════════════════════════════════════════════════════════╝"
    echo ""
    log_info "Build artifacts location:"
    log_info "  $output_path"
    echo ""
    log_info "Next steps:"
    log_info "  1. Test the installer on a Windows machine"
    log_info "  2. Verify USB device detection works"
    log_info "  3. Upload artifacts to release management"
    echo ""
    log_info "Full build log: $LOG_FILE"
}

# ============================================================================
# Execute
# ============================================================================

main "$@"
