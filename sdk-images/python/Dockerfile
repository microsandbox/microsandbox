# First stage: Build the portal binary with Python feature enabled
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
RUN cargo build --release --bin portal --features python

# Second stage: Create the Python image
FROM python:latest

# Set environment variables and create user
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    DEBIAN_FRONTEND=noninteractive

ARG USER_NAME="python-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Install system dependencies, create user, set up directories, and install Python packages in one layer
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    wget \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    # Create non-root user and necessary directories
    && useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME \
    && mkdir -p /etc/microsandbox/portal \
    && chown -R $USER_NAME:$USER_GID /etc/microsandbox \
    # Install core Python packages
    && pip install --no-cache-dir --upgrade pip setuptools wheel \
    && pip install --no-cache-dir \
        black \
        flake8 \
        mypy \
        pytest \
        pytest-cov \
        requests \
        ipython

# Copy the portal binary from the builder stage and set permissions
COPY --from=builder /usr/src/microsandbox/target/release/portal /usr/local/bin/
RUN chmod +x /usr/local/bin/portal

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that starts portal and keeps the container running
CMD ["bash", "-c", "echo 'Python environment with microsandbox-portal ready' && portal & tail -f /dev/null"]
