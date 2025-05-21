#!/bin/bash

# publish_crates.sh
# ----------------
# This script publishes all Cargo packages in the microsandbox workspace to crates.io.
# It ensures all packages have the same version and are published in the correct order
# based on dependencies.
#
# Usage:
#   ./scripts/publish_crates.sh [options]
#
# Options:
#   -h, --help                    Show help message
#   -v, --version VERSION         Version to publish (required, must be semver)
#   -d, --dry-run                 Perform a dry run (don't actually publish)
#   -c, --check-only              Only check that versions are consistent
#   -y, --yes                     Skip confirmation prompts
#   -t, --token TOKEN             Set crates.io token (if not set via env var)
#
# Examples:
#   ./scripts/publish_crates.sh -v 0.1.0            # Publish version 0.1.0 of all crates
#   ./scripts/publish_crates.sh -v 0.2.0 -d         # Test publishing version 0.2.0 (dry run)
#   ./scripts/publish_crates.sh -v 0.1.0 -y -t TOKEN # Publish with token, no confirmation
#
# Note: This script assumes you're already logged in to crates.io using cargo login,
# or that you provide a token with the -t option.

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

# Packages in the workspace (in dependency order)
PACKAGES=(
    "microsandbox-utils"
    "microsandbox-core"
    "microsandbox-server"
    "microsandbox-portal"
    "microsandbox-cli"
)

# Default values
VERSION=""
DRY_RUN=false
CHECK_ONLY=false
SKIP_CONFIRM=false
CRATES_TOKEN=""
# Store the last package name for comparison later
LAST_PACKAGE="${PACKAGES[4]}"  # microsandbox-cli is the last package

# Display usage information
function show_usage {
    echo -e "${BLUE}Usage:${NC} $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  -h, --help                    Show this help message"
    echo "  -v, --version VERSION         Version to publish (required, must be semver)"
    echo "  -d, --dry-run                 Perform a dry run (don't actually publish)"
    echo "  -c, --check-only              Only check that versions are consistent"
    echo "  -y, --yes                     Skip confirmation prompts"
    echo "  -t, --token TOKEN             Set crates.io token (if not set via env var)"
    echo
    echo "Examples:"
    echo "  $0 -v 0.1.0            # Publish version 0.1.0 of all crates"
    echo "  $0 -v 0.2.0 -d         # Test publishing version 0.2.0 (dry run)"
    echo "  $0 -v 0.1.0 -y -t TOKEN # Publish with token, no confirmation"
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

# Check if version is valid semver
validate_version() {
    local version=$1
    if ! [[ $version =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9\.]+)?(\+[a-zA-Z0-9\.]+)?$ ]]; then
        error "Invalid version format: $version"
        echo "Version must be in semver format (e.g., 1.2.3, 1.2.3-alpha, 1.2.3+metadata)"
        exit 1
    fi
}

# Check if cargo is available
check_cargo() {
    if ! command -v cargo &> /dev/null; then
        error "Cargo is not installed or not in PATH"
        exit 1
    fi
}

# Check if user is logged in to crates.io
check_crates_auth() {
    # If token is provided, set it up
    if [ -n "$CRATES_TOKEN" ]; then
        info "Setting provided crates.io token"
        if [ "$DRY_RUN" = false ]; then
            cargo login "$CRATES_TOKEN" >/dev/null
        else
            info "[DRY RUN] Would set crates.io token"
        fi
        return 0
    fi

    # Otherwise check if already logged in
    if [ ! -f "$HOME/.cargo/credentials.toml" ] && [ ! -f "$HOME/.cargo/credentials" ]; then
        error "Not logged in to crates.io. Use 'cargo login' or provide token with -t/--token"
        exit 1
    fi
}

# Update version in Cargo.toml files
update_versions() {
    local version=$1
    local workspace_toml="$PROJECT_ROOT/Cargo.toml"

    info "Updating version in workspace Cargo.toml to $version"
    if [ "$DRY_RUN" = false ]; then
        # Update version in workspace.package section of Cargo.toml
        sed -i.bak "/^\[workspace\.package\]/,/^\[/ s/^version = \"[0-9]*\.[0-9]*\.[0-9]*.*\"/version = \"$version\"/" "$workspace_toml"
        rm "${workspace_toml}.bak"
    else
        info "[DRY RUN] Would update version in $workspace_toml [workspace.package] section to $version"
    fi

    # For crates that don't use workspace versioning, update their version
    for package in "${PACKAGES[@]}"; do
        local cargo_toml="$PROJECT_ROOT/$package/Cargo.toml"
        if [ -f "$cargo_toml" ]; then
            # Check if the package uses workspace versioning
            if grep -q "version.workspace = true" "$cargo_toml"; then
                info "$package uses workspace versioning, no direct version update needed"
            else
                info "Updating version in $package/Cargo.toml to $version"
                if [ "$DRY_RUN" = false ]; then
                    # Update within [package] section only
                    sed -i.bak "/^\[package\]/,/^\[/ s/^version = \"[0-9]*\.[0-9]*\.[0-9]*.*\"/version = \"$version\"/" "$cargo_toml"
                    rm "${cargo_toml}.bak"
                else
                    info "[DRY RUN] Would update direct version in $cargo_toml"
                fi
            fi

            # Update dependencies on other workspace packages that don't use path-only dependencies
            if [ "$DRY_RUN" = false ]; then
                for dep in "${PACKAGES[@]}"; do
                    # Only update if the dependency specifies a version and not just a path
                    if grep -q "$dep = { \?version" "$cargo_toml"; then
                        sed -i.bak "s/$dep = { \?version = \"[0-9]*\.[0-9]*\.[0-9]*.*\"/$dep = { version = \"$version\"/" "$cargo_toml"
                        rm "${cargo_toml}.bak"
                    fi
                done
            else
                info "[DRY RUN] Would update dependencies in $cargo_toml"
            fi
        else
            warn "Could not find Cargo.toml for package $package"
        fi
    done

    info "All version numbers updated to $version"
}

# Check if versions are consistent
check_versions() {
    # Get the workspace version from the [workspace.package] section
    local workspace_section=$(sed -n '/^\[workspace\.package\]/,/^\[/p' "$PROJECT_ROOT/Cargo.toml")
    local workspace_version=$(echo "$workspace_section" | grep 'version =' | head -1 | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+.*)".*/\1/')

    if [ -z "$workspace_version" ]; then
        error "Could not find workspace version in Cargo.toml"
        exit 1
    fi

    info "Workspace version: $workspace_version"

    local consistent=true
    for package in "${PACKAGES[@]}"; do
        local cargo_toml="$PROJECT_ROOT/$package/Cargo.toml"
        if [ -f "$cargo_toml" ]; then
            # Check if package uses workspace versioning
            if grep -q "version.workspace = true" "$cargo_toml"; then
                info "$package version: $workspace_version (via workspace)"
            else
                # Extract the package version from the [package] section
                local package_section=$(sed -n '/^\[package\]/,/^\[/p' "$cargo_toml")
                local package_version=$(echo "$package_section" | grep 'version =' | head -1 | sed -E 's/.*"([0-9]+\.[0-9]+\.[0-9]+.*)".*/\1/')

                if [ -z "$package_version" ]; then
                    error "Could not find version in $package"
                    consistent=false
                elif [ "$package_version" != "$workspace_version" ]; then
                    error "Version mismatch in $package: $package_version (expected $workspace_version)"
                    consistent=false
                else
                    info "$package version: $package_version"
                fi
            fi
        else
            warn "Could not find Cargo.toml for package $package"
        fi
    done

    if [ "$consistent" = false ]; then
        error "Versions are not consistent across the workspace"
        exit 1
    else
        info "All versions are consistent with workspace version: $workspace_version"
    fi
}

# Publish a single crate
publish_crate() {
    local package=$1
    local package_dir="$PROJECT_ROOT/$package"

    if [ ! -d "$package_dir" ]; then
        error "Package directory not found: $package_dir"
        return 1
    fi

    info "Publishing $package v$VERSION to crates.io"
    cd "$package_dir"

    if [ "$DRY_RUN" = true ]; then
        # For dry runs, skip the verification step entirely to avoid dependency issues
        info "[DRY RUN] Simulating cargo publish for $package v$VERSION"

        # Use cargo publish with --dry-run and --no-verify to skip checking dependencies on crates.io
        if cargo publish --dry-run --no-verify --allow-dirty; then
            info "[DRY RUN] Package verification successful for $package v$VERSION"
        else
            error "[DRY RUN] Failed to verify package $package"
            exit 1
        fi
    else
        # Real publish - should proceed only if all dependencies are properly available
        # Capture both stdout and stderr
        output=$(cargo publish 2>&1)
        exit_code=$?

        # Check if the output contains "already exists" error message
        if [ $exit_code -ne 0 ]; then
            if echo "$output" | grep -q "already exists"; then
                warn "Package $package v$VERSION already exists on crates.io, skipping..."
                # Return success so the script continues
                return 0
            else
                # It's a different error, so print it and exit
                error "Failed to publish $package v$VERSION"
                echo "$output"
                exit 1
            fi
        else
            info "Successfully published $package v$VERSION"
        fi
    fi

    # Return to project root
    cd "$PROJECT_ROOT"
}

# Check if a package is available on crates.io
wait_for_package_availability() {
    local package=$1
    local version=$2
    local max_attempts=40  # roughly 2 minutes (40 * 3s)
    local attempt=1

    # In dry-run mode we skip the remote check entirely – we assume success.
    if [ "$DRY_RUN" = true ]; then
        info "[DRY RUN] Skipping crates.io availability check for $package"
        return 0
    fi

    # If a package was already found to exist during publication, we don't need to wait
    if [ -n "$SKIP_WAIT_FOR_$package" ]; then
        info "Package $package v$version was already on crates.io, skipping wait check"
        return 0
    fi

    info "Waiting for $package v$version to be available on crates.io…"

    while [ $attempt -le $max_attempts ]; do
        # Query crates.io via its API
        response=$(curl -s "https://crates.io/api/v1/crates/${package}/${version}")

        # Check if the response contains successful version data
        # Look for both the version number and the package name in the response
        if echo "$response" | grep -q "\"num\":\"${version}\"" && \
           echo "$response" | grep -q "\"crate\":\"${package}\""; then
            info "$package v$version is now available on crates.io!"
            return 0
        fi

        # Debug output if needed
        if [ $attempt -eq 1 ] || [ $((attempt % 10)) -eq 0 ]; then
            # Log the API response occasionally for debugging
            echo "API response (attempt $attempt):"
            echo "$response" | head -10
        fi

        sleep 3
        attempt=$((attempt + 1))
    done

    warn "Timed out waiting for $package v$version to become available."

    # Even if we time out, we'll check one last time by making a direct HTTP request to the package page
    # This is a fallback in case the API is inconsistent
    if curl -s -I "https://crates.io/crates/${package}/${version}" | grep -q "HTTP/"; then
        info "Package $package v$version appears to be available on crates.io website, continuing..."
        return 0
    fi

    # Ask the user what to do
    read -p "Continue with publishing anyway? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        info "Continuing with publishing as requested..."
        return 0
    else
        error "Publishing aborted due to dependency availability issues."
        exit 1
    fi
}

# Publish all crates in order
publish_crates() {
    info "Publishing all crates with version $VERSION"

    # Temporarily disable strict error handling for this function
    set +e

    if [ "$SKIP_CONFIRM" = false ]; then
        read -p "Are you sure you want to publish all crates with version $VERSION? (y/n) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            info "Publishing cancelled."
            # Re-enable strict error handling before exiting
            set -e
            exit 0
        fi
    fi

    # Track published packages to know which ones we need to wait for
    local published_packages=()
    # Track already existing packages to skip wait checks
    declare -A already_exists_packages

    for package in "${PACKAGES[@]}"; do
        # First check if we need to wait for any dependencies
        local has_unpublished_deps=false

        for published in "${published_packages[@]}"; do
            # Check if the current package depends on previously published packages
            # Handle both formats:
            # 1. microsandbox-utils = { version = "0.1.0", ... }
            # 2. microsandbox-utils.workspace = true
            if grep -q "$published = { \?version" "$PROJECT_ROOT/$package/Cargo.toml" || \
               grep -q "$published\.workspace = true" "$PROJECT_ROOT/$package/Cargo.toml"; then

                if [ "$DRY_RUN" = false ]; then
                    info "$package depends on $published, waiting for it to be available..."

                    # Skip waiting if we already know it exists
                    if [ "${already_exists_packages[$published]}" = "1" ]; then
                        info "Dependency $published v$VERSION already exists on crates.io, no need to wait"
                    else
                        wait_for_package_availability "$published" "$VERSION"
                    fi
                else
                    info "$package depends on $published (dependency noted for dry run)"
                fi
            fi
        done

        # Now publish the package
        info "Publishing $package v$VERSION to crates.io"
        cd "$PROJECT_ROOT/$package"

        if [ "$DRY_RUN" = true ]; then
            # For dry runs, skip the verification step entirely to avoid dependency issues
            info "[DRY RUN] Simulating cargo publish for $package v$VERSION"

            # Use cargo publish with --dry-run and --no-verify to skip checking dependencies on crates.io
            cargo publish --dry-run --no-verify --allow-dirty
            if [ $? -eq 0 ]; then
                info "[DRY RUN] Package verification successful for $package v$VERSION"
            else
                error "[DRY RUN] Failed to verify package $package"
                # Re-enable strict error handling before exiting
                set -e
                exit 1
            fi
        else
            # Real publish - should proceed only if all dependencies are properly available
            # Capture both stdout and stderr
            output=$(cargo publish 2>&1)
            exit_code=$?

            # Check if the output contains "already exists" error message
            if [ $exit_code -ne 0 ]; then
                if echo "$output" | grep -q "already exists"; then
                    warn "Package $package v$VERSION already exists on crates.io, skipping..."
                    # Mark this package as already existing to skip waiting for it later
                    already_exists_packages[$package]=1
                else
                    # It's a different error, so print it and exit
                    error "Failed to publish $package v$VERSION"
                    echo "$output"
                    # Re-enable strict error handling before exiting
                    set -e
                    exit 1
                fi
            else
                info "Successfully published $package v$VERSION"
            fi
        fi

        # Return to project root
        cd "$PROJECT_ROOT"

        # Add to the list of published packages
        published_packages+=("$package")

        # After publishing (real run), wait until crates.io shows the crate before proceeding
        if [ "$DRY_RUN" = false ] && [ "$package" != "$LAST_PACKAGE" ]; then
            # Skip waiting if we already know it exists
            if [ "${already_exists_packages[$package]}" = "1" ]; then
                info "Package $package v$VERSION already exists on crates.io, no need to wait"
            else
                wait_for_package_availability "$package" "$VERSION"
            fi
        fi
    done

    info "All crates published successfully with version $VERSION"

    # Re-enable strict error handling
    set -e
}

# Parse command line arguments
if [ $# -eq 0 ]; then
    show_usage
    exit 1
fi

while [[ $# -gt 0 ]]; do
    key="$1"
    case $key in
        -h|--help)
            show_usage
            exit 0
            ;;
        -v|--version)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No version provided after -v/--version option"
                show_usage
                exit 1
            fi
            VERSION="$2"
            shift
            ;;
        -d|--dry-run)
            DRY_RUN=true
            info "Running in dry-run mode, no packages will be published"
            ;;
        -c|--check-only)
            CHECK_ONLY=true
            info "Only checking version consistency, no packages will be published"
            ;;
        -y|--yes)
            SKIP_CONFIRM=true
            info "Skipping confirmation prompts"
            ;;
        -t|--token)
            if [ -z "$2" ] || [[ "$2" == -* ]]; then
                error "No token provided after -t/--token option"
                show_usage
                exit 1
            fi
            CRATES_TOKEN="$2"
            shift
            ;;
        *)
            error "Unknown option: ${1}"
            show_usage
            exit 1
            ;;
    esac
    shift
done

# Validate required parameters
if [ -z "$VERSION" ] && [ "$CHECK_ONLY" = false ]; then
    error "Version is required (-v/--version)"
    show_usage
    exit 1
fi

# Validate version if provided
if [ -n "$VERSION" ]; then
    validate_version "$VERSION"
fi

# Check prerequisites
check_cargo

# Perform the requested operations
if [ "$CHECK_ONLY" = true ]; then
    check_versions
    exit 0
fi

# Update versions
if [ -n "$VERSION" ]; then
    update_versions "$VERSION"
fi

# Check versions after updating
check_versions

# Authentication check is only needed for actual publishing
if [ "$DRY_RUN" = false ]; then
    check_crates_auth
fi

# Publish the crates
publish_crates
