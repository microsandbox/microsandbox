---
order: 70
icon: terminal
tags: [references]
---

# CLI Reference

Complete reference documentation for the microsandbox command-line interface.

---

### Installation

```bash
curl -sSL https://get.microsandbox.dev | sh
```

---

### Quick Start

```bash
# Start the server in development mode
msb server start --dev
```

---

### Global Options

==- Common Flags
| Flag | Description |
|------|-------------|
| `-V, --version` | Show version |
| `--error` | Show logs with error level |
| `--warn` | Show logs with warn level |
| `--info` | Show logs with info level |
| `--debug` | Show logs with debug level |
| `--trace` | Show logs with trace level |
===

---

### Server Management

==- `msb server start`
Start the sandbox server.

```bash
msb server start [options]
```

| Option              | Description              |
| ------------------- | ------------------------ |
| `--host <host>`     | Host to listen on        |
| `--port <port>`     | Port to listen on        |
| `-p, --path <path>` | Namespace directory path |
| `--dev`             | Run in development mode  |
| `-k, --key <key>`   | Set secret key           |
| `-d, --detach`      | Run in background        |
| `-r, --reset-key`   | Reset the server key     |

**Examples:**

```bash
# Start server in development mode
msb server start --dev

# Start server on custom port
msb server start --port 8080

# Start server on custom host
msb server start --host 0.0.0.0

# Start server in background with custom namespace path
msb server start --detach --path /custom/namespaces

# Start server with a specific key
msb server start --key mySecretKey123
```

===

==- `msb server keygen`
Generate a new API key.

```bash
msb server keygen [options]
```

| Option                 | Description                         |
| ---------------------- | ----------------------------------- |
| `--expire <duration>`  | Token expiration (1s, 2m, 3h, etc.) |
| `-n, --namespace <ns>` | Namespace for the API key           |

**Examples:**

```bash
# Generate an API key for all namespaces
msb server keygen

# Generate an API key that expires in 24 hours, for all namespaces
msb server keygen --expire 24h

# Generate an API key for a specific namespace
msb server keygen --namespace production

# Generate a short-lived key for testing
msb server keygen --expire 30m --namespace testing
```

===

==- `msb server status`
Show server status.

```bash
msb server status [--sandbox] [names...] [options]
```

| Option                 | Description                  |
| ---------------------- | ---------------------------- |
| `-s, --sandbox`        | Apply to sandboxes           |
| `-n, --namespace <ns>` | Namespace to show status for |

**Examples:**

```bash
# Show status for all sandboxes
msb server status

# Show status for specific sandboxes
msb server status app database

# Show status for sandboxes in a specific namespace
msb server status --namespace production
```

===

---

### Project Management

==- `msb init`
Initialize a new microsandbox project.

```bash
msb init [--file <path>]
```

| Option              | Description                                   |
| ------------------- | --------------------------------------------- |
| `-f, --file <path>` | Path to the sandbox file or project directory |

**Examples:**

```bash
# Initialize project in current directory
msb init


# Initialize project in a specific directory
msb init --file /path/to/project/

# Initialize project with custom sandbox file location
msb init --file ./path/to/Sandboxfile
```

===

==- `msb add`
Add a new sandbox to a project.

```bash
msb add [--sandbox] [--build] [--group] <names...> --image <image> [options]
```

| Option                 | Description                           |
| ---------------------- | ------------------------------------- |
| `-s, --sandbox`        | Apply to a sandbox (default)          |
| `-b, --build`          | Apply to a build sandbox              |
| `-g, --group`          | Apply to a group                      |
| `--image <image>`      | Image to use                          |
| `--memory <MiB>`       | Memory limit in MiB                   |
| `--cpus <count>`       | Number of CPUs                        |
| `-v, --volume <map>`   | Volume mappings (host:container)      |
| `-p, --port <map>`     | Port mappings (host:container)        |
| `--env <KEY=VALUE>`    | Environment variables                 |
| `--env-file <path>`    | Environment file                      |
| `--depends-on <deps>`  | Dependencies                          |
| `--workdir <path>`     | Working directory                     |
| `--shell <shell>`      | Shell to use                          |
| `--script <name=cmd>`  | Scripts to add                        |
| `--start <cmd>`        | Start script                          |
| `--import <name=path>` | Files to import                       |
| `--export <name=path>` | Files to export                       |
| `--scope <scope>`      | Network scope (local/public/any/none) |
| `-f, --file <path>`    | Path to sandbox file                  |

**Examples:**

```bash
# Add a simple sandbox
msb add --sandbox app --image node:18

# Add a simple sandbox (--sandbox is default)
msb add app --image node:18

# Add a sandbox with port mapping and environment variables
msb add web --image nginx:alpine --port 8080:80 --env NODE_ENV=production

# Add a sandbox with volume mounts and resource limits
msb add database --image postgres:15 --volume ./data:/var/lib/postgresql/data --memory 512 --cpus 2

# Add a sandbox with dependencies and custom scripts
msb add api --image my/api --depends-on database --script test="pytest" --start "python app.py"
```

===

==- `msb remove`
Remove a sandbox from the project.

```bash
msb remove [--sandbox] [--build] [--group] <names...> [--file <path>]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to a sandbox (default) |
| `-b, --build`       | Apply to a build sandbox     |
| `-g, --group`       | Apply to a group             |
| `-f, --file <path>` | Path to sandbox file         |

**Examples:**

```bash
# Remove a sandbox
msb remove --sandbox app

# Remove a sandbox (--sandbox is default)
msb remove app

# Remove multiple sandboxes
msb remove web api database

# Remove from a specific sandbox file
msb remove app --file ./path/to/Sandboxfile
```

===

==- `msb list`
List sandboxes defined in a project.

```bash
msb list [--sandbox] [--build] [--group] [--file <path>]
```

| Option              | Description              |
| ------------------- | ------------------------ |
| `-s, --sandbox`     | List sandboxes (default) |
| `-b, --build`       | List build sandboxes     |
| `-g, --group`       | List groups              |
| `-f, --file <path>` | Path to sandbox file     |

**Examples:**

```bash
# List all sandboxes
msb list --sandbox

# List all sandboxes (--sandbox is default)
msb list

# List from a specific sandbox file
msb list --file ./config/sandbox.yaml
```

===

---

### Sandbox Operations

==- `msb run`
Run a sandbox defined in the project.

```bash
msb run [--sandbox] [--build] <NAME[~SCRIPT]> [options] [-- args...]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to a sandbox (default) |
| `-b, --build`       | Apply to a build sandbox     |
| `-f, --file <path>` | Path to sandbox file         |
| `-d, --detach`      | Run in background            |
| `-e, --exec <cmd>`  | Execute a command            |
| `-- <args...>`      | Additional arguments         |

**Examples:**

```bash
# Run a sandbox
msb run --sandbox app

# Run a sandbox (--sandbox is default)
msb run app

# Run a specific script in a sandbox
msb run app~test

# Run a specific script in a sandbox with additional arguments
msb run app~test -- -a 1 -b 2

# Run in background
msb run app --detach

# Execute a command within a sandbox
msb run app --exec bash

# Execute a command within a sandbox with additional arguments
msb run app --exec bash -- -c "echo 'Hello, World!'"
```

===

==- `msb shell`
Open a shell in a sandbox. This opens whatever shell is configured for the sandbox image.

```bash
msb shell [--sandbox] [--build] <name> [options] [-- args...]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to a sandbox (default) |
| `-b, --build`       | Apply to a build sandbox     |
| `-f, --file <path>` | Path to sandbox file         |
| `-d, --detach`      | Run in background            |
| `-- <args...>`      | Additional arguments         |

**Examples:**

```bash
# Open shell in a sandbox
msb shell --sandbox app

# Open shell in a sandbox (--sandbox is default)
msb shell app

# Open shell with additional arguments
msb shell app -- -c "echo 'Hello, World!'"
```

===

==- `msb exe`
Run a temporary sandbox. This sandbox will be removed after it exits.

```bash
msb exe [--image] <NAME[~SCRIPT]> [options] [-- args...]
```

| Option               | Description           |
| -------------------- | --------------------- |
| `--cpus <count>`     | Number of CPUs        |
| `--memory <MiB>`     | Memory in MB          |
| `-v, --volume <map>` | Volume mappings       |
| `-p, --port <map>`   | Port mappings         |
| `--env <KEY=VALUE>`  | Environment variables |
| `--workdir <path>`   | Working directory     |
| `--scope <scope>`    | Network scope         |
| `-e, --exec <cmd>`   | Execute a command     |
| `-- <args...>`       | Additional arguments  |

**Examples:**

```bash
# Run a temporary sandbox with an image (--image is default)
msb exe python:3.11

# Run with resource limits and volume mounts
msb exe ubuntu:22.04 --memory 256 --cpus 1 --volume ./code:/workspace

# Execute a specific command
msb exe node:18 --exec npm -- test

# Run with environment variables and port mapping
msb exe nginx:alpine --env NODE_ENV=production --port 8080:80

# Pass additional arguments
msb exe python:3.11 -- -c "print('Hello World')"
```

===

==- `msb log`
Show logs of a build, sandbox, or group.

```bash
msb log [--sandbox] [--build] [--group] <name> [options]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to a sandbox (default) |
| `-b, --build`       | Apply to a build sandbox     |
| `-g, --group`       | Apply to a group             |
| `-f, --file <path>` | Path to sandbox file         |
| `-f, --follow`      | Follow the logs              |
| `-t, --tail <n>`    | Number of lines to show      |

**Examples:**

```bash
# Show logs for a sandbox (--sandbox is default)
msb log app

# Follow logs in real-time
msb log app --follow

# Show last 50 lines
msb log app --tail 50
```

===

==- `msb tree`
Show tree of layers that make up a sandbox.

!!!warning Coming Soon
This will be available in a future release.
!!!

```bash
msb tree [--sandbox] [--build] [--group] <names...> [-L <level>]
```

| Option          | Description                  |
| --------------- | ---------------------------- |
| `-s, --sandbox` | Apply to a sandbox (default) |
| `-b, --build`   | Apply to a build sandbox     |
| `-g, --group`   | Apply to a group             |
| `-L <level>`    | Maximum depth level          |

**Examples:**

```bash
# Show tree for a sandbox (--sandbox is default)
msb tree app

# Show tree for multiple sandboxes
msb tree app database

# Limit tree depth
msb tree app -L 3
```

===

---

### Project Lifecycle

==- `msb apply`
Start or stop project sandboxes based on configuration.

```bash
msb apply [--file <path>] [--detach]
```

| Option              | Description          |
| ------------------- | -------------------- |
| `-f, --file <path>` | Path to sandbox file |
| `-d, --detach`      | Run in background    |

**Examples:**

```bash
# Apply current project configuration
msb apply

# Apply in background
msb apply --detach

# Apply specific sandbox file
msb apply --file ./path/to/Sandboxfile
```

===

==- `msb up`
Run project sandboxes.

```bash
msb up [--sandbox] [--build] [--group] [names...] [options]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to sandboxes (default) |
| `-b, --build`       | Apply to build sandboxes     |
| `-g, --group`       | Apply to groups              |
| `-f, --file <path>` | Path to sandbox file         |
| `-d, --detach`      | Run in background            |

**Examples:**

```bash
# Start all sandboxes
msb up --sandbox

# Start all sandboxes (--sandbox is default)
msb up

# Start specific sandboxes
msb up app database

# Start in background
msb up --detach
```

===

==- `msb down`
Stop project sandboxes.

```bash
msb down [--sandbox] [--build] [--group] [names...] [options]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to sandboxes (default) |
| `-b, --build`       | Apply to build sandboxes     |
| `-g, --group`       | Apply to groups              |
| `-f, --file <path>` | Path to sandbox file         |

**Examples:**

```bash
# Stop all sandboxes
msb down --sandbox

# Stop all sandboxes (--sandbox is default)
msb down

# Stop specific sandboxes
msb down app database

# Stop from specific sandbox file
msb down --file ./path/to/Sandboxfile
```

===

==- `msb status`
Show statuses of running sandboxes.

```bash
msb status [--sandbox] [--build] [--group] [names...] [options]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to sandboxes (default) |
| `-b, --build`       | Apply to build sandboxes     |
| `-g, --group`       | Apply to groups              |
| `-f, --file <path>` | Path to sandbox file         |

**Examples:**

```bash
# Show status of all sandboxes
msb status --sandbox

# Show status of all sandboxes (--sandbox is default)
msb status

# Show status of specific sandboxes
msb status app database

# Show status from specific sandbox file
msb status --file ./path/to/Sandboxfile
```

===

---

### Image Management

==- `msb build`
Build images.

!!!warning Coming Soon
This will be available in a future release.
!!!

```bash
msb build [--build] [--sandbox] [--group] <names...> [--snapshot]
```

| Option          | Description                  |
| --------------- | ---------------------------- |
| `-b, --build`   | Build from build definition  |
| `-s, --sandbox` | Build from sandbox (default) |
| `-g, --group`   | Build from group             |
| `--snapshot`    | Create a snapshot            |

**Examples:**

```bash
# Build from sandbox
msb build app --sandbox

# Build from sandbox (--sandbox is default)
msb build app

# Build multiple sandboxes
msb build app database

# Create a snapshot while building
msb build app --snapshot
```

===

==- `msb pull`
Pull image from a registry.

```bash
msb pull [--image] [--image-group] <name> [options]
```

| Option                    | Description                 |
| ------------------------- | --------------------------- |
| `-i, --image`             | Apply to an image (default) |
| `-G, --image-group`       | Apply to an image group     |
| `-L, --layer-path <path>` | Path to store layer files   |

**Examples:**

```bash
# Pull an image
msb pull --image python:3.11

# Pull an image (--image is default)
msb pull python:3.11

# Pull with custom layer storage path
msb pull ubuntu:22.04 --layer-path /custom/layers
```

===

==- `msb push`
Push image to a registry.

!!!warning Coming Soon
This will be available in a future release.
!!!

```bash
msb push [--image] [--image-group] <name>
```

| Option              | Description                 |
| ------------------- | --------------------------- |
| `-i, --image`       | Apply to an image (default) |
| `-G, --image-group` | Apply to an image group     |

**Examples:**

```bash
# Push an image
msb push --image myapp:latest

# Push an image (--image is default)
msb push myapp:latest
```

===

---

### Maintenance

==- `msb clean`
Clean cached sandbox layers, metadata, etc.

```bash
msb clean [--sandbox] [name] [options]
```

| Option              | Description                  |
| ------------------- | ---------------------------- |
| `-s, --sandbox`     | Apply to a sandbox (default) |
| `-u, --user`        | Clean user-level caches      |
| `-a, --all`         | Clean all                    |
| `-f, --file <path>` | Path to sandbox file         |
| `--force`           | Force clean                  |

**Examples:**

```bash
# Clean specific sandbox
msb clean --sandbox app

# Clean specific sandbox (--sandbox is default)
msb clean app

# Clean all caches
msb clean --all

# Clean user-level caches
msb clean --user

# Force clean without confirmation
msb clean app --force
```

===

==- `msb self upgrade`
Upgrade microsandbox itself.

!!!warning Coming Soon
This will be available in a future release.
!!!

```bash
msb self upgrade
```

**Examples:**

```bash
# Upgrade to the latest version
msb self upgrade
```

===

==- `msb self uninstall`
Uninstall microsandbox.

```bash
msb self uninstall
```

**Examples:**

```bash
# Uninstall microsandbox
msb self uninstall
```

===
