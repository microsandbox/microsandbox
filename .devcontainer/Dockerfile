FROM mcr.microsoft.com/devcontainers/rust:1-1-bookworm

# Install additional dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    patchelf bc libelf-dev flex bison python3-pip

USER vscode

RUN pip3 install pyelftools --break-system-packages
