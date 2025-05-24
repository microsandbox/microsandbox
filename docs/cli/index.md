---
icon: terminal
title: CLI Reference
description: Complete reference for the Microsandbox CLI
order: 200
---

# CLI Reference

The Microsandbox CLI (`msb`) is a powerful tool for managing lightweight sandboxes and images. This section provides comprehensive documentation for all commands and options.

<div align='center'>• • •</div>

## Overview

The `msb` command-line tool provides functionality for:

- **Project Management** - Initialize and manage sandbox projects
- **Sandbox Operations** - Run, manage, and monitor sandboxes
- **Image Management** - Pull, build, and push sandbox images
- **Server Management** - Start, stop, and configure the sandbox server

<div align='center'>• • •</div>

## Quick Reference

### Project Commands

- `msb init` - Initialize a new project
- `msb add` - Add sandboxes to project
- `msb run` - Run project sandboxes
- `msb list` - List project sandboxes

### Sandbox Commands

- `msb exe` - Run temporary sandboxes
- `msb shell` - Open interactive shells
- `msb install` - Install sandbox scripts
- `msb status` - Check sandbox status

### Server Commands

- `msb server start` - Start the server
- `msb server stop` - Stop the server
- `msb server status` - Check server status
- `msb server keygen` - Generate API keys

### Image Commands

- `msb pull` - Pull images from registry
- `msb build` - Build custom images
- `msb push` - Push images to registry

<div align='center'>• • •</div>

## Global Options

All `msb` commands support these global options:

| Option          | Description                |
| --------------- | -------------------------- |
| `-V, --version` | Show version information   |
| `-h, --help`    | Show help for command      |
| `--error`       | Show error-level logs only |
| `--warn`        | Show warning-level logs    |
| `--info`        | Show info-level logs       |
| `--debug`       | Show debug-level logs      |
| `--trace`       | Show trace-level logs      |

<div align='center'>• • •</div>

## Environment Variables

Configure CLI behavior with environment variables:

| Variable         | Description             | Default                  |
| ---------------- | ----------------------- | ------------------------ |
| `MSB_SERVER_URL` | Sandbox server URL      | `http://127.0.0.1:5555`  |
| `MSB_API_KEY`    | Authentication key      | None                     |
| `MSB_LOG_LEVEL`  | Default log level       | `info`                   |
| `MSB_CONFIG_DIR` | Configuration directory | `~/.config/microsandbox` |

<div align='center'>• • •</div>

## Command Categories

### Project Workflow

```bash
# Initialize a new project
msb init

# Add a Python sandbox
msb add app --image python --start "python app.py"

# Run the sandbox
msb run app

# Check status
msb status
```

### Quick Execution

```bash
# Run temporary Python environment
msb exe python

# Execute specific command
msb exe python --exec "python -c 'print(\"Hello\")'"

# Install as system command
msb install python my-python
```

### Server Management

```bash
# Start server in development mode
msb server start --dev

# Generate API key
msb server keygen --expire 30d

# Check server status
msb server status
```

<div align='center'>• • •</div>

## Shell Completions

Enable tab completion for your shell:

+++ Bash

```bash
echo 'eval "$(msb completion bash)"' >> ~/.bashrc
source ~/.bashrc
```

+++ Zsh

```bash
echo 'eval "$(msb completion zsh)"' >> ~/.zshrc
source ~/.zshrc
```

+++ Fish

```bash
msb completion fish > ~/.config/fish/completions/msb.fish
```

+++

<div align='center'>• • •</div>

## Getting Help

### Command Help

Get help for any command:

```bash
# General help
msb --help

# Command-specific help
msb server --help
msb run --help
```

### Examples

Most commands include usage examples:

```bash
msb add --help
# Shows examples of adding different types of sandboxes
```

<div align='center'>• • •</div>

## Command Aliases

For convenience, several commands have shorter aliases:

| Full Command  | Alias | Description               |
| ------------- | ----- | ------------------------- |
| `msb run`     | `msr` | Run project sandbox       |
| `msb exe`     | `msx` | Execute temporary sandbox |
| `msb install` | `msi` | Install sandbox           |

<div align='center'>• • •</div>

## What's Next?

- [**Command Reference**](commands/overview.md) - Detailed command documentation
- [**Project Guide**](../projects/index.md) - Learn about project workflows
- [**Examples**](../examples/index.md) - See CLI usage examples

---

:bulb: **Tip**: Use `msb <command> --help` to get detailed help for any specific command.
