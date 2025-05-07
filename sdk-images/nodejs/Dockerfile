# First stage: Build the portal binary with Node.js feature enabled
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

# Build the portal binary with Node.js feature enabled
RUN cargo build --release --bin portal --features nodejs

# Second stage: Create the Node.js image
# Using latest LTS version (currently 20) from Chainguard's minimalist images
FROM node:20-slim

# Set environment variables
ENV NODE_ENV=development \
    NPM_CONFIG_LOGLEVEL=info

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    ca-certificates \
    build-essential \
    python3 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# The node image already includes a non-root 'node' user with UID 1000
# Set up working directory for the node user
RUN mkdir -p /home/node/work \
    && chown -R node:node /home/node

# Install global npm packages
RUN npm install -g npm@latest \
    typescript \
    ts-node \
    nodemon \
    eslint \
    prettier

# Copy the portal binary from the builder stage
COPY --from=builder /usr/src/microsandbox/target/release/portal /usr/local/bin/
RUN chmod +x /usr/local/bin/portal

# Create a directory for portal configuration
RUN mkdir -p /etc/microsandbox/portal \
    && chown -R node:node /etc/microsandbox

# Switch to the non-root node user
USER node
WORKDIR /home/node/work

# Set a command that starts portal and keeps the container running
CMD ["bash", "-c", "echo 'Node.js environment with microsandbox-portal ready' && portal & tail -f /dev/null"]
