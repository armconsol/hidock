#!/usr/bin/env bash

################################################################################
# HiNotes Desktop - Linux Build Script
#
# Builds x86_64 Linux packages (.deb, .AppImage, .rpm) for HiNotes Desktop
# Tauri application with USB device support.
#
# Requirements:
#   - Rust toolchain (stable)
#   - Node.js 18+
#   - Tauri CLI dependencies
#   - Package-specific tools (dpkg, appimagetool, rpmbuild)
#
# Usage:
#   ./scripts/build-linux.sh [OPTIONS]
#
# Options:
#   --release          Build in release mode (default)
#   --debug            Build in debug mode
#   --deb-only         Build .deb only
#   --appimage-only    Build .AppImage only
#   --rpm-only         Build .rpm only
#   --skip-deps        Skip dependency checks
#   --verbose          Enable verbose output
#   --clean            Clean build artifacts before building
#   --help             Show this help message
#
# Output:
#   Artifacts are generated in:
#   - src-tauri/target/release/bundle/deb/
#   - src-tauri/target/release/bundle/appimage/
#   - src-tauri/target/release/bundle/rpm/
#
# Exit codes:
#   0 - Success
#   1 - General error
#   2 - Dependency check failed
#   3 - Build failed
#   4 - Test failed
################################################################################

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
BUILD_MODE="release"
VERBOSE=false
SKIP_DEPS=false
CLEAN_BUILD=false

# Build target flags
BUILD_DEB=true
BUILD_APPIMAGE=true
BUILD_RPM=true

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Logging
LOG_FILE="${PROJECT_ROOT}/build-linux.log"
START_TIME=$(date +%s)

# ============================================================================
# Logging Functions
# ============================================================================

log_info() {
    echo -e "${BLUE}[INFO]${NC} $*" | tee -a "${LOG_FILE}"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $*" | tee -a "${LOG_FILE}"
}

log_warn() {
    echo -e "${YELLOW}[WARNING]${NC} $*" | tee -a "${LOG_FILE}"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $*" | tee -a "${LOG_FILE}"
}

log_section() {
    echo ""
    echo -e "${BLUE}============================================${NC}" | tee -a "${LOG_FILE}"
    echo -e "${BLUE}  $*${NC}" | tee -a "${LOG_FILE}"
    echo -e "${BLUE}============================================${NC}" | tee -a "${LOG_FILE}"
}

# ============================================================================
# Helper Functions
# ============================================================================

show_help() {
    grep '^#' "${BASH_SOURCE[0]}" | grep -v '#!/usr/bin/env' | sed 's/^# \?//'
    exit 0
}

check_command() {
    local cmd=$1
    local package=${2:-$1}
    if ! command -v "${cmd}" &> /dev/null; then
        log_error "Required command '${cmd}' not found. Install: ${package}"
        return 1
    fi
    return 0
}

get_version() {
    grep '"version":' "${PROJECT_ROOT}/package.json" | head -1 | sed 's/.*"version": "\(.*\)".*/\1/'
}

get_architecture() {
    uname -m
}

get_os_info() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "${NAME} ${VERSION_ID:-}"
    else
        echo "Unknown Linux"
    fi
}

cleanup_artifacts() {
    log_info "Cleaning previous build artifacts..."
    rm -rf "${PROJECT_ROOT}/src-tauri/target/release/bundle"
    rm -rf "${PROJECT_ROOT}/dist"
    rm -rf "${PROJECT_ROOT}/node_modules/.vite"
    log_success "Cleanup completed"
}

# ============================================================================
# Dependency Checks
# ============================================================================

check_system_dependencies() {
    log_section "Checking System Dependencies"

    local missing_deps=()
    local os_info
    os_info=$(get_os_info)
    log_info "OS: ${os_info}"
    log_info "Architecture: $(get_architecture)"

    # Core build tools
    check_command "cargo" "rust (https://rustup.rs)" || missing_deps+=("cargo")
    check_command "node" "nodejs" || missing_deps+=("node")
    check_command "npm" "npm" || missing_deps+=("npm")
    check_command "pkg-config" "pkg-config" || missing_deps+=("pkg-config")
    check_command "gcc" "build-essential" || missing_deps+=("gcc")

    # Tauri dependencies
    if ! pkg-config --exists gtk+-3.0; then
        log_error "GTK3 development files not found"
        missing_deps+=("libgtk-3-dev")
    fi

    if ! pkg-config --exists webkit2gtk-4.1; then
        log_error "WebKit2GTK development files not found"
        missing_deps+=("libwebkit2gtk-4.1-dev")
    fi

    if ! pkg-config --exists libusb-1.0; then
        log_error "libusb development files not found"
        missing_deps+=("libusb-1.0-0-dev")
    fi

    # Package-specific tools
    if [ "${BUILD_DEB}" = true ]; then
        check_command "dpkg-deb" "dpkg" || log_warn ".deb building may fail"
    fi

    if [ "${BUILD_RPM}" = true ]; then
        check_command "rpmbuild" "rpm" || log_warn ".rpm building may fail"
    fi

    # Check Rust target
    if ! rustup target list --installed | grep -q "x86_64-unknown-linux-gnu"; then
        log_warn "x86_64-unknown-linux-gnu target not installed"
        log_info "Installing Rust target..."
        rustup target add x86_64-unknown-linux-gnu || missing_deps+=("rust-target")
    fi

    if [ ${#missing_deps[@]} -gt 0 ]; then
        log_error "Missing dependencies: ${missing_deps[*]}"
        log_info ""
        log_info "Install on Debian/Ubuntu:"
        log_info "  sudo apt-get update"
        log_info "  sudo apt-get install -y build-essential curl wget file libssl-dev \\"
        log_info "    libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev \\"
        log_info "    libwebkit2gtk-4.1-dev libusb-1.0-0-dev pkg-config dpkg rpm"
        log_info ""
        log_info "Install on Fedora/RHEL:"
        log_info "  sudo dnf install -y gcc gcc-c++ curl wget file openssl-devel \\"
        log_info "    gtk3-devel libappindicator-gtk3-devel librsvg2-devel \\"
        log_info "    webkit2gtk4.1-devel libusb-devel pkg-config rpm-build"
        return 2
    fi

    log_success "All system dependencies satisfied"
    return 0
}

check_node_dependencies() {
    log_section "Checking Node.js Dependencies"

    cd "${PROJECT_ROOT}"

    if [ ! -d "node_modules" ]; then
        log_info "node_modules not found, running npm install..."
        npm install || {
            log_error "npm install failed"
            return 1
        }
    fi

    log_info "Node.js version: $(node --version)"
    log_info "npm version: $(npm --version)"

    log_success "Node.js dependencies ready"
    return 0
}

check_rust_dependencies() {
    log_section "Checking Rust Dependencies"

    log_info "Rust version: $(rustc --version)"
    log_info "Cargo version: $(cargo --version)"

    # Check Tauri CLI
    if ! cargo tauri --version &> /dev/null; then
        log_warn "Tauri CLI not found, installing..."
        cargo install tauri-cli --version "^2.0.0" || {
            log_error "Failed to install Tauri CLI"
            return 1
        }
    fi

    log_info "Tauri CLI version: $(cargo tauri --version)"

    log_success "Rust dependencies ready"
    return 0
}

# ============================================================================
# Build Functions
# ============================================================================

build_frontend() {
    log_section "Building Frontend"

    cd "${PROJECT_ROOT}"

    log_info "Running TypeScript compiler..."
    npm run build || {
        log_error "Frontend build failed"
        return 3
    }

    if [ ! -d "dist" ]; then
        log_error "dist directory not created"
        return 3
    fi

    log_success "Frontend built successfully"
    return 0
}

build_tauri() {
    log_section "Building Tauri Application"

    cd "${PROJECT_ROOT}"

    local build_flags=()

    if [ "${BUILD_MODE}" = "release" ]; then
        build_flags+=("--release")
    else
        build_flags+=("--debug")
    fi

    # Determine which bundles to build
    local bundle_targets=()
    [ "${BUILD_DEB}" = true ] && bundle_targets+=("deb")
    [ "${BUILD_APPIMAGE}" = true ] && bundle_targets+=("appimage")
    [ "${BUILD_RPM}" = true ] && bundle_targets+=("rpm")

    if [ ${#bundle_targets[@]} -eq 0 ]; then
        log_error "No bundle targets specified"
        return 3
    fi

    log_info "Building bundles: ${bundle_targets[*]}"
    log_info "Build mode: ${BUILD_MODE}"

    for target in "${bundle_targets[@]}"; do
        log_info "Building ${target} bundle..."

        if [ "${VERBOSE}" = true ]; then
            cargo tauri build "${build_flags[@]}" --bundles "${target}" --target x86_64-unknown-linux-gnu -v || {
                log_error "Failed to build ${target} bundle"
                return 3
            }
        else
            cargo tauri build "${build_flags[@]}" --bundles "${target}" --target x86_64-unknown-linux-gnu 2>&1 | tee -a "${LOG_FILE}" || {
                log_error "Failed to build ${target} bundle"
                return 3
            }
        fi

        log_success "${target} bundle created"
    done

    log_success "Tauri build completed"
    return 0
}

# ============================================================================
# Verification Functions
# ============================================================================

verify_artifacts() {
    log_section "Verifying Build Artifacts"

    local bundle_dir="${PROJECT_ROOT}/src-tauri/target/release/bundle"
    local version
    version=$(get_version)
    local artifacts_found=0

    log_info "Looking for artifacts in: ${bundle_dir}"
    log_info "Expected version: ${version}"

    # Check .deb
    if [ "${BUILD_DEB}" = true ]; then
        local deb_path="${bundle_dir}/deb/hinotes-desktop_${version}_amd64.deb"
        if [ -f "${deb_path}" ]; then
            local deb_size
            deb_size=$(du -h "${deb_path}" | cut -f1)
            log_success ".deb package found: ${deb_path} (${deb_size})"

            # Verify deb package
            if command -v dpkg-deb &> /dev/null; then
                dpkg-deb --info "${deb_path}" >> "${LOG_FILE}" 2>&1
                log_info "Package details written to log file"
            fi

            artifacts_found=$((artifacts_found + 1))
        else
            log_warn ".deb package not found at expected path"
            log_info "Searching for .deb files..."
            find "${bundle_dir}/deb" -name "*.deb" -type f 2>/dev/null | while read -r file; do
                log_info "  Found: ${file}"
            done
        fi
    fi

    # Check .AppImage
    if [ "${BUILD_APPIMAGE}" = true ]; then
        local appimage_path="${bundle_dir}/appimage/hinotes-desktop_${version}_amd64.AppImage"
        if [ -f "${appimage_path}" ]; then
            local appimage_size
            appimage_size=$(du -h "${appimage_path}" | cut -f1)
            log_success ".AppImage found: ${appimage_path} (${appimage_size})"

            # Verify AppImage is executable
            if [ -x "${appimage_path}" ]; then
                log_info "AppImage is executable"
            else
                log_warn "AppImage is not executable, fixing permissions..."
                chmod +x "${appimage_path}"
            fi

            artifacts_found=$((artifacts_found + 1))
        else
            log_warn ".AppImage not found at expected path"
            log_info "Searching for .AppImage files..."
            find "${bundle_dir}/appimage" -name "*.AppImage" -type f 2>/dev/null | while read -r file; do
                log_info "  Found: ${file}"
            done
        fi
    fi

    # Check .rpm
    if [ "${BUILD_RPM}" = true ]; then
        local rpm_path="${bundle_dir}/rpm/hinotes-desktop-${version}-1.x86_64.rpm"
        if [ -f "${rpm_path}" ]; then
            local rpm_size
            rpm_size=$(du -h "${rpm_path}" | cut -f1)
            log_success ".rpm package found: ${rpm_path} (${rpm_size})"

            # Verify rpm package
            if command -v rpm &> /dev/null; then
                rpm -qip "${rpm_path}" >> "${LOG_FILE}" 2>&1
                log_info "Package details written to log file"
            fi

            artifacts_found=$((artifacts_found + 1))
        else
            log_warn ".rpm package not found at expected path"
            log_info "Searching for .rpm files..."
            find "${bundle_dir}/rpm" -name "*.rpm" -type f 2>/dev/null | while read -r file; do
                log_info "  Found: ${file}"
            done
        fi
    fi

    if [ ${artifacts_found} -eq 0 ]; then
        log_error "No build artifacts found"
        return 3
    fi

    log_success "Found ${artifacts_found} build artifact(s)"
    return 0
}

run_basic_tests() {
    log_section "Running Basic Tests"

    cd "${PROJECT_ROOT}"

    # Test that binary exists and is executable
    local binary_path="${PROJECT_ROOT}/src-tauri/target/release/hinotes-desktop"
    if [ -f "${binary_path}" ]; then
        log_info "Binary found: ${binary_path}"

        if [ -x "${binary_path}" ]; then
            log_success "Binary is executable"
        else
            log_error "Binary is not executable"
            return 4
        fi

        # Check dependencies (ldd)
        if command -v ldd &> /dev/null; then
            log_info "Checking binary dependencies..."
            if ldd "${binary_path}" >> "${LOG_FILE}" 2>&1; then
                local missing_libs
                missing_libs=$(ldd "${binary_path}" | grep "not found" || true)
                if [ -n "${missing_libs}" ]; then
                    log_error "Binary has missing dependencies:"
                    echo "${missing_libs}"
                    return 4
                else
                    log_success "All binary dependencies satisfied"
                fi
            fi
        fi
    else
        log_warn "Binary not found at ${binary_path}"
    fi

    log_success "Basic tests passed"
    return 0
}

# ============================================================================
# Report Functions
# ============================================================================

generate_build_report() {
    log_section "Build Report"

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - START_TIME))
    local minutes=$((duration / 60))
    local seconds=$((duration % 60))

    local version
    version=$(get_version)

    echo ""
    log_info "Build Summary"
    log_info "  Version: ${version}"
    log_info "  Build Mode: ${BUILD_MODE}"
    log_info "  Architecture: $(get_architecture)"
    log_info "  Build Time: ${minutes}m ${seconds}s"
    log_info ""

    log_info "Artifacts Location:"
    local bundle_dir="${PROJECT_ROOT}/src-tauri/target/release/bundle"

    if [ "${BUILD_DEB}" = true ]; then
        echo "  .deb: ${bundle_dir}/deb/"
    fi
    if [ "${BUILD_APPIMAGE}" = true ]; then
        echo "  .AppImage: ${bundle_dir}/appimage/"
    fi
    if [ "${BUILD_RPM}" = true ]; then
        echo "  .rpm: ${bundle_dir}/rpm/"
    fi

    echo ""
    log_info "Log File: ${LOG_FILE}"
    echo ""

    log_info "Testing Instructions:"
    log_info "  Debian/Ubuntu: sudo dpkg -i ${bundle_dir}/deb/*.deb"
    log_info "  AppImage: ${bundle_dir}/appimage/*.AppImage"
    log_info "  Fedora/RHEL: sudo rpm -i ${bundle_dir}/rpm/*.rpm"
    echo ""

    log_info "USB Device Permissions:"
    log_info "  Ensure your user is in the 'plugdev' group:"
    log_info "    sudo usermod -aG plugdev \$USER"
    log_info "  Install udev rules for HiDoc P1:"
    log_info "    echo 'SUBSYSTEM==\"usb\", ATTR{idVendor}==\"413c\", ATTR{idProduct}==\"81d7\", MODE=\"0666\"' | sudo tee /etc/udev/rules.d/99-hidoc-p1.rules"
    log_info "    sudo udevadm control --reload-rules && sudo udevadm trigger"
    echo ""
}

# ============================================================================
# Main Execution
# ============================================================================

parse_arguments() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --release)
                BUILD_MODE="release"
                shift
                ;;
            --debug)
                BUILD_MODE="debug"
                shift
                ;;
            --deb-only)
                BUILD_DEB=true
                BUILD_APPIMAGE=false
                BUILD_RPM=false
                shift
                ;;
            --appimage-only)
                BUILD_DEB=false
                BUILD_APPIMAGE=true
                BUILD_RPM=false
                shift
                ;;
            --rpm-only)
                BUILD_DEB=false
                BUILD_APPIMAGE=false
                BUILD_RPM=true
                shift
                ;;
            --skip-deps)
                SKIP_DEPS=true
                shift
                ;;
            --verbose|-v)
                VERBOSE=true
                shift
                ;;
            --clean)
                CLEAN_BUILD=true
                shift
                ;;
            --help|-h)
                show_help
                ;;
            *)
                log_error "Unknown option: $1"
                echo "Use --help for usage information"
                exit 1
                ;;
        esac
    done
}

main() {
    # Initialize log file
    echo "HiNotes Desktop - Linux Build Log" > "${LOG_FILE}"
    echo "Started: $(date)" >> "${LOG_FILE}"
    echo "======================================" >> "${LOG_FILE}"

    log_section "HiNotes Desktop - Linux Build"
    log_info "Build started at $(date)"
    log_info "Project: ${PROJECT_ROOT}"

    # Parse command line arguments
    parse_arguments "$@"

    # Clean if requested
    if [ "${CLEAN_BUILD}" = true ]; then
        cleanup_artifacts
    fi

    # Check dependencies
    if [ "${SKIP_DEPS}" = false ]; then
        check_system_dependencies || exit $?
        check_node_dependencies || exit $?
        check_rust_dependencies || exit $?
    else
        log_warn "Skipping dependency checks (--skip-deps)"
    fi

    # Build
    build_frontend || exit $?
    build_tauri || exit $?

    # Verify
    verify_artifacts || exit $?
    run_basic_tests || exit $?

    # Report
    generate_build_report

    log_success "Build completed successfully!"
    exit 0
}

# Run main function with all arguments
main "$@"
