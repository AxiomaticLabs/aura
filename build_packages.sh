#!/bin/bash
# AuraDB Universal Build Script
# Builds packages for Linux, Windows, and macOS

set -e

echo "🔨 Building AuraDB packages for all platforms..."
echo "================================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Check if we're on the right platform
check_platform() {
    case "$1" in
        linux)
            if [[ "$OSTYPE" != "linux-gnu"* ]]; then
                print_warning "Skipping Linux packaging (not on Linux)"
                return 1
            fi
            ;;
        windows)
            if [[ "$OSTYPE" != "msys"* ]] && [[ "$OSTYPE" != "win32"* ]]; then
                print_warning "Skipping Windows packaging (not on Windows)"
                return 1
            fi
            ;;
        macos)
            if [[ "$OSTYPE" != "darwin"* ]]; then
                print_warning "Skipping macOS packaging (not on macOS)"
                return 1
            fi
            ;;
    esac
    return 0
}

# Build release binaries
build_release() {
    print_info "Building release binaries..."
    cargo build --release --bin aura-server
    cargo build --release --bin aura-cli
    print_status "Release binaries built"
}

# Linux packaging with cargo-deb
build_linux() {
    if ! check_platform linux; then return 0; fi

    print_info "Building Linux packages..."

    # Install cargo-deb if not present
    if ! command -v cargo-deb &> /dev/null; then
        print_info "Installing cargo-deb..."
        cargo install cargo-deb
    fi

    # Build server .deb
    print_info "Building aura-server .deb package..."
    cargo deb -p aura-server
    print_status "Server .deb built: target/debian/aura-server_*.deb"

    # Build CLI .deb
    print_info "Building aura-cli .deb package..."
    cargo deb -p aura-cli
    print_status "CLI .deb built: target/debian/aura-cli_*.deb"

    # Build RPM if cargo-generate-rpm is available
    if command -v cargo-generate-rpm &> /dev/null; then
        print_info "Building RPM packages..."
        cargo generate-rpm -p aura-server
        cargo generate-rpm -p aura-cli
        print_status "RPM packages built"
    else
        print_warning "cargo-generate-rpm not found, skipping RPM build"
        print_info "Install with: cargo install cargo-generate-rpm"
    fi
}

# Windows packaging with cargo-wix
build_windows() {
    if ! check_platform windows; then return 0; fi

    print_info "Building Windows packages..."

    # Install cargo-wix if not present
    if ! command -v cargo-wix &> /dev/null; then
        print_info "Installing cargo-wix..."
        cargo install cargo-wix
    fi

    # Build server MSI
    print_info "Building aura-server .msi package..."
    cargo wix -p aura-server
    print_status "Server MSI built: crates/aura-server/target/wix/aura-server-*.msi"

    # Build CLI MSI
    print_info "Building aura-cli .msi package..."
    cargo wix -p aura-cli
    print_status "CLI MSI built: aura-cli/target/wix/aura-cli-*.msi"
}

# macOS packaging
build_macos() {
    if ! check_platform macos; then return 0; fi

    print_info "Building macOS packages..."

    # Run the macOS build script
    if [[ -f "build_mac_pkg.sh" ]]; then
        print_info "Running macOS PKG build script..."
        chmod +x build_mac_pkg.sh
        ./build_mac_pkg.sh
        print_status "macOS PKG built: target/aura-db-*.pkg"
    else
        print_error "build_mac_pkg.sh not found"
    fi
}

# Create distribution summary
create_summary() {
    echo ""
    print_info "Build Summary:"
    echo "=============="

    echo "Linux (.deb/.rpm):"
    if [[ -d "target/debian" ]]; then
        ls -la target/debian/*.deb 2>/dev/null || echo "  No .deb files found"
    fi
    if [[ -d "target/generate-rpm" ]]; then
        ls -la target/generate-rpm/*.rpm 2>/dev/null || echo "  No .rpm files found"
    fi

    echo ""
    echo "Windows (.msi):"
    if [[ -d "crates/aura-server/target/wix" ]]; then
        ls -la crates/aura-server/target/wix/*.msi 2>/dev/null || echo "  No server MSI found"
    fi
    if [[ -d "aura-cli/target/wix" ]]; then
        ls -la aura-cli/target/wix/*.msi 2>/dev/null || echo "  No CLI MSI found"
    fi

    echo ""
    echo "macOS (.pkg):"
    ls -la target/aura-db-*.pkg 2>/dev/null || echo "  No .pkg files found"

    echo ""
    print_info "Installation Instructions:"
    echo "=========================="
    echo "Linux (Ubuntu/Debian): sudo dpkg -i target/debian/*.deb"
    echo "Linux (Fedora/RHEL):   sudo rpm -i target/generate-rpm/*.rpm"
    echo "Windows:               Double-click the .msi files"
    echo "macOS:                 sudo installer -pkg target/aura-db-*.pkg -target /"
    echo ""
    echo "After installation:"
    echo "- Linux:   sudo systemctl status aura-server"
    echo "- Windows: Check Services.msc for 'AuraDB' service"
    echo "- macOS:   sudo launchctl list com.aura.db"
}

# Main build process
main() {
    local target_platform="$1"

    case "$target_platform" in
        linux)
            build_release
            build_linux
            ;;
        windows)
            build_release
            build_windows
            ;;
        macos)
            build_release
            build_macos
            ;;
        all|"")
            build_release
            build_linux
            build_windows
            build_macos
            ;;
        *)
            print_error "Unknown platform: $target_platform"
            echo "Usage: $0 [linux|windows|macos|all]"
            exit 1
            ;;
    esac

    create_summary
}

# Run main function with provided argument
main "$1"