FROM python:3.11-slim

# Set environment variables
ENV PYTHONUNBUFFERED=1 \
    PYTHONDONTWRITEBYTECODE=1 \
    PIP_NO_CACHE_DIR=1 \
    DEBIAN_FRONTEND=noninteractive

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    curl \
    git \
    wget \
    libssl-dev \
    ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Create a non-root user to run code
ARG USER_NAME="python-user"
ARG USER_UID="1000"
ARG USER_GID="100"

# Create non-root user and necessary directories
RUN useradd -m -s /bin/bash -N -u $USER_UID $USER_NAME \
    && mkdir -p /home/$USER_NAME/work \
    && chown -R $USER_NAME:$USER_GID /home/$USER_NAME

# Install core Python packages
RUN pip install --no-cache-dir --upgrade pip setuptools wheel \
    && pip install --no-cache-dir \
    black \
    flake8 \
    mypy \
    pytest \
    pytest-cov \
    requests \
    ipython

# Switch to the non-root user
USER $USER_NAME
WORKDIR /home/$USER_NAME/work

# Set a command that keeps the container running
CMD ["bash", "-c", "echo Python environment ready && tail -f /dev/null"]
