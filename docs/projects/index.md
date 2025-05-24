---
icon: package
title: Projects
description: Package-manager-like workflow for sandbox development
order: 400
---

# Projects

The `msb` CLI brings the familiar feel of package managers to sandbox development. Think of it like npm or cargo, but for sandboxes! Create a Sandboxfile, define your environments, and manage your sandboxes with simple commands.

## Overview

Projects provide a structured way to manage multiple sandbox environments with:

- **Sandboxfile** - Configuration manifest for your environments
- **Persistent State** - Environments maintain state between sessions
- **Resource Management** - Configure memory, CPU, and other resources
- **Script Management** - Define and run custom scripts
- **Dependency Management** - Handle inter-sandbox dependencies

## Quick Start

### 1. Initialize a Project

Create a new project in your current directory:

```bash
msb init
```

This creates a `Sandboxfile` configuration manifest for managing sandbox environments.

### 2. Add Sandboxes

Add sandboxes to your project with specific configurations:

```bash
# Add a Python application sandbox
msb add app \
    --image python \
    --cpus 1 \
    --memory 1024 \
    --start 'python app.py'

# Add a database sandbox
msb add db \
    --image postgres \
    --memory 512 \
    --env "POSTGRES_DB=myapp" \
    --port "5432:5432"

# Add a web server
msb add web \
    --image node \
    --memory 1024 \
    --port "3000:3000" \
    --depends-on app,db
```

### 3. View Your Configuration

Check your `Sandboxfile`:

```bash
cat Sandboxfile
```

```yaml
# Sandbox configurations
sandboxes:
  app:
    image: python
    memory: 1024
    cpus: 1
    scripts:
      start: python app.py

  db:
    image: postgres
    memory: 512
    environment:
      POSTGRES_DB: myapp
    ports:
      - "5432:5432"

  web:
    image: node
    memory: 1024
    ports:
      - "3000:3000"
    depends_on:
      - app
      - db
```

### 4. Run Your Sandboxes

Execute sandboxes defined in your project:

```bash
# Run specific sandbox
msb run app

# Run with custom script
msb run app~test

# Run all sandboxes
msb up
```

## Project Structure

### Sandboxfile

The `Sandboxfile` is the heart of your project, defining all sandbox configurations:

```yaml
# Project metadata
name: my-project
version: "1.0.0"

# Sandbox definitions
sandboxes:
  app:
    image: python:3.11
    memory: 1024
    cpus: 2
    environment:
      DEBUG: "true"
      API_KEY: "${API_KEY}"
    volumes:
      - "./src:/workspace/src"
    ports:
      - "8000:8000"
    scripts:
      start: python src/main.py
      test: pytest tests/
      dev: python src/main.py --debug
    depends_on:
      - db

  db:
    image: postgres:15
    memory: 512
    environment:
      POSTGRES_DB: myapp
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes:
      - "./data:/var/lib/postgresql/data"
    ports:
      - "5432:5432"

# Build configurations (optional)
builds:
  custom-app:
    context: ./docker
    dockerfile: Dockerfile.app
```

### Environment Directory

When running project sandboxes, all changes are persisted to the `./menv` directory:

```
./menv/
├── app/          # App sandbox state
├── db/           # Database sandbox state
└── web/          # Web sandbox state
```

This means you can stop and restart sandboxes without losing your work.

## Command Reference

### Project Management

```bash
# Initialize new project
msb init

# Add sandbox to project
msb add <name> --image <image> [options]

# Remove sandbox from project
msb remove <name>

# List project sandboxes
msb list
```

### Running Sandboxes

```bash
# Run specific sandbox
msb run <name>

# Run with specific script
msb run <name>~<script>

# Run with custom command
msb run <name> --exec "custom command"

# Run in background
msb run <name> --detach
```

### Project Operations

```bash
# Start all sandboxes
msb up

# Stop all sandboxes
msb down

# Check status
msb status

# Apply configuration changes
msb apply
```

## Configuration Options

### Sandbox Configuration

Each sandbox in your `Sandboxfile` can be configured with:

| Option        | Description            | Example                |
| ------------- | ---------------------- | ---------------------- |
| `image`       | Container image to use | `python:3.11`          |
| `memory`      | Memory limit in MB     | `1024`                 |
| `cpus`        | CPU allocation         | `2.0`                  |
| `environment` | Environment variables  | `DEBUG: "true"`        |
| `volumes`     | Volume mounts          | `"./src:/workspace"`   |
| `ports`       | Port mappings          | `"8000:8000"`          |
| `scripts`     | Named scripts          | `start: python app.py` |
| `depends_on`  | Dependencies           | `[db, cache]`          |
| `workdir`     | Working directory      | `/workspace`           |
| `shell`       | Default shell          | `/bin/bash`            |

### Environment Variables

Use environment variable substitution:

```yaml
sandboxes:
  app:
    environment:
      API_KEY: "${API_KEY}"
      DEBUG: "${DEBUG:-false}"
      DATABASE_URL: "postgres://user:pass@db:5432/myapp"
```

### Volume Mounts

Mount local directories into sandboxes:

```yaml
sandboxes:
  app:
    volumes:
      - "./src:/workspace/src" # Source code
      - "./data:/workspace/data" # Data directory
      - "~/.ssh:/root/.ssh:ro" # SSH keys (read-only)
```

### Port Mappings

Expose sandbox ports to your host:

```yaml
sandboxes:
  web:
    ports:
      - "3000:3000" # HTTP server
      - "3001:3001" # WebSocket server
      - "127.0.0.1:8080:80" # Bind to specific interface
```

## Advanced Features

### Dependencies

Define startup order with dependencies:

```yaml
sandboxes:
  db:
    image: postgres
    # ... configuration

  cache:
    image: redis
    # ... configuration

  app:
    image: python
    depends_on:
      - db
      - cache
    # ... configuration
```

### Custom Scripts

Define reusable scripts for common tasks:

```yaml
sandboxes:
  app:
    scripts:
      start: python src/main.py
      test: pytest tests/ -v
      lint: flake8 src/
      dev: |
        pip install -e .
        python src/main.py --debug
      migrate: python manage.py migrate
```

Run scripts with:

```bash
msb run app~test    # Run tests
msb run app~lint    # Run linter
msb run app~dev     # Development mode
```

### Build Configurations

Define custom images in your project:

```yaml
builds:
  my-app:
    context: ./docker
    dockerfile: Dockerfile
    args:
      PYTHON_VERSION: "3.11"

sandboxes:
  app:
    image: my-app # Use custom build
```

## Workflow Examples

### Web Development

```yaml
sandboxes:
  frontend:
    image: node:18
    ports: ["3000:3000"]
    volumes: ["./frontend:/workspace"]
    scripts:
      start: npm run dev
      build: npm run build
      test: npm test

  backend:
    image: python:3.11
    ports: ["8000:8000"]
    volumes: ["./backend:/workspace"]
    environment:
      DATABASE_URL: "postgres://user:pass@db:5432/app"
    scripts:
      start: uvicorn main:app --reload
      test: pytest
    depends_on: [db]

  db:
    image: postgres:15
    environment:
      POSTGRES_DB: app
      POSTGRES_USER: user
      POSTGRES_PASSWORD: password
    volumes: ["./data:/var/lib/postgresql/data"]
```

### Data Science

```yaml
sandboxes:
  jupyter:
    image: jupyter/datascience-notebook
    ports: ["8888:8888"]
    volumes:
      - "./notebooks:/home/jovyan/work"
      - "./data:/home/jovyan/data"
    scripts:
      start: jupyter lab --ip=0.0.0.0 --allow-root

  processing:
    image: python:3.11
    memory: 4096
    volumes: ["./scripts:/workspace", "./data:/data"]
    scripts:
      process: python process_data.py
      train: python train_model.py
```

## Best Practices

### Project Organization

```
my-project/
├── Sandboxfile           # Project configuration
├── menv/                 # Sandbox state (auto-generated)
├── src/                  # Source code
├── data/                 # Data files
├── scripts/              # Utility scripts
└── README.md            # Project documentation
```

### Configuration Management

- Use environment variables for secrets
- Keep sensitive data out of the Sandboxfile
- Use `.env` files for local development
- Document required environment variables

### Resource Management

- Set appropriate memory limits
- Use CPU limits for resource-intensive tasks
- Monitor resource usage with `msb status`
- Clean up unused resources with `msb clean`

## Troubleshooting

### Common Issues

#### Sandbox Won't Start

```bash
# Check configuration
msb list

# View logs
msb log <sandbox-name>

# Check dependencies
msb status
```

#### Port Conflicts

```bash
# Check what's using the port
lsof -i :3000

# Use different port mapping
msb add app --port "3001:3000"
```

#### Volume Mount Issues

```bash
# Check permissions
ls -la ./src

# Use absolute paths
volumes: ["/absolute/path:/workspace"]
```

## Next Steps

- [**CLI Reference**](../cli/index.md) - Detailed command documentation
- [**Examples**](../examples/index.md) - Project examples and templates

---

:bulb: **Tip**: Use `msb <command> --help` to see all available options for project commands.
