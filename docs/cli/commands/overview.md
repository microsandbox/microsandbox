---
icon: list-unordered
title: Command Overview
description: Complete list of all Microsandbox CLI commands
order: 201
---

# Command Overview

This page provides a comprehensive overview of all available `msb` commands, organized by category. Each command has its own detailed documentation page.

<div align='center'>• • •</div>

## Project Management

Manage Microsandbox projects and their sandbox configurations.

| Command                   | Description                   | Page           |
| ------------------------- | ----------------------------- | -------------- |
| [`msb init`](init.md)     | Initialize a new project      | [→](init.md)   |
| [`msb add`](add.md)       | Add sandboxes to project      | [→](add.md)    |
| [`msb remove`](remove.md) | Remove sandboxes from project | [→](remove.md) |
| [`msb list`](list.md)     | List project sandboxes        | [→](list.md)   |

<div align='center'>• • •</div>

## Sandbox Execution

Run and manage sandbox environments.

| Command                         | Description                       | Page              |
| ------------------------------- | --------------------------------- | ----------------- |
| [`msb run`](run.md)             | Run project sandboxes             | [→](run.md)       |
| [`msb exe`](exe.md)             | Run temporary sandboxes           | [→](exe.md)       |
| [`msb shell`](shell.md)         | Open interactive shells           | [→](shell.md)     |
| [`msb install`](install.md)     | Install sandbox as system command | [→](install.md)   |
| [`msb uninstall`](uninstall.md) | Remove installed sandboxes        | [→](uninstall.md) |

<div align='center'>• • •</div>

## Project Operations

Manage multiple sandboxes as a group.

| Command                   | Description                 | Page           |
| ------------------------- | --------------------------- | -------------- |
| [`msb up`](up.md)         | Start all project sandboxes | [→](up.md)     |
| [`msb down`](down.md)     | Stop all project sandboxes  | [→](down.md)   |
| [`msb status`](status.md) | Show sandbox status         | [→](status.md) |

<div align='center'>• • •</div>

## Server Management

Control the Microsandbox server that orchestrates sandboxes.

| Command                   | Description               | Page           |
| ------------------------- | ------------------------- | -------------- |
| [`msb server`](server.md) | Manage the sandbox server | [→](server.md) |

### Server Subcommands

| Subcommand          | Description            |
| ------------------- | ---------------------- |
| `msb server start`  | Start the server       |
| `msb server stop`   | Stop the server        |
| `msb server status` | Check server status    |
| `msb server keygen` | Generate API keys      |
| `msb server log`    | Show server logs       |
| `msb server list`   | List running sandboxes |

<div align='center'>• • •</div>

## Image Management

Manage container images for sandboxes.

| Command                 | Description               | Page          |
| ----------------------- | ------------------------- | ------------- |
| [`msb pull`](pull.md)   | Pull images from registry | [→](pull.md)  |
| [`msb build`](build.md) | Build custom images       | [→](build.md) |
| [`msb push`](push.md)   | Push images to registry   | [→](push.md)  |

<div align='center'>• • •</div>

## Utility Commands

Additional tools and utilities.

| Command                     | Description              | Page            |
| --------------------------- | ------------------------ | --------------- |
| [`msb log`](log.md)         | Show logs for sandboxes  | [→](log.md)     |
| [`msb clean`](clean.md)     | Clean cached data        | [→](clean.md)   |
| [`msb version`](version.md) | Show version information | [→](version.md) |

<div align='center'>• • •</div>

## Command Aliases

For convenience, several commands have shorter aliases:

| Full Command  | Alias | Description               |
| ------------- | ----- | ------------------------- |
| `msb run`     | `msr` | Run project sandbox       |
| `msb exe`     | `msx` | Execute temporary sandbox |
| `msb install` | `msi` | Install sandbox           |

<div align='center'>• • •</div>

## Quick Reference

### Common Workflows

#### Project Development

```bash
msb init                           # Initialize project
msb add app --image python        # Add sandbox
msb run app                        # Run sandbox
```

#### Quick Experimentation

```bash
msb exe python                     # Temporary environment
msx python --exec "pip install requests"  # One-off command
```

#### Server Management

```bash
msb server start --dev             # Start development server
msb server status                  # Check server health
msb server stop                    # Stop server
```

#### System Installation

```bash
msb install python py              # Install as system command
py                                 # Use installed sandbox
msb uninstall py                   # Remove installation
```

<div align='center'>• • •</div>

## Global Options

All commands support these global options:

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

Configure CLI behavior:

| Variable         | Description        | Default                 |
| ---------------- | ------------------ | ----------------------- |
| `MSB_SERVER_URL` | Sandbox server URL | `http://127.0.0.1:5555` |
| `MSB_API_KEY`    | Authentication key | None                    |
| `MSB_LOG_LEVEL`  | Default log level  | `info`                  |

<div align='center'>• • •</div>

## Getting Help

### Command-Specific Help

Get detailed help for any command:

```bash
msb --help                         # General help
msb <command> --help               # Command-specific help
msb server start --help            # Subcommand help
```

### Documentation

- **Individual Commands** - Click any command link above for detailed documentation
- **Examples** - Each command page includes practical examples
- **Options** - Complete option reference for each command

---

:bulb: **Tip**: Use `msb <command> --help` to get detailed help for any specific command, including all available options and examples.
