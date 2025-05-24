---
icon: package
title: msb init
description: Initialize a new Microsandbox project
order: 210
---

# msb init

Initialize a new Microsandbox project in the current directory.

## Usage

```bash
msb init [OPTIONS]
```

## Description

The `msb init` command creates a new Microsandbox project by generating a `Sandboxfile` configuration manifest in the current directory. This file serves as the foundation for managing sandbox environments in your project.

## Options

| Option       | Description                |
| ------------ | -------------------------- |
| `-h, --help` | Show help for this command |

## Examples

### Basic Project Initialization

```bash
# Initialize a new project in the current directory
msb init
```

This creates a `Sandboxfile` with basic configuration:

```yaml
# Microsandbox project configuration
name: my-project
version: "1.0.0"

sandboxes: {}
```

### After Initialization

Once initialized, you can start adding sandboxes to your project:

```bash
# Add a Python application sandbox
msb add app --image python --start "python app.py"

# Add a database sandbox
msb add db --image postgres --env "POSTGRES_DB=myapp"
```

## Files Created

- **`Sandboxfile`** - Main configuration file for the project
- **`.gitignore`** (if not exists) - Adds `menv/` to ignore list

## Related Commands

- [`msb add`](add.md) - Add sandboxes to the project
- [`msb run`](run.md) - Run project sandboxes
- [`msb list`](list.md) - List project sandboxes

## What's Next?

After initializing your project:

1. Add sandboxes with `msb add`
2. Configure your environments
3. Start running with `msb run`

---

:bulb: **Tip**: The `Sandboxfile` uses YAML format and supports environment variable substitution with `${VAR_NAME}` syntax.
