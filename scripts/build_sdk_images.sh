#!/bin/bash

# build_sdk_images.sh
# ------------------
# This script builds Docker images for microsandbox SDK environments.
#
# Usage:
#   ./scripts/build_sdk_images.sh [options]
#
# Options:
#   -h, --help              Show help message
#   -s, --sdk SDK_NAME      Build specific SDK image (rust, python, nodejs)
#   -a, --all               Build all SDK images (default if no options provided)
#
# The script performs the following tasks:
#   1. Detects which SDK images to build based on arguments
#   2. Builds each selected image using Docker
#   3. Tags images with a consistent naming scheme (msb-[sdk])
#
# Examples:
#   ./scripts/build_sdk_images.sh                # Build all SDK images
#   ./scripts/build_sdk_images.sh -s python      # Build only the Python SDK image
#   ./scripts/build_sdk_images.sh -s rust -s nodejs  # Build Rust and Node.js SDK images
#
# All images are built from the project root, using multi-stage builds that
# compile the portal binary with appropriate language features enabled.

# Exit on any error
set -e

# Color variables
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the project root directory
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# List of available SDKs
AVAILABLE_SDKS=("rust" "python" "nodejs")

# Display usage information
function show_usage {
    echo -e "${BLUE}Usage:${NC} $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -h, --help              Show this help message"
    printf "  -s, --sdk SDK_NAME      Build specific SDK image (${YELLOW}rust${NC}, ${YELLOW}python${NC}, ${YELLOW}nodejs${NC})\n"
    echo "  -a, --all               Build all SDK images (default)"
    echo
    echo "Examples:"
    echo "  $0                      # Build all SDK images"
    echo "  $0 -s python            # Build only the Python SDK image"
    echo "  $0 -s rust -s nodejs    # Build Rust and Node.js SDK images"
    echo
}

# Logging functions
info() {
    printf "${GREEN}:: %s${NC}\n" "$1"
}

warn() {
    printf "${YELLOW}:: %s${NC}\n" "$1"
}

error() {
    printf "${RED}:: %s${NC}\n" "$1"
}

# Function to build a specific SDK image
function build_sdk_image {
    local sdk=$1
    local image_name="msb-${sdk}"
    local dockerfile="$PROJECT_ROOT/sdk-images/${sdk}/Dockerfile"

    # Check if Dockerfile exists
    if [ ! -f "$dockerfile" ]; then
        error "Dockerfile not found for ${sdk} SDK at ${dockerfile}"
        return 1
    fi

    info "Building ${sdk} SDK image..."
    info "Using Dockerfile: ${dockerfile}"
    info "Image name: ${image_name}"

    # Build the image
    docker build -t "$image_name" -f "$dockerfile" "$PROJECT_ROOT"

    # Check if build was successful
    if [ $? -eq 0 ]; then
        info "Successfully built ${image_name} image!"
        info "You can run it with: docker run -it -p 4444:4444 -e RUST_LOG=info --name ${image_name} ${image_name}"
    else
        error "Failed to build ${image_name} image!"
        return 1
    fi
}

# Parse command line arguments
SDKS_TO_BUILD=()

# If no arguments are provided, default to build all
if [ $# -eq 0 ]; then
    info "No arguments provided, building all available SDK images"
    SDKS_TO_BUILD=("${AVAILABLE_SDKS[@]}")
else
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        key="$1"
        case $key in
            -h|--help)
                show_usage
                exit 0
                ;;
            -s|--sdk)
                if [ -z "$2" ] || [[ "$2" == -* ]]; then
                    error "No SDK name provided after -s/--sdk option"
                    show_usage
                    exit 1
                fi
                # Check if the provided SDK is in the list of available SDKs
                if [[ " ${AVAILABLE_SDKS[*]} " =~ " $2 " ]]; then
                    SDKS_TO_BUILD+=("$2")
                    info "Added ${2} to build queue"
                else
                    error "Invalid SDK name: ${2}"
                    printf "Available SDKs: ${YELLOW}%s${NC}\n" "${AVAILABLE_SDKS[*]}"
                    exit 1
                fi
                shift
                ;;
            -a|--all)
                info "Building all available SDK images"
                SDKS_TO_BUILD=("${AVAILABLE_SDKS[@]}")
                ;;
            *)
                error "Unknown option: ${1}"
                show_usage
                exit 1
                ;;
        esac
        shift
    done
fi

# If still no SDKs to build after parsing arguments, show usage
if [ ${#SDKS_TO_BUILD[@]} -eq 0 ]; then
    error "No SDK specified to build"
    show_usage
    exit 1
fi

# Display what will be built
info "Building the following SDK images: ${SDKS_TO_BUILD[*]}"

# Build each SDK image
printf "\n${BLUE}==================== BUILD PROCESS STARTED ====================${NC}\n\n"
for sdk in "${SDKS_TO_BUILD[@]}"; do
    printf "\n${BLUE}---------- Building ${YELLOW}%s${BLUE} SDK ----------${NC}\n\n" "${sdk}"
    build_sdk_image "$sdk"
    if [ $? -ne 0 ]; then
        warn "Failed to build ${sdk} SDK, continuing with next..."
    else
        printf "\n${GREEN}Successfully built ${YELLOW}%s${GREEN} SDK!${NC}\n\n" "${sdk}"
    fi
done

info "All specified SDK images have been processed"
printf "\n${BLUE}======================= BUILD SUMMARY ========================${NC}\n"
echo "Images built:"
for sdk in "${SDKS_TO_BUILD[@]}"; do
    printf "  - ${YELLOW}msb-%s${NC}\n" "${sdk}"
done
printf "\n${GREEN}You can run these images with:${NC}\n"
echo "  docker run -it -p 4444:4444 -e RUST_LOG=info --name IMAGE_NAME IMAGE_NAME"
printf "${BLUE}================================================================${NC}\n\n"
