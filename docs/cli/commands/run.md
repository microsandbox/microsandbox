---
icon: play
title: msb run
description: Run a sandbox defined in the project
order: 212
---

# msb run

Run a sandbox defined in the project.

## Usage

```bash
msb run <NAME[~SCRIPT]> [-- <ARGS>...]
```

## Description

The `msb run` command executes sandboxes that are defined in your project's `Sandboxfile`. It can run the default start script or execute specific named scripts within the sandbox environment.

## Arguments

| Argument          | Description                                |
| ----------------- | ------------------------------------------ |
| `<NAME[~SCRIPT]>` | Sandbox name, optionally with script name  |
| `[-- <ARGS>...]`  | Additional arguments to pass to the script |

## Options

| Option              | Description                |
| ------------------- | -------------------------- |
| `-s, --sandbox`     | Apply to sandbox           |
| `-b, --build`       | Apply to build sandbox     |
| `-d, --detach`      | Run in background          |
| `-e, --exec <EXEC>` | Execute custom command     |
| `-f, --file <FILE>` | Specify custom Sandboxfile |
| `-h, --help`        | Show help for this command |

## Examples

### Basic Execution

```bash
# Run the default start script
msb run app

# Run a specific script
msb run app~test

# Run with arguments
msb run app -- --verbose --debug
```

### Custom Commands

```bash
# Execute a custom command instead of the configured script
msb run app --exec "python debug.py"

# Run an interactive shell
msb run app --exec "bash"
```

### Background Execution

```bash
# Run in the background (detached mode)
msb run web --detach

# Check status of running sandboxes
msb status
```

### Multiple Scripts

```bash
# Run different scripts for the same sandbox
msb run app~start     # Start the application
msb run app~test      # Run tests
msb run app~migrate   # Run database migrations
msb run app~build     # Build the application
```

## Script Configuration

Scripts are defined in your `Sandboxfile`:

```yaml
sandboxes:
  app:
    image: python
    scripts:
      start: python app.py
      test: pytest tests/
      dev: python app.py --debug --reload
      migrate: alembic upgrade head
      shell: bash
```

## Environment Persistence

When running project sandboxes, all changes are automatically persisted to the `./menv` directory:

```
./menv/
├── app/          # App sandbox state
├── db/           # Database sandbox state
└── web/          # Web sandbox state
```

This means you can:

- Install packages and they persist between runs
- Create files and they remain available
- Stop and restart without losing work
- Share state between different script executions

## Working with Dependencies

If your sandbox has dependencies, they will be started automatically:

```yaml
sandboxes:
  api:
    image: python
    depends_on:
      - db
      - cache
    scripts:
      start: uvicorn main:app

  db:
    image: postgres

  cache:
    image: redis
```

```bash
# This will start db and cache first, then api
msb run api
```

## Passing Arguments

Arguments after `--` are passed directly to the script:

```bash
# Pass arguments to the script
msb run app~test -- --verbose --coverage

# Equivalent to running: pytest tests/ --verbose --coverage
```

## Custom Execution

Use `--exec` to run arbitrary commands:

```bash
# Install a package
msb run app --exec "pip install requests"

# Check installed packages
msb run app --exec "pip list"

# Run a one-off script
msb run app --exec "python -c 'print(\"Hello World\")'"
```

## File Specification

Use a different Sandboxfile:

```bash
# Use a different configuration file
msb run app --file Sandboxfile.dev

# Use configuration from a different directory
msb run app --file ../other-project/Sandboxfile
```

## Exit Codes

The command returns the exit code of the executed script:

```bash
msb run app~test
echo $?  # Shows the exit code of the test script
```

## Related Commands

- [`msb add`](add.md) - Add sandboxes to the project
- [`msb exe`](exe.md) - Run temporary sandboxes
- [`msb status`](status.md) - Check sandbox status
- [`msb list`](list.md) - List project sandboxes

## Aliases

For convenience, you can use the shorter alias:

```bash
# These are equivalent
msb run app
msr app
```

## Tips

- Use descriptive script names in your `Sandboxfile`
- Leverage the persistent environment for development workflows
- Use `--detach` for long-running services
- Pass configuration through environment variables rather than arguments when possible

---

:bulb: **Tip**: Use `msb run <sandbox>~<tab>` to see available scripts for a sandbox (if shell completion is enabled).
