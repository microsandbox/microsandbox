# First stage: Build the portal binary with Rust feature enabled
FROM rust:slim AS builder

# Install dependencies needed for building
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/microsandbox

# Copy the source code
# Note: This assumes the Docker build command is run from the root of the repository
COPY . .

# Build the portal binary with Rust feature enabled
RUN cargo build --release --bin portal --features rust

# Second stage: Create the Rust image
FROM rust:1.86-slim

# Set environment variables
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.86.0

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run code
ARG USER_NAME="rust-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Create non-root user and necessary directories
RUN useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME

# Install common Rust tools
RUN rustup component add rustfmt clippy rls rust-analysis rust-src \
    && cargo install cargo-edit cargo-watch cargo-expand

# Copy the portal binary from the builder stage
COPY --from=builder /usr/src/microsandbox/target/release/portal /usr/local/bin/
RUN chmod +x /usr/local/bin/portal

# Create a directory for portal configuration
RUN mkdir -p /etc/microsandbox/portal \
    && chown -R $USER_NAME:$USER_GID /etc/microsandbox

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that starts portal and keeps the container running
CMD ["bash", "-c", "echo 'Rust environment with microsandbox-portal ready' && portal & tail -f /dev/null"]
