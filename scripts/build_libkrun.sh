#!/bin/sh

# build_libkrun.sh
# ---------------
# This script automates the building of libkrun and libkrunfw libraries,
# which are essential components for running micro virtual machines.
#
# Usage:
#   ./build_libkrun.sh [options]
#
# Options:
#   --no-cleanup, --no-clean    Skip cleanup of build directories and VMs after completion
#   --force-build, --force      Force rebuild even if libraries are already built
#
# Requirements:
#   - git
#   - make
#   - Rust/Cargo (for libkrun)
#   - Python with packages in ~/.local/lib/python3.*/site-packages (for libkrunfw)
#   - On macOS: krunvm must be installed (brew tap slp/krun && brew install krunvm)
#   - On Linux: patchelf must be installed
#
# The script performs the following tasks:
#   1. Creates build directory if needed
#   2. Clones libkrunfw from Github
#   3. Clones libkrun from GitHub
#   4. Builds both libraries in the build directory
#   5. Creates non-versioned variants of libraries
#   6. Handles cleanup (including VM deletion on macOS) unless --no-cleanup is specified
#
# Library Build Paths:
#   Libraries are built and placed in the ./build directory:
#   Linux:
#     - ./build/libkrun.so.$ABI_VERSION (versioned)
#     - ./build/libkrun.so (symlink to versioned)
#     - ./build/libkrunfw.so.$ABI_VERSION (versioned)
#     - ./build/libkrunfw.so (symlink to versioned)
#   macOS:
#     - ./build/libkrun.$ABI_VERSION.dylib (versioned)
#     - ./build/libkrun.dylib (symlink to versioned)
#     - ./build/libkrunfw.$ABI_VERSION.dylib (versioned)
#     - ./build/libkrunfw.dylib (symlink to versioned)
#   Note: $ABI_VERSION is determined from each library's Makefile
#
# Error Handling:
#   - The script checks for errors after each critical operation
#   - Exits with status code 1 on any failure
#   - Performs cleanup on exit unless --no-cleanup is specified
#   - On macOS, cleanup includes deleting libkrunfw-builder and libkrun-builder VMs
#
# Platform Support:
#   - Linux: Full support
#   - macOS: Requires krunvm, handles platform-specific paths and library extensions
#   - Other platforms are not supported
#
# Examples:
#   # Standard build
#   ./build_libkrun.sh
#
#   # Build without cleaning up build directory and VMs
#   ./build_libkrun.sh --no-cleanup
#
#   # Force rebuild even if libraries exist
#   ./build_libkrun.sh --force-build
#
#   # Combine options
#   ./build_libkrun.sh --no-cleanup --force-build

# Color variables
RED="\033[1;31m"
GREEN="\033[1;32m"
YELLOW="\033[1;33m"
RESET="\033[0m"

# Package requirements per distribution
DEBIAN_PACKAGES="patchelf bc libelf-dev gcc flex bison git python3 curl"
FEDORA_PACKAGES="patchelf bc elfutils-libelf-devel gcc flex bison git python3 curl"
ARCH_PACKAGES="patchelf bc libelf gcc flex bison git python3 curl"
SUSE_PACKAGES="patchelf bc libelf-devel gcc flex bison git python3 curl"

# Function to detect Linux distribution
detect_linux_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        echo "$ID"
    elif [ -f /etc/debian_version ]; then
        echo "debian"
    elif [ -f /etc/fedora-release ]; then
        echo "fedora"
    elif [ -f /etc/arch-release ]; then
        echo "arch"
    else
        echo "unknown"
    fi
}

# Function to check for required packages
check_linux_packages() {
    local distro=$(detect_linux_distro)
    local missing_packages=""
    local install_command=""
    local packages=""

    case $distro in
        debian|ubuntu)
            install_command="apt-get install -y"
            packages=$DEBIAN_PACKAGES
            # Use dpkg-query and an exact status check to verify that the package is fully installed
            for pkg in $packages; do
                status=$(dpkg-query -W -f='${Status}' "$pkg" 2>/dev/null || echo "not installed")
                if [ "$status" != "install ok installed" ]; then
                    missing_packages="$missing_packages $pkg"
                fi
            done
            ;;
        fedora|rhel|centos)
            install_command="dnf install -y"
            packages=$FEDORA_PACKAGES
            # Check if rpm command exists and packages are installed
            for pkg in $packages; do
                if ! rpm -q "$pkg" >/dev/null 2>&1; then
                    missing_packages="$missing_packages $pkg"
                fi
            done
            ;;
        arch)
            install_command="pacman -S --noconfirm"
            packages=$ARCH_PACKAGES
            # Check if pacman command exists and packages are installed
            for pkg in $packages; do
                if ! pacman -Qi "$pkg" >/dev/null 2>&1; then
                    missing_packages="$missing_packages $pkg"
                fi
            done
            ;;
        suse|opensuse|sles)
            install_command="zypper install -y"
            packages=$SUSE_PACKAGES
            # Check if rpm command exists and packages are installed
            for pkg in $packages; do
                if ! rpm -q "$pkg" >/dev/null 2>&1; then
                    missing_packages="$missing_packages $pkg"
                fi
            done
            ;;
        *)
            warn "Unable to detect Linux distribution. Please ensure required packages are installed manually:"
            info "Required packages: patchelf, bc, libelf-dev/elfutils-libelf-devel, gcc, flex, bison"
            return
            ;;
    esac

    if [ -n "$missing_packages" ]; then
        error "Missing required packages:$missing_packages"
        info "You can install them using:"
        info "sudo $install_command$missing_packages"
        exit 1
    fi
}

# Function to check for required packages
check_python_dependencies() {
    # Check if python3 is installed
    if ! command -v python3 >/dev/null 2>&1; then
        error "python3 is not installed. Please install python3."
        exit 1
    fi

    # Check if pip is available using python3 -m pip
    if ! python3 -m pip --version >/dev/null 2>&1; then
        error "pip is not installed. Please install pip for python3 (e.g., apt-get install python3-pip)."
        exit 1
    fi

    # Check if pyelftools is installed (you import it as 'elftools')
    if ! python3 -c "import elftools" >/dev/null 2>&1; then
        error "pyelftools module is not installed. Please run: pip3 install pyelftools"
        exit 1
    fi
}

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

# Store the original working directory
ORIGINAL_DIR="$(pwd)"

# Ensure PATH includes common binary locations
export PATH="/usr/local/bin:/usr/bin:/bin:$PATH"

# Set up variables
BUILD_DIR="$ORIGINAL_DIR/build"
LIBKRUNFW_REPO="https://github.com/appcypher/libkrunfw.git"
LIBKRUN_REPO="https://github.com/appcypher/libkrun.git"
NO_CLEANUP=false
FORCE_BUILD=false

# Parse command line arguments
for arg in "$@"
do
    case $arg in
      --no-clean|--no-cleanup)
        NO_CLEANUP=true
        shift
        ;;
      --force|--force-build)
        FORCE_BUILD=true
        shift
        ;;
    esac
done

# Determine the OS type
OS_TYPE="$(uname -s)"

# Check if krunvm is installed on macOS, if applicable
if [ "$OS_TYPE" = "Darwin" ]; then
  if ! which krunvm >/dev/null 2>&1; then
    printf "${RED}krunvm command not found. Please install it using: brew tap slp/krun && brew install krunvm${RESET}\n"
    exit 1
  fi
fi

# Check for required packages and Python dependencies on Linux
if [ "$OS_TYPE" = "Linux" ]; then
    check_linux_packages
    check_python_dependencies
fi

# Function to handle cleanup
cleanup() {
  if [ "$NO_CLEANUP" = true ]; then
    info "Skipping cleanup as requested."
    return
  fi

  warn "Cleaning up..."

  cd "$ORIGINAL_DIR" || { error "Failed to change back to original directory"; exit 1; }

  rm -rf "$BUILD_DIR"
  if [ "$OS_TYPE" = "Darwin" ]; then
    warn "Deleting libkrunfw-builder VM..."
    krunvm delete libkrunfw-builder

    warn "Deleting libkrun-builder VM..."
    krunvm delete libkrun-builder
  fi
  info "Cleanup complete."
}

# Trap EXIT signal to run cleanup
trap cleanup EXIT

# Function to check command success
check_success() {
  if [ $? -ne 0 ]; then
    error "Error occurred: $1"
    exit 1
  fi
}

# Common function to check for existing installations
check_existing_lib() {
    if [ "$FORCE_BUILD" = true ]; then
        info "Force build enabled. Skipping check for existing $1."
        return 0
    fi

    local lib_name="$1"

    # Get ABI version from the appropriate Makefile
    local abi_version=$(get_abi_version "$BUILD_DIR/$lib_name/Makefile")

    case "$OS_TYPE" in
        Linux)
            lib_path="$BUILD_DIR/$lib_name.so.$abi_version"
            ;;
        Darwin)
            lib_path="$BUILD_DIR/$lib_name.$abi_version.dylib"
            ;;
        *)
            error "Unsupported OS: $OS_TYPE"
            exit 1
            ;;
    esac

    if [ -f "$lib_path" ]; then
        info "$lib_name already exists in $lib_path. Skipping build."
        return 1
    fi
    return 0
}

# Function to create build directory
create_build_directory() {
  cd "$ORIGINAL_DIR" || { error "Failed to change to original directory"; exit 1; }

  if [ -d "$BUILD_DIR" ]; then
    info "Build directory already exists. Skipping creation..."
  else
    info "Creating build directory..."
    mkdir -p "$BUILD_DIR"
    check_success "Failed to create build directory"
  fi
}

# Common function to clone repositories
clone_repo() {
  cd "$BUILD_DIR" || { error "Failed to change to build directory"; exit 1; }

  local repo_url="$1"
  local repo_name="$2"
  shift 2  # Remove the first two arguments, leaving any additional args

  if [ -d "$repo_name" ]; then
    info "$repo_name directory already exists. Skipping cloning..."
  else
    info "Cloning $repo_name repository..."
    git clone "$repo_url" "$@"  # Pass any remaining arguments to git clone
    check_success "Failed to clone $repo_name repository"
  fi
}

# Function to extract ABI version from Makefile
get_abi_version() {
    local makefile="$1"
    local abi_version=$(grep "^ABI_VERSION.*=" "$makefile" | cut -d'=' -f2 | tr -d ' ')
    if [ -z "$abi_version" ]; then
        error "Could not determine ABI version from $makefile"
        exit 1
    fi
    echo "$abi_version"
}

# Function to extract FULL_VERSION from Makefile
get_full_version() {
    local makefile="$1"
    local full_version=$(grep "^FULL_VERSION.*=" "$makefile" | cut -d'=' -f2 | tr -d ' ')
    if [ -z "$full_version" ]; then
        error "Could not determine FULL_VERSION from $makefile"
        exit 1
    fi
    echo "$full_version"
}

# Function to build and copy libkrunfw
build_libkrunfw() {
    cd "$BUILD_DIR/libkrunfw" || { error "Failed to change to libkrunfw directory"; exit 1; }

    local abi_version=$(get_abi_version "Makefile")
    info "Detected libkrunfw ABI version: $abi_version"

    info "Building libkrunfw..."
    export PYTHONPATH="$HOME/.local/lib/python3.*/site-packages:$PYTHONPATH"

    case "$OS_TYPE" in
        Darwin)
            # On macOS, we need sudo to allow krunvm set xattr on the volume
            sudo make PYTHONPATH="$PYTHONPATH"
            ;;
        *)
            make PYTHONPATH="$PYTHONPATH"
            ;;
    esac
    check_success "Failed to build libkrunfw"

    # Copy the library to build directory and create symlink
    info "Copying libkrunfw to build directory..."
    cd "$BUILD_DIR" || { error "Failed to change to build directory"; exit 1; }
    case "$OS_TYPE" in
        Linux)
            cp libkrunfw/libkrunfw.so.$abi_version.* "libkrunfw.so.$abi_version"
            patchelf --set-rpath '$ORIGIN' "libkrunfw.so.$abi_version"
            ln -sf "libkrunfw.so.$abi_version" "libkrunfw.so"
            ;;
        Darwin)
            cp libkrunfw/libkrunfw.$abi_version.dylib "libkrunfw.$abi_version.dylib"
            install_name_tool -id "@rpath/libkrunfw.$abi_version.dylib" "libkrunfw.$abi_version.dylib"
            ln -sf "libkrunfw.$abi_version.dylib" "libkrunfw.dylib"
            ;;
        *)
            error "Unsupported OS: $OS_TYPE"
            exit 1
            ;;
    esac
    check_success "Failed to copy libkrunfw"
}

# Function to build and copy libkrun
build_libkrun() {
    cd "$BUILD_DIR/libkrun" || { error "Failed to change to libkrun directory"; exit 1; }

    local abi_version=$(get_abi_version "Makefile")
    local full_version=$(get_full_version "Makefile")
    info "Detected libkrun ABI version: $abi_version"
    info "Detected libkrun FULL version: $full_version"

    info "Building libkrun..."
    # Update library path to use our build directory
    export LIBRARY_PATH="$BUILD_DIR:$LIBRARY_PATH"
    export PATH="$HOME/.cargo/bin:$PATH"

    case "$OS_TYPE" in
        Darwin)
            sudo make LIBRARY_PATH="$LIBRARY_PATH" PATH="$PATH"
            ;;
        *)
            make LIBRARY_PATH="$LIBRARY_PATH" PATH="$PATH"
            ;;
    esac
    check_success "Failed to build libkrun"

    # Copy and rename the library to build directory and create symlink
    info "Copying libkrun to build directory..."
    cd "$BUILD_DIR" || { error "Failed to change to build directory"; exit 1; }
    case "$OS_TYPE" in
        Linux)
            cp libkrun/target/release/libkrun.so.$full_version "libkrun.so.$abi_version"
            patchelf --set-rpath '$ORIGIN' "libkrun.so.$abi_version"
            patchelf --set-needed "libkrunfw.so.4" "libkrun.so.$abi_version"
            ln -sf "libkrun.so.$abi_version" "libkrun.so"
            ;;
        Darwin)
            cp libkrun/target/release/libkrun.$full_version.dylib "libkrun.$abi_version.dylib"
            install_name_tool -id "@rpath/libkrun.$abi_version.dylib" "libkrun.$abi_version.dylib"
            install_name_tool -change "libkrunfw.4.dylib" "@rpath/libkrunfw.4.dylib" "libkrun.$abi_version.dylib"
            ln -sf "libkrun.$abi_version.dylib" "libkrun.dylib"
            ;;
        *)
            error "Unsupported OS: $OS_TYPE"
            exit 1
            ;;
    esac
    check_success "Failed to copy libkrun"
}

# Main script execution
check_existing_lib "libkrunfw"
if [ $? -eq 0 ]; then
    create_build_directory
    clone_repo "$LIBKRUNFW_REPO" "libkrunfw" --single-branch
    build_libkrunfw
fi

check_existing_lib "libkrun"
if [ $? -eq 0 ]; then
    create_build_directory
    clone_repo "$LIBKRUN_REPO" "libkrun" --single-branch
    build_libkrun
fi

# Finished
info "Setup complete."
