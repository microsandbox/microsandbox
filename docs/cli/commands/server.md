---
icon: server
title: msb server
description: Manage the Microsandbox server
order: 220
---

# msb server

Manage the Microsandbox server that orchestrates sandbox environments.

## Usage

```bash
msb server <SUBCOMMAND> [OPTIONS]
```

## Description

The `msb server` command group provides functionality to start, stop, and manage the Microsandbox server. The server is responsible for creating and managing sandbox environments, handling API requests, and orchestrating container lifecycles.

## Subcommands

### `msb server start`

Start the Microsandbox server.

```bash
msb server start [OPTIONS]
```

**Options:**

| Option          | Description                            |
| --------------- | -------------------------------------- |
| `--dev`         | Development mode (no API key required) |
| `--detach`      | Run in background                      |
| `--port <PORT>` | Server port (default: 5555)            |
| `--host <HOST>` | Bind address (default: 127.0.0.1)      |

**Examples:**

```bash
# Start in development mode
msb server start --dev

# Start in background
msb server start --dev --detach

# Start on custom port
msb server start --port 8080

# Start with external access
msb server start --host 0.0.0.0
```

### `msb server stop`

Stop the Microsandbox server.

```bash
msb server stop
```

**Examples:**

```bash
# Stop the server
msb server stop
```

### `msb server status`

Check server status and health.

```bash
msb server status
```

**Examples:**

```bash
# Check if server is running
msb server status
```

### `msb server keygen`

Generate a new API key for authentication.

```bash
msb server keygen [OPTIONS]
```

**Options:**

| Option                | Description                         |
| --------------------- | ----------------------------------- |
| `--expire <DURATION>` | Key expiration (e.g., 30d, 3mo, 1y) |

**Examples:**

```bash
# Generate key with 3 month expiration
msb server keygen --expire 3mo

# Generate key with 1 year expiration
msb server keygen --expire 1y

# Generate key with 30 day expiration
msb server keygen --expire 30d
```

### `msb server log`

Show server logs.

```bash
msb server log [OPTIONS]
```

**Examples:**

```bash
# Show recent logs
msb server log

# Follow logs in real-time
msb server log --follow
```

### `msb server list`

List running sandboxes on the server.

```bash
msb server list [OPTIONS]
```

**Examples:**

```bash
# List all running sandboxes
msb server list

# List with detailed information
msb server list --verbose
```

### `msb server ssh`

SSH into a running sandbox.

```bash
msb server ssh <SANDBOX_ID>
```

**Examples:**

```bash
# SSH into a specific sandbox
msb server ssh sandbox-abc123
```

## Development vs Production Mode

### Development Mode (`--dev`)

- **No API key required** - Simplifies local development
- **Relaxed security** - Suitable for trusted environments
- **Easy setup** - No authentication configuration needed

```bash
msb server start --dev
```

### Production Mode

- **API key required** - Enhanced security
- **Full authentication** - Protects against unauthorized access
- **Audit logging** - Track API usage

```bash
# Start production server
msb server start

# Generate API key
msb server keygen --expire 3mo

# Set environment variable
export MSB_API_KEY=msb_your_generated_key_here
```

## Configuration

### Environment Variables

Configure the server using environment variables:

```bash
# Server configuration
export MSB_SERVER_HOST="0.0.0.0"
export MSB_SERVER_PORT="5555"

# Authentication
export MSB_API_KEY="your_api_key"

# Logging
export MSB_LOG_LEVEL="info"
```

### Network Configuration

| Bind Address | Access Level       | Use Case                  |
| ------------ | ------------------ | ------------------------- |
| `127.0.0.1`  | Localhost only     | Local development         |
| `0.0.0.0`    | All interfaces     | Remote access, production |
| `10.0.0.1`   | Specific interface | Restricted network access |

## Security Considerations

### API Key Management

- Generate keys with appropriate expiration times
- Rotate keys regularly in production
- Store keys securely (environment variables, secret managers)
- Never commit keys to version control

### Network Security

- Use `127.0.0.1` for local-only access
- Configure firewalls when binding to `0.0.0.0`
- Consider reverse proxies for HTTPS termination
- Monitor access logs in production

## Troubleshooting

### Server Won't Start

```bash
# Check if port is already in use
lsof -i :5555

# Check server logs
msb server log

# Try a different port
msb server start --port 5556
```

### Permission Errors (Linux)

```bash
# Ensure user is in required groups
groups $USER

# Should include: kvm, libvirt
sudo usermod -aG kvm $USER
sudo usermod -aG libvirt $USER
```

### Connection Issues

```bash
# Check server status
msb server status

# Verify network connectivity
curl http://127.0.0.1:5555/health

# Check firewall settings
sudo ufw status
```

## Related Commands

- [`msb pull`](pull.md) - Pull sandbox images
- [`msb run`](run.md) - Run project sandboxes
- [`msb exe`](exe.md) - Run temporary sandboxes

## Examples

### Complete Setup Workflow

```bash
# 1. Start server in development mode
msb server start --dev --detach

# 2. Check server status
msb server status

# 3. Pull required images
msb pull microsandbox/python
msb pull microsandbox/node

# 4. Test with a temporary sandbox
msb exe python --exec "print('Server is working!')"
```

### Production Deployment

```bash
# 1. Start production server
msb server start --host 0.0.0.0 --detach

# 2. Generate API key
API_KEY=$(msb server keygen --expire 3mo)

# 3. Configure client
export MSB_API_KEY="$API_KEY"

# 4. Test connection
msb server status
```

## Tips

- Use `--dev` mode for local development
- Always use API keys in production
- Monitor server logs for issues
- Set appropriate resource limits for your environment
- Use `--detach` for background operation

---

:bulb: **Tip**: The server also functions as an MCP (Model Context Protocol) server, making it compatible with AI tools like Cursor and Claude.
