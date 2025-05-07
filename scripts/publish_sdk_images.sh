#!/bin/bash

# publish_sdk_images.sh
# --------------------
# This script publishes Docker images for microsandbox SDK environments to a registry.
# It supports publishing for the current architecture or creating multi-arch manifests.
#
# Usage:
#   ./scripts/publish_sdk_images.sh [options]
#
# Options:
#   -h, --help                Show help message
#   -s, --sdk SDK_NAME        Publish specific SDK image (rust, python, nodejs)
#   -a, --all                 Publish all SDK images (default if no SDK specified)
#   -u, --username USERNAME   Docker registry username (required)
#   -o, --org ORGANIZATION    Docker registry organization/account (defaults to username)
#   -t, --tag TAG             Tag to use for the images (default: latest)
#   -m, --multi-arch          Create and push multi-arch manifests (requires prior builds on different architectures)
#   -r, --registry REGISTRY   Docker registry to use (default: docker.io)
#   --dry-run                 Don't actually push images, just show what would be done
#
# Examples:
#   ./scripts/publish_sdk_images.sh -u myuser -s python                # Push python image to docker.io/myuser/msb-python:latest
#   ./scripts/publish_sdk_images.sh -u myuser -o myorg -s nodejs -t v1 # Push nodejs image to docker.io/myorg/msb-nodejs:v1
#   ./scripts/publish_sdk_images.sh -u myuser --multi-arch -a          # Push all multi-arch images
#
# Note: For multi-arch manifests, you need to have built the images on different
# architectures and pushed them with architecture-specific tags first.

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

# Default values
USERNAME=""
ORGANIZATION=""
TAG="latest"
REGISTRY="docker.io"
MULTI_ARCH=false
DRY_RUN=false

# Display usage information
function show_usage {
    echo -e "${BLUE}Usage:${NC} $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -h, --help                Show this help message"
    printf "  -s, --sdk SDK_NAME        Publish specific SDK image (${YELLOW}rust${NC}, ${YELLOW}python${NC}, ${YELLOW}nodejs${NC})\n"
    echo "  -a, --all                 Publish all SDK images (default if no SDK specified)"
    echo "  -u, --username USERNAME   Docker registry username (required)"
    echo "  -o, --org ORGANIZATION    Docker registry organization/account (defaults to username)"
    echo "  -t, --tag TAG             Tag to use for the images (default: latest)"
    echo "  -m, --multi-arch          Create and push multi-arch manifests"
    echo "  -r, --registry REGISTRY   Docker registry to use (default: docker.io)"
    echo "  --dry-run                 Don't actually push images, just show what would be done"
    echo
    echo "Examples:"
    echo "  $0 -u myuser -s python                # Push python image to docker.io/myuser/msb-python:latest"
    echo "  $0 -u myuser -o myorg -s nodejs -t v1 # Push nodejs image to docker.io/myorg/msb-nodejs:v1"
    echo "  $0 -u myuser --multi-arch -a          # Push all multi-arch images"
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

# Detect architecture and normalize to Docker's naming convention
detect_architecture() {
    local arch=$(uname -m)

    case "$arch" in
        x86_64)
            echo "amd64"
            ;;
        amd64)
            echo "amd64"
            ;;
        arm64)
            echo "arm64"
            ;;
        aarch64)
            echo "arm64"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Check if Docker is available
check_docker() {
    if ! command -v docker &> /dev/null; then
        error "Docker is not installed or not in PATH"
        exit 1
    fi
}

# Check if Docker buildx is available (required for multi-arch builds)
check_buildx() {
    if ! docker buildx version &> /dev/null; then
        error "Docker buildx is not available, required for multi-arch builds"
        exit 1
    fi
}

# Function to publish a single SDK image
publish_sdk_image() {
    local sdk=$1
    local arch=$(detect_architecture)
    local local_image="msb-${sdk}"
    local remote_image="${REGISTRY}/${ORGANIZATION}/msb-${sdk}"

    info "Publishing ${sdk} SDK image for architecture ${arch}..."

    # Check if the local image exists
    if ! docker image inspect "${local_image}" &> /dev/null; then
        error "Local image ${local_image} not found. Build it first with ./scripts/build_sdk_images.sh -s ${sdk}"
        return 1
    fi

    # Tag the image with architecture and custom tag
    local arch_tag="${remote_image}:${TAG}-${arch}"

    info "Tagging ${local_image} as ${arch_tag}"
    if [ "$DRY_RUN" = false ]; then
        docker tag "${local_image}" "${arch_tag}"
    else
        info "[DRY RUN] Would tag: docker tag ${local_image} ${arch_tag}"
    fi

    # Push the architecture-specific image
    info "Pushing ${arch_tag}"
    if [ "$DRY_RUN" = false ]; then
        docker push "${arch_tag}"
    else
        info "[DRY RUN] Would push: docker push ${arch_tag}"
    fi

    # If multi-arch is enabled, we'll handle that in a separate function
    if [ "$MULTI_ARCH" = false ]; then
        # Also tag as latest for convenience
        local latest_tag="${remote_image}:${TAG}"
        info "Tagging ${local_image} as ${latest_tag}"
        if [ "$DRY_RUN" = false ]; then
            docker tag "${local_image}" "${latest_tag}"
            docker push "${latest_tag}"
        else
            info "[DRY RUN] Would tag: docker tag ${local_image} ${latest_tag}"
            info "[DRY RUN] Would push: docker push ${latest_tag}"
        fi
    fi

    info "Successfully published ${sdk} SDK image for ${arch}"
}

# Function to create and push multi-architecture manifests
create_multi_arch_manifest() {
    local sdk=$1
    local remote_image="${REGISTRY}/${ORGANIZATION}/msb-${sdk}"
    local manifest_tag="${remote_image}:${TAG}"

    info "Creating multi-architecture manifest for ${sdk}..."

    # Supported architectures
    local archs=("amd64" "arm64")
    local manifest_cmd="docker manifest create ${manifest_tag}"
    local manifest_exists=false

    # Check if all architecture-specific images exist
    for arch in "${archs[@]}"; do
        local arch_tag="${remote_image}:${TAG}-${arch}"

        # Check if this architecture tag exists
        if [ "$DRY_RUN" = false ]; then
            if ! docker manifest inspect "${arch_tag}" &> /dev/null; then
                warn "Architecture-specific image ${arch_tag} not found. Skipping ${arch} for manifest."
                continue
            fi
        fi

        manifest_cmd="${manifest_cmd} ${arch_tag}"
        manifest_exists=true
    done

    if [ "$manifest_exists" = false ]; then
        error "No architecture-specific images found for ${sdk}. Cannot create manifest."
        return 1
    fi

    # Create and push the manifest
    info "Creating manifest: ${manifest_tag}"
    if [ "$DRY_RUN" = false ]; then
        # First remove any existing manifest with the same name
        docker manifest rm "${manifest_tag}" 2>/dev/null || true

        # Create the new manifest
        eval ${manifest_cmd}

        # Push the manifest
        info "Pushing manifest: ${manifest_tag}"
        docker manifest push "${manifest_tag}"
    else
        info "[DRY RUN] Would create manifest: ${manifest_cmd}"
        info "[DRY RUN] Would push manifest: docker manifest push ${manifest_tag}"
    fi

    info "Successfully created and pushed multi-arch manifest for ${sdk}"
}

# Parse command line arguments
SDKS_TO_PUBLISH=()

# If no arguments are provided, show usage
if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

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
                SDKS_TO_PUBLISH+=("$2")
                info "Added ${2} to publish queue"
            else
                error "Invalid SDK name: ${2}"
                printf "Available SDKs: ${YELLOW}%s${NC}\n" "${AVAILABLE_SDKS[*]}"
                exit 1
            fi
            shift
            ;;
        -a|--all)
            info "Publishing all available SDK images"
            SDKS_TO_PUBLISH=("${AVAILABLE_SDKS[@]}")
            ;;
        -u|--username)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No username provided after -u/--username option"
                show_usage
                exit 1
            fi
            USERNAME="$2"
            shift
            ;;
        -o|--org)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No organization provided after -o/--org option"
                show_usage
                exit 1
            fi
            ORGANIZATION="$2"
            shift
            ;;
        -t|--tag)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No tag provided after -t/--tag option"
                show_usage
                exit 1
            fi
            TAG="$2"
            shift
            ;;
        -r|--registry)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No registry provided after -r/--registry option"
                show_usage
                exit 1
            fi
            REGISTRY="$2"
            shift
            ;;
        -m|--multi-arch)
            MULTI_ARCH=true
            ;;
        --dry-run)
            DRY_RUN=true
            info "Running in dry-run mode, no changes will be made"
            ;;
        *)
            error "Unknown option: ${1}"
            show_usage
            exit 1
            ;;
    esac
    shift
done

# If no SDKs specified but other options given, default to all
if [ ${#SDKS_TO_PUBLISH[@]} -eq 0 ]; then
    info "No specific SDK selected, defaulting to all SDKs"
    SDKS_TO_PUBLISH=("${AVAILABLE_SDKS[@]}")
fi

# Validate required parameters
if [ -z "$USERNAME" ]; then
    error "Docker registry username is required (-u/--username)"
    show_usage
    exit 1
fi

# If organization is not specified, use username
if [ -z "$ORGANIZATION" ]; then
    ORGANIZATION="$USERNAME"
    info "Organization not specified, using username: ${ORGANIZATION}"
fi

# Check for Docker and buildx if needed
check_docker
if [ "$MULTI_ARCH" = true ]; then
    check_buildx
fi

# Login to Docker registry (if not already logged in)
info "Logging in to Docker registry ${REGISTRY} as ${USERNAME}"
if [ "$DRY_RUN" = false ]; then
    if ! docker login "${REGISTRY}" -u "${USERNAME}"; then
        error "Failed to log in to Docker registry"
        exit 1
    fi
else
    info "[DRY RUN] Would login: docker login ${REGISTRY} -u ${USERNAME}"
fi

# Display what will be published
info "Publishing the following SDK images to ${REGISTRY}/${ORGANIZATION}: ${SDKS_TO_PUBLISH[*]}"
info "Using tag: ${TAG}"
if [ "$MULTI_ARCH" = true ]; then
    info "Creating multi-architecture manifests"
fi

# Publish each SDK image
printf "\n${BLUE}==================== PUBLISHING STARTED ====================${NC}\n\n"
for sdk in "${SDKS_TO_PUBLISH[@]}"; do
    printf "\n${BLUE}---------- Publishing ${YELLOW}%s${BLUE} SDK ----------${NC}\n\n" "${sdk}"
    publish_sdk_image "$sdk"
    if [ $? -ne 0 ]; then
        warn "Failed to publish ${sdk} SDK, continuing with next..."
    else
        printf "\n${GREEN}Successfully published ${YELLOW}%s${GREEN} SDK!${NC}\n\n" "${sdk}"
    fi

    # Create multi-arch manifest if requested
    if [ "$MULTI_ARCH" = true ]; then
        printf "\n${BLUE}---------- Creating Multi-Arch Manifest for ${YELLOW}%s${BLUE} SDK ----------${NC}\n\n" "${sdk}"
        create_multi_arch_manifest "$sdk"
        if [ $? -ne 0 ]; then
            warn "Failed to create multi-arch manifest for ${sdk} SDK, continuing with next..."
        else
            printf "\n${GREEN}Successfully created multi-arch manifest for ${YELLOW}%s${GREEN} SDK!${NC}\n\n" "${sdk}"
        fi
    fi
done

# Final summary
info "All specified SDK images have been processed"
printf "\n${BLUE}======================= PUBLISH SUMMARY ========================${NC}\n"
echo "Images published to ${REGISTRY}/${ORGANIZATION}:"
for sdk in "${SDKS_TO_PUBLISH[@]}"; do
    printf "  - ${YELLOW}msb-%s${NC}\n" "${sdk}"
done

if [ "$MULTI_ARCH" = true ]; then
    echo -e "\n${GREEN}Multi-architecture manifests created for:${NC}"
    for sdk in "${SDKS_TO_PUBLISH[@]}"; do
        printf "  - ${YELLOW}msb-%s:${TAG}${NC}\n" "${sdk}"
    done
fi

if [ "$DRY_RUN" = true ]; then
    printf "\n${YELLOW}This was a dry run. No actual changes were made.${NC}\n"
fi

printf "${BLUE}================================================================${NC}\n\n"
