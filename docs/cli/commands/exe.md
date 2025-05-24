---
icon: zap
title: msb exe
description: Run a temporary sandbox (no project required)
order: 213
---

# msb exe

Run a temporary sandbox without requiring a project configuration.

## Usage

```bash
msb exe <IMAGE> [OPTIONS] [-- <ARGS>...]
```

## Description

The `msb exe` command creates and runs temporary sandboxes that are completely isolated and leave no trace after execution. Unlike project sandboxes, temporary sandboxes don't persist any changes and are perfect for experimentation, testing untrusted code, or one-off tasks.

## Arguments

| Argument         | Description                                 |
| ---------------- | ------------------------------------------- |
| `<IMAGE>`        | Container image to use for the sandbox      |
| `[-- <ARGS>...]` | Additional arguments to pass to the command |

## Options

### Resource Configuration

| Option              | Description           |
| ------------------- | --------------------- |
| `--memory <MEMORY>` | Memory limit in MB    |
| `--cpus <CPUS>`     | CPU limit (e.g., 1.5) |

### Environment Configuration

| Option                  | Description                       |
| ----------------------- | --------------------------------- |
| `-v, --volume <VOLUME>` | Volume mappings (HOST:CONTAINER)  |
| `-p, --port <PORT>`     | Port mappings (HOST:CONTAINER)    |
| `--env <ENV>`           | Environment variables (KEY=VALUE) |
| `--workdir <WORKDIR>`   | Working directory inside sandbox  |

### Execution Options

| Option              | Description                              |
| ------------------- | ---------------------------------------- |
| `-e, --exec <EXEC>` | Execute specific command                 |
| `--scope <SCOPE>`   | Network scope (local, public, any, none) |

### General Options

| Option       | Description                |
| ------------ | -------------------------- |
| `-h, --help` | Show help for this command |

## Examples

### Interactive Environments

```bash
# Start an interactive Python environment
msb exe python

# Start an interactive Node.js environment
msb exe node

# Start a shell in Alpine Linux
msb exe alpine
```

### Execute Specific Commands

```bash
# Run a Python script
msb exe python --exec "python -c 'print(\"Hello World\")'"

# Install and test a package
msb exe python --exec "pip install requests && python -c 'import requests; print(requests.__version__)'"

# Run a Node.js script
msb exe node --exec "node -e 'console.log(process.version)'"
```

### With Resource Limits

```bash
# Run with memory limit
msb exe python --memory 512 --exec "python memory_intensive_script.py"

# Run with CPU limit
msb exe python --cpus 0.5 --exec "python cpu_intensive_task.py"

# Run with both limits
msb exe python --memory 1024 --cpus 2 --exec "python data_processing.py"
```

### With Volume Mounts

```bash
# Mount current directory
msb exe python --volume ".:/workspace" --workdir "/workspace"

# Mount specific files
msb exe python --volume "./script.py:/app/script.py" --exec "python /app/script.py"

# Mount data directory
msb exe python --volume "./data:/data" --exec "python analyze.py /data"
```

### With Port Mappings

```bash
# Run a web server
msb exe python --port "8000:8000" --exec "python -m http.server 8000"

# Run a Node.js app
msb exe node --port "3000:3000" --volume "./app:/workspace" --workdir "/workspace" --exec "npm start"

# Multiple ports
msb exe nginx --port "80:80" --port "443:443"
```

### With Environment Variables

```bash
# Set environment variables
msb exe python --env "DEBUG=true" --env "API_KEY=secret" --exec "python app.py"

# Use shell variables
msb exe python --env "HOME_DIR=${HOME}" --exec "python backup.py"
```

### Network Scopes

```bash
# No network access
msb exe python --scope none --exec "python offline_script.py"

# Local network only
msb exe python --scope local --exec "python local_service.py"

# Full internet access (default)
msb exe python --scope any --exec "python web_scraper.py"
```

## Common Use Cases

### Testing Untrusted Code

```bash
# Safely test code from the internet
msb exe python --scope none --exec "$(curl -s https://example.com/script.py)"

# Test with limited resources
msb exe python --memory 256 --cpus 0.5 --exec "python suspicious_script.py"
```

### Quick Experiments

```bash
# Try a new Python library
msb exe python --exec "pip install numpy && python -c 'import numpy; print(numpy.random.rand(3,3))'"

# Test different Node.js versions
msb exe node:16 --exec "node --version"
msb exe node:18 --exec "node --version"
```

### Data Processing

```bash
# Process data files
msb exe python --volume "./data:/data" --memory 2048 --exec "python process_large_dataset.py"

# Convert file formats
msb exe python --volume "./files:/files" --exec "python convert_csv_to_json.py"
```

### Development Tools

```bash
# Run linting tools
msb exe python --volume ".:/code" --workdir "/code" --exec "pip install flake8 && flake8 ."

# Format code
msb exe python --volume ".:/code" --workdir "/code" --exec "pip install black && black ."
```

## Temporary Nature

Temporary sandboxes are completely ephemeral:

- **No persistence** - All changes are lost when the sandbox stops
- **Clean environment** - Each run starts with a fresh container
- **No state sharing** - Multiple runs don't share any data
- **Automatic cleanup** - Resources are automatically freed

## Security Benefits

- **Complete isolation** - Code runs in a separate VM
- **Resource limits** - Prevent resource exhaustion
- **Network control** - Restrict network access as needed
- **No host access** - Cannot affect the host system

## Related Commands

- [`msb run`](run.md) - Run project sandboxes
- [`msb install`](install.md) - Install sandbox as system command
- [`msb shell`](shell.md) - Open interactive shell

## Aliases

For convenience, you can use the shorter alias:

```bash
# These are equivalent
msb exe python
msx python
```

## Tips

- Use temporary sandboxes for testing untrusted code
- Set appropriate resource limits for unknown workloads
- Use `--scope none` for completely offline execution
- Mount only the files you need to minimize attack surface
- Prefer temporary sandboxes for one-off tasks

---

:bulb: **Tip**: Temporary sandboxes are perfect for safely running code from the internet or testing new libraries without affecting your system.
