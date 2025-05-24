---
icon: plus
title: msb add
description: Add a new sandbox to the project configuration
order: 211
---

# msb add

Add a new sandbox to the project configuration.

## Usage

```bash
msb add <NAMES>... --image <IMAGE> [OPTIONS]
```

## Description

The `msb add` command registers a new sandbox in your project's `Sandboxfile`. This allows you to define reusable sandbox environments with specific configurations, resource limits, and startup scripts.

## Arguments

| Argument     | Description                           |
| ------------ | ------------------------------------- |
| `<NAMES>...` | One or more names for the sandbox(es) |

## Required Options

| Option            | Description            |
| ----------------- | ---------------------- |
| `--image <IMAGE>` | Container image to use |

## Optional Configuration

| Option                   | Description                            |
| ------------------------ | -------------------------------------- |
| `--memory <MEMORY>`      | Memory limit in MB                     |
| `--cpus <CPUS>`          | CPU allocation (e.g., 1.5)             |
| `--env <ENV>`            | Environment variables (KEY=VALUE)      |
| `--port <PORT>`          | Port mappings (HOST:CONTAINER)         |
| `--volume <VOLUME>`      | Volume mappings (HOST:CONTAINER)       |
| `--workdir <WORKDIR>`    | Working directory inside sandbox       |
| `--start <START>`        | Default start script                   |
| `--depends-on <DEPENDS>` | Sandbox dependencies (comma-separated) |

## Examples

### Basic Sandbox

```bash
# Add a simple Python sandbox
msb add app --image python
```

### Application Server

```bash
# Add a Python web application
msb add web --image python \
    --memory 1024 \
    --cpus 2 \
    --env "PORT=8000" \
    --port "8000:8000" \
    --volume "./src:/workspace" \
    --workdir "/workspace" \
    --start "python app.py"
```

### Database Service

```bash
# Add a PostgreSQL database
msb add db --image postgres:15 \
    --memory 512 \
    --env "POSTGRES_DB=myapp" \
    --env "POSTGRES_USER=user" \
    --env "POSTGRES_PASSWORD=password" \
    --port "5432:5432" \
    --volume "./data:/var/lib/postgresql/data"
```

### Multiple Sandboxes

```bash
# Add multiple related sandboxes
msb add frontend backend --image node \
    --memory 1024 \
    --port "3000:3000"
```

### With Dependencies

```bash
# Add a web service that depends on a database
msb add api --image python \
    --depends-on db \
    --env "DATABASE_URL=postgresql://user:password@db:5432/myapp" \
    --start "uvicorn main:app --host 0.0.0.0"
```

## Configuration File

After running `msb add`, your `Sandboxfile` will be updated:

```yaml
sandboxes:
  app:
    image: python
    memory: 1024
    cpus: 2
    environment:
      PORT: "8000"
    ports:
      - "8000:8000"
    volumes:
      - "./src:/workspace"
    workdir: "/workspace"
    scripts:
      start: python app.py
    depends_on:
      - db
```

## Environment Variables

Environment variables can be set in multiple ways:

```bash
# Single variable
msb add app --image python --env "DEBUG=true"

# Multiple variables
msb add app --image python --env "DEBUG=true" --env "PORT=8000"

# Using shell variables
msb add app --image python --env "API_KEY=${API_KEY}"
```

## Volume Mounts

Volume syntax supports various patterns:

```bash
# Bind mount (host:container)
--volume "./src:/workspace"

# Named volume
--volume "data:/var/lib/data"

# Read-only mount
--volume "./config:/etc/config:ro"
```

## Port Mappings

Port mapping syntax:

```bash
# Basic mapping (host:container)
--port "8000:8000"

# Different ports
--port "80:8000"

# Multiple ports
--port "80:8000" --port "443:8443"

# Container port only (random host port)
--port "8000"
```

## Related Commands

- [`msb init`](init.md) - Initialize a new project
- [`msb run`](run.md) - Run project sandboxes
- [`msb remove`](remove.md) - Remove sandboxes from project
- [`msb list`](list.md) - List project sandboxes

## Tips

- Use descriptive names for your sandboxes
- Set appropriate resource limits based on your needs
- Use environment variables for configuration that changes between environments
- Consider dependencies when designing multi-service applications

---

:bulb: **Tip**: Run `msb add --help` to see all available options and their descriptions.
