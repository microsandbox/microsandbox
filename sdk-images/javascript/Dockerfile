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

# Create a non-root user to run code
ARG USER_NAME="node-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Create non-root user and necessary directories
RUN useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME

# Install global npm packages
RUN npm install -g npm@latest \
    typescript \
    ts-node \
    nodemon \
    eslint \
    prettier

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that keeps the container running
CMD ["bash", "-c", "echo Node.js environment ready && tail -f /dev/null"]
