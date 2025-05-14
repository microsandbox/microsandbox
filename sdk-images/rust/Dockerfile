# First stage: Build the portal binary with Rust feature enabled
FROM rust:slim AS builder

# Install dependencies and build the portal binary
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /usr/src/microsandbox

WORKDIR /usr/src/microsandbox
COPY . .
RUN cargo build --release --bin portal --features rust

# Second stage: Create the Rust image
FROM rust:1.86-slim

# Set environment variables
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.86.0

ARG USER_NAME="rust-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Install system dependencies, create user, set up directories, and install Rust tools in one layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    pkg-config \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    # Create non-root user and necessary directories
    && useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME \
    && mkdir -p /etc/microsandbox/portal \
    && chown -R $USER_NAME:$USER_GID /etc/microsandbox \
    # Install common Rust tools
    && rustup component add rustfmt clippy rls rust-analysis rust-src \
    && cargo install cargo-edit cargo-watch cargo-expand

# Copy the portal binary from the builder stage and set permissions
COPY --from=builder /usr/src/microsandbox/target/release/portal /usr/local/bin/
RUN chmod +x /usr/local/bin/portal

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that starts portal and keeps the container running
CMD ["bash", "-c", "echo 'Rust environment with microsandbox-portal ready' && portal & tail -f /dev/null"]
