---
icon: download
title: msb install
description: Install a sandbox as a system-wide executable
order: 214
---

# msb install

Install a sandbox as a system-wide executable that can be launched from anywhere.

## Usage

```bash
msb install <IMAGE> [ALIAS] [OPTIONS]
```

## Description

The `msb install` command creates a system-wide launcher for a sandbox environment. Once installed, you can start the sandbox by simply typing its name in any terminal, making frequently used environments incredibly convenient to access.

## Arguments

| Argument  | Description                                    |
| --------- | ---------------------------------------------- |
| `<IMAGE>` | Container image to install                     |
| `[ALIAS]` | Optional custom name for the installed sandbox |

## Options

| Option              | Description                |
| ------------------- | -------------------------- |
| `--memory <MEMORY>` | Memory limit in MB         |
| `--cpus <CPUS>`     | CPU limit (e.g., 1.5)      |
| `-h, --help`        | Show help for this command |

## Examples

### Basic Installation

```bash
# Install Python environment
msb install python

# Install with custom name
msb install python py

# Install specific version
msb install python:3.11 py311
```

### With Resource Limits

```bash
# Install with memory limit
msb install python data-python --memory 2048

# Install with CPU limit
msb install python compute-python --cpus 4

# Install with both limits
msb install python heavy-python --memory 4096 --cpus 2
```

### Multiple Environments

```bash
# Install different Python versions
msb install python:3.9 py39
msb install python:3.10 py310
msb install python:3.11 py311

# Install different Node.js versions
msb install node:16 node16
msb install node:18 node18
msb install node:20 node20
```

### Specialized Environments

```bash
# Data science environment
msb install jupyter/datascience-notebook datascience --memory 4096

# Development environment
msb install ubuntu:22.04 dev-env --memory 1024

# Lightweight shell
msb install alpine:latest shell
```

## How It Works

When you install a sandbox:

1. **Launcher Creation** - A small executable is created in your system PATH
2. **Configuration Storage** - Sandbox settings are saved for reuse
3. **State Persistence** - Each installed sandbox maintains its own persistent state
4. **Instant Access** - Launch from any directory with a simple command

## Installation Location

Installed sandboxes are placed in:

- **macOS/Linux**: `~/.local/bin/` (added to PATH automatically)
- **Configuration**: `~/.config/microsandbox/installed/`
- **State**: `~/.local/share/microsandbox/environments/`

## Using Installed Sandboxes

After installation, simply type the sandbox name:

```bash
# Launch installed Python environment
python

# Launch with custom name
py311

# Launch data science environment
datascience

# Launch development environment
dev-env
```

## State Persistence

Installed sandboxes maintain state between sessions:

```bash
# Install packages in one session
python
>>> pip install requests numpy pandas
>>> exit()

# Packages are still available in the next session
python
>>> import requests, numpy, pandas  # All available!
```

## Managing Installed Sandboxes

### List Installed Sandboxes

```bash
# List all installed sandboxes
msb list --installed

# Show detailed information
msb list --installed --verbose
```

### Update Installed Sandboxes

```bash
# Update to latest image
msb install python --update

# Reinstall with new configuration
msb install python data-python --memory 4096 --force
```

### Remove Installed Sandboxes

```bash
# Remove an installed sandbox
msb uninstall python

# Remove with custom name
msb uninstall data-python
```

## Use Cases

### Development Environments

```bash
# Python development
msb install python:3.11 py --memory 2048

# Node.js development
msb install node:18 node --memory 1024

# Go development
msb install golang:1.21 go --memory 1024
```

### Data Science

```bash
# Jupyter environment
msb install jupyter/datascience-notebook jupyter --memory 4096

# R environment
msb install r-base:4.3 r --memory 2048

# Python with ML libraries
msb install tensorflow/tensorflow:latest ml --memory 8192
```

### System Administration

```bash
# Ubuntu environment
msb install ubuntu:22.04 ubuntu

# Alpine Linux
msb install alpine:latest alpine

# Network tools
msb install nicolaka/netshoot nettools
```

### Language-Specific Environments

```bash
# Rust development
msb install rust:1.70 rust

# Java development
msb install openjdk:17 java

# PHP development
msb install php:8.2 php
```

## Configuration Examples

### Resource-Optimized Installations

```bash
# Lightweight environments
msb install alpine:latest shell --memory 128 --cpus 0.5

# Heavy computation
msb install python:3.11 compute --memory 8192 --cpus 8

# Balanced development
msb install node:18 dev --memory 1024 --cpus 2
```

### Specialized Workflows

```bash
# Code formatting
msb install python:3.11 formatter
formatter
>>> pip install black flake8 isort
>>> # Use for code formatting across projects

# Testing environment
msb install python:3.11 test-env
test-env
>>> pip install pytest pytest-cov tox
>>> # Use for running tests
```

## Security Considerations

- **Isolation** - Each installed sandbox runs in complete isolation
- **Resource Limits** - Configured limits prevent resource exhaustion
- **State Separation** - Different sandboxes don't share state
- **Clean Removal** - Uninstalling removes all traces

## Related Commands

- [`msb exe`](exe.md) - Run temporary sandboxes
- [`msb run`](run.md) - Run project sandboxes
- [`msb uninstall`](uninstall.md) - Remove installed sandboxes
- [`msb list`](list.md) - List installed sandboxes

## Aliases

For convenience, you can use the shorter alias:

```bash
# These are equivalent
msb install python
msi python
```

## Tips

- Use descriptive names for specialized environments
- Set appropriate resource limits based on intended use
- Install multiple versions of the same tool with different names
- Leverage state persistence for package installations
- Use installed sandboxes for consistent development environments

---

:bulb: **Tip**: Installed sandboxes are perfect for creating consistent, reproducible development environments that you can access from anywhere on your system.
