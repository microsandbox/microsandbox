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

| Option | Description |
|--------|-------------|
| `--port <port>` | Port to listen on |
| `-p, --path <path>` | Namespace directory path |
| `--dev` | Run in development mode |
| `-k, --key <key>` | Set secret key |
| `-d, --detach` | Run in background |
| `-r, --reset-key` | Reset the server key |
===

==- `msb server keygen`
Generate a new API key.

```bash
msb server keygen [options]
```

| Option | Description |
|--------|-------------|
| `--expire <duration>` | Token expiration (1s, 2m, 3h, etc.) |
| `-n, --namespace <ns>` | Namespace for the API key |
===

==- `msb server status`
Show server status.

```bash
msb server status [--sandbox] [names...] [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to sandboxes |
| `-n, --namespace <ns>` | Namespace to show status for |
===

---

### Project Management

==- `msb init`
Initialize a new microsandbox project.

```bash
msb init [--file <path>]
```

| Option | Description |
|--------|-------------|
| `-f, --file <path>` | Path to the sandbox file or project directory |
===

==- `msb add`
Add a new sandbox to the project.

```bash
msb add [--sandbox] [--build] [--group] <names...> --image <image> [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-g, --group` | Apply to a group |
| `--image <image>` | Image to use |
| `--memory <MiB>` | Memory limit in MiB |
| `--cpus <count>` | Number of CPUs |
| `-v, --volume <map>` | Volume mappings (host:container) |
| `-p, --port <map>` | Port mappings (host:container) |
| `--env <KEY=VALUE>` | Environment variables |
| `--env-file <path>` | Environment file |
| `--depends-on <deps>` | Dependencies |
| `--workdir <path>` | Working directory |
| `--shell <shell>` | Shell to use |
| `--script <name=cmd>` | Scripts to add |
| `--start <cmd>` | Start script |
| `--import <name=path>` | Files to import |
| `--export <name=path>` | Files to export |
| `--scope <scope>` | Network scope (local/public/any/none) |
| `-f, --file <path>` | Path to sandbox file |
===

==- `msb remove`
Remove a sandbox from the project.

```bash
msb remove [--sandbox] [--build] [--group] <names...> [--file <path>]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-g, --group` | Apply to a group |
| `-f, --file <path>` | Path to sandbox file |
===

==- `msb list`
List sandboxes defined in the project.

```bash
msb list [--sandbox] [--build] [--group] [--file <path>]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | List sandboxes |
| `-b, --build` | List build sandboxes |
| `-g, --group` | List groups |
| `-f, --file <path>` | Path to sandbox file |
===

---

### Sandbox Operations

==- `msb run`
Run a sandbox defined in the project.

```bash
msb run [--sandbox] [--build] <NAME[~SCRIPT]> [options] [-- args...]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-f, --file <path>` | Path to sandbox file |
| `-d, --detach` | Run in background |
| `-e, --exec <cmd>` | Execute a command |
| `-- <args...>` | Additional arguments |
===

==- `msb shell`
Open a shell in a sandbox.

```bash
msb shell [--sandbox] [--build] <name> [options] [-- args...]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-f, --file <path>` | Path to sandbox file |
| `-d, --detach` | Run in background |
| `-- <args...>` | Additional arguments |
===

==- `msb exe`
Run a temporary sandbox.

```bash
msb exe [--image] <NAME[~SCRIPT]> [options] [-- args...]
```

| Option | Description |
|--------|-------------|
| `--cpus <count>` | Number of CPUs |
| `--memory <MiB>` | Memory in MB |
| `-v, --volume <map>` | Volume mappings |
| `-p, --port <map>` | Port mappings |
| `--env <KEY=VALUE>` | Environment variables |
| `--workdir <path>` | Working directory |
| `--scope <scope>` | Network scope |
| `-e, --exec <cmd>` | Execute a command |
| `-- <args...>` | Additional arguments |
===

==- `msb log`
Show logs of a build, sandbox, or group.

```bash
msb log [--sandbox] [--build] [--group] <name> [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-g, --group` | Apply to a group |
| `-f, --file <path>` | Path to sandbox file |
| `-f, --follow` | Follow the logs |
| `-t, --tail <n>` | Number of lines to show |
===

==- `msb tree`
Show tree of layers that make up a sandbox.

```bash
msb tree [--sandbox] [--build] [--group] <names...> [-L <level>]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-b, --build` | Apply to a build sandbox |
| `-g, --group` | Apply to a group |
| `-L <level>` | Maximum depth level |
===

---

### Project Lifecycle

==- `msb apply`
Start or stop project sandboxes based on configuration.

```bash
msb apply [--file <path>] [--detach]
```

| Option | Description |
|--------|-------------|
| `-f, --file <path>` | Path to sandbox file |
| `-d, --detach` | Run in background |
===

==- `msb up`
Run project sandboxes.

```bash
msb up [--sandbox] [--build] [--group] [names...] [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to sandboxes |
| `-b, --build` | Apply to build sandboxes |
| `-g, --group` | Apply to groups |
| `-f, --file <path>` | Path to sandbox file |
| `-d, --detach` | Run in background |
===

==- `msb down`
Stop project sandboxes.

```bash
msb down [--sandbox] [--build] [--group] [names...] [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to sandboxes |
| `-b, --build` | Apply to build sandboxes |
| `-g, --group` | Apply to groups |
| `-f, --file <path>` | Path to sandbox file |
===

==- `msb status`
Show statuses of running sandboxes.

```bash
msb status [--sandbox] [--build] [--group] [names...] [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to sandboxes |
| `-b, --build` | Apply to build sandboxes |
| `-g, --group` | Apply to groups |
| `-f, --file <path>` | Path to sandbox file |
===

---

### Image Management

==- `msb build`
Build images.

```bash
msb build [--build] [--sandbox] [--group] <names...> [--snapshot]
```

| Option | Description |
|--------|-------------|
| `-b, --build` | Build from build definition |
| `-s, --sandbox` | Build from sandbox |
| `-g, --group` | Build from group |
| `--snapshot` | Create a snapshot |
===

==- `msb pull`
Pull image from a registry.

```bash
msb pull [--image] [--image-group] <name> [options]
```

| Option | Description |
|--------|-------------|
| `-i, --image` | Apply to an image |
| `-G, --image-group` | Apply to an image group |
| `-L, --layer-path <path>` | Path to store layer files |
===

==- `msb push`
Push image to a registry.

```bash
msb push [--image] [--image-group] <name>
```

| Option | Description |
|--------|-------------|
| `-i, --image` | Apply to an image |
| `-G, --image-group` | Apply to an image group |
===

---

### Maintenance

==- `msb clean`
Clean cached sandbox layers, metadata, etc.

```bash
msb clean [--sandbox] [name] [options]
```

| Option | Description |
|--------|-------------|
| `-s, --sandbox` | Apply to a sandbox |
| `-u, --user` | Clean user-level caches |
| `-a, --all` | Clean all |
| `-f, --file <path>` | Path to sandbox file |
| `--force` | Force clean |
===

==- `msb self upgrade`
Upgrade microsandbox itself.

```bash
msb self upgrade
```
===

==- `msb self uninstall`
Uninstall microsandbox.

```bash
msb self uninstall
```
===
