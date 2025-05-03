FROM rust:1.77-slim

# Set environment variables
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:$PATH \
    RUST_VERSION=1.77.0

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

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that keeps the container running
CMD ["bash", "-c", "echo Rust environment ready && tail -f /dev/null"]
