#!/bin/sh

# install_microsandbox.sh
# ------------------
# This script downloads and installs microsandbox binaries and libraries for the user's platform.
#
# Usage:
#   ./install_microsandbox.sh [options]
#
# Options:
#   --version       Specify version to install (default: 0.2.0)
#   --no-cleanup   Skip cleanup of temporary files after installation
#
# The script performs the following tasks:
#   1. Detects OS and architecture
#   2. Downloads appropriate release archive from GitHub
#   3. Verifies checksum
#   4. Installs binaries to ~/.local/bin
#   5. Installs libraries to ~/.local/lib
#   6. Creates unversioned library symlinks
#
# Installation Paths:
#   Executables: ~/.local/bin/
#   Libraries: ~/.local/lib/

# Color variables
RED="\033[1;31m"
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
RESET="\033[0m"

# Logging functions
info() {
    printf "${GREEN}:: %s${RESET}\n" "$1"
}

warn() {
    printf "${YELLOW}:: %s${RESET}\n" "$1"
}

error() {
    printf "${RED}:: %s${RESET}\n" "$1"
}

# Default values
VERSION="0.2.0"
NO_CLEANUP=false
TEMP_DIR="/tmp/microsandbox-install"
GITHUB_REPO="microsandbox/microsandbox"

# Installation directories
BIN_DIR="$HOME/.local/bin"
LIB_DIR="$HOME/.local/lib"

# Parse command line arguments
for arg in "$@"; do
    case $arg in
        --version=*)
            VERSION="${arg#*=}"
            shift
            ;;
        --no-cleanup)
            NO_CLEANUP=true
            shift
            ;;
    esac
done

# Function to check command existence
check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        error "Required command '$1' not found. Please install it first."
        exit 1
    fi
}

# Check required commands
check_command curl
check_command tar
check_command shasum

# Detect OS and architecture
detect_platform() {
    OS="unknown"
    ARCH="unknown"

    case "$(uname -s)" in
        Linux*)     OS="linux";;
        Darwin*)    OS="darwin";;
        *)          error "Unsupported operating system: $(uname -s)"; exit 1;;
    esac

    case "$(uname -m)" in
        x86_64)     ARCH="x86_64";;
        arm64)      ARCH="arm64";;
        aarch64)    ARCH="aarch64";;
        *)          error "Unsupported architecture: $(uname -m)"; exit 1;;
    esac

    # Normalize architecture for Darwin
    if [ "$OS" = "darwin" ] && [ "$ARCH" = "aarch64" ]; then
        ARCH="arm64"
    fi

    PLATFORM="${OS}-${ARCH}"
    ARCHIVE_NAME="microsandbox-${VERSION}-${PLATFORM}.tar.gz"
    CHECKSUM_FILE="${ARCHIVE_NAME}.sha256"
}

# Cleanup function
cleanup() {
    if [ "$NO_CLEANUP" = true ]; then
        info "Skipping cleanup as requested"
        return
    fi

    info "Cleaning up temporary files..."
    rm -rf "$TEMP_DIR"
}

# Set up trap for cleanup
trap cleanup EXIT

# Create necessary directories
create_directories() {
    info "Creating installation directories..."
    mkdir -p "$BIN_DIR" "$LIB_DIR" "$TEMP_DIR"
    if [ $? -ne 0 ]; then
        error "Failed to create directories"
        exit 1
    fi
}

# Download files from GitHub
download_files() {
    info "Downloading microsandbox ${VERSION} for ${PLATFORM}..."

    BASE_URL="https://github.com/${GITHUB_REPO}/releases/download/microsandbox-v${VERSION}"

    cd "$TEMP_DIR" || exit 1

    # Download archive with progress bar
    curl -L -# -o "${ARCHIVE_NAME}" "${BASE_URL}/${ARCHIVE_NAME}" || { error "Failed to download archive"; exit 1; }

    # Download checksum silently
    curl -L -s -o "${CHECKSUM_FILE}" "${BASE_URL}/${CHECKSUM_FILE}" || { error "Failed to download checksum"; exit 1; }
}

# Verify checksum
verify_checksum() {
    info "Verifying checksum..."
    cd "$TEMP_DIR" || exit 1

    # Redirect detailed output to /dev/null but keep the exit status
    if ! (shasum -a 256 -c "$CHECKSUM_FILE" >/dev/null 2>&1); then
        error "Checksum verification failed"
        exit 1
    fi
}

# Extract and install files
install_files() {
    info "Extracting files..."
    cd "$TEMP_DIR" || exit 1

    tar xzf "$ARCHIVE_NAME" || { error "Failed to extract archive"; exit 1; }

    EXTRACT_DIR="microsandbox-${VERSION}-${PLATFORM}"
    cd "$EXTRACT_DIR" || { error "Failed to enter extract directory"; exit 1; }

    # Install main executables
    info "Installing executables..."
    install -m 755 msb "$BIN_DIR/" || { error "Failed to install msb"; exit 1; }
    install -m 755 msbrun "$BIN_DIR/" || { error "Failed to install msbrun"; exit 1; }
    install -m 755 msbserver "$BIN_DIR/" || { error "Failed to install msbserver"; exit 1; }

    # Install alias executables
    install -m 755 msr "$BIN_DIR/" || { error "Failed to install msr"; exit 1; }
    install -m 755 msx "$BIN_DIR/" || { error "Failed to install msx"; exit 1; }
    install -m 755 msi "$BIN_DIR/" || { error "Failed to install msi"; exit 1; }

    # Self codesign on macOS
    if [ "$OS" = "darwin" ]; then
        info "Attempting to codesign executables on macOS..."
        codesign --force -s - "$BIN_DIR/msbrun" 2>/dev/null || true
        info "Codesigning done"
    fi

    # Install libraries
    info "Installing libraries..."
    if [ "$OS" = "darwin" ]; then
        # Install versioned dylibs
        for lib in *.dylib; do
            install -m 755 "$lib" "$LIB_DIR/" || { error "Failed to install $lib"; exit 1; }
        done

        # Create unversioned symlinks
        cd "$LIB_DIR" || exit 1
        ln -sf libkrun.*.dylib libkrun.dylib
        ln -sf libkrunfw.*.dylib libkrunfw.dylib
    else
        # Install versioned shared objects
        for lib in *.so.*; do
            install -m 755 "$lib" "$LIB_DIR/" || { error "Failed to install $lib"; exit 1; }
        done

        # Create unversioned symlinks
        cd "$LIB_DIR" || exit 1
        ln -sf libkrun.so.* libkrun.so
        ln -sf libkrunfw.so.* libkrunfw.so
    fi
}

# Function to check if a line exists in a file
line_exists() {
    grep -Fxq "$1" "$2" 2>/dev/null
}

# Function to add environment config for sh/bash/zsh
setup_posix_shell() {
    local shell_rc="$1"
    local shell_name="$2"
    local lib_path_var="$3"

    info "Setting up $shell_name configuration..."

    # Create the file if it doesn't exist
    touch "$shell_rc"

    # PATH configuration
    if ! line_exists 'export PATH="$HOME/.local/bin:$PATH"' "$shell_rc"; then
        echo >> "$shell_rc"
        echo '# Added by microsandbox installer' >> "$shell_rc"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$shell_rc"
    fi

    # Library path configuration
    if ! line_exists "export $lib_path_var=\"\$HOME/.local/lib:\$$lib_path_var\"" "$shell_rc"; then
        echo "export $lib_path_var=\"\$HOME/.local/lib:\$$lib_path_var\"" >> "$shell_rc"
    fi
}

# Function to set up fish shell
setup_fish() {
    local fish_config="$HOME/.config/fish/config.fish"
    local lib_path_var="$1"

    info "Setting up fish configuration..."

    # Create config directory if it doesn't exist
    mkdir -p "$(dirname "$fish_config")"
    touch "$fish_config"

    # PATH configuration
    if ! line_exists "set -gx PATH $HOME/.local/bin \$PATH" "$fish_config"; then
        echo >> "$fish_config"
        echo '# Added by microsandbox installer' >> "$fish_config"
        echo "set -gx PATH $HOME/.local/bin \$PATH" >> "$fish_config"
    fi

    # Library path configuration
    if ! line_exists "set -gx $lib_path_var $HOME/.local/lib \$$lib_path_var" "$fish_config"; then
        echo "set -gx $lib_path_var $HOME/.local/lib \$$lib_path_var" >> "$fish_config"
    fi
}

# Add this function near the other utility functions
check_shell() {
    command -v "$1" >/dev/null 2>&1
}

# Function to configure shell environment
configure_shell_env() {
    info "Configuring detected shells..."

    # Determine library path variable based on OS
    local lib_path_var
    case "$(uname -s)" in
        Linux*)     lib_path_var="LD_LIBRARY_PATH";;
        Darwin*)    lib_path_var="DYLD_LIBRARY_PATH";;
        *)          warn "Unsupported OS for environment configuration"; return 1;;
    esac

    # Configure bash if installed
    if check_shell bash; then
        setup_posix_shell "$HOME/.bashrc" "bash" "$lib_path_var"
    fi

    # Configure zsh if installed
    if check_shell zsh; then
        setup_posix_shell "$HOME/.zshrc" "zsh" "$lib_path_var"
    fi

    # Configure fish if installed
    if check_shell fish; then
        setup_fish "$lib_path_var"
    fi

    # Always configure .profile for POSIX shell compatibility
    setup_posix_shell "$HOME/.profile" "sh" "$lib_path_var"

    info "All detected shell environments configured. Please restart your shell or source your shell's config file"
}

# Main installation process
main() {
    info "Starting microsandbox installation..."

    detect_platform
    create_directories
    download_files
    verify_checksum
    install_files

    # Configure shell environment
    configure_shell_env
    if [ $? -ne 0 ]; then
        warn "Shell environment configuration failed, but installation completed"
    fi

    info "Installation completed successfully!"
    info "Executables installed to: $BIN_DIR"
    info "  - msb: main microsandbox command"
    info "  - msbrun: microsandbox runtime executable"
    info "  - msbserver: microsandbox server executable"
    info "  - msr: alias for 'msb run'"
    info "  - msx: alias for 'msb exe'"
    info "  - msi: alias for 'msb install'"
    info "Libraries installed to: $LIB_DIR"
    info "Please restart your shell or source your shell's config file to use microsandbox"
}

# Run main installation
main
