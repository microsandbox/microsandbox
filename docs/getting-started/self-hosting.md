---
icon: server
title: Self-Hosting
description: Set up your own Microsandbox server
order: 102
---

# Self-Hosting

To get started with Microsandbox, you need to host your own sandbox server. Whether that's on a local machine or in the cloud, it's up to you.

Self-hosting lets you manage your own data and code, making it easier to comply with security policies. Having a sandbox server set up locally also allows you to test and move through ideas quickly.

## Prerequisites

Before starting, ensure you have:

- [Microsandbox CLI installed](installation.md)
- Supported platform (macOS with Apple Silicon or Linux with KVM)
- At least 4GB RAM and 2GB free storage

<div align='center'>• • •</div>

## Quick Setup

### 1. Start the Server

Start your sandbox server in development mode (no API key required):

```bash
msb server start --dev
```

For production use, start without the `--dev` flag:

```bash
msb server start
```

!!!tip "Background Mode"
Use the `--detach` flag to run the server in the background:

```bash
msb server start --dev --detach
```

!!!

### 2. Pull SDK Images

Download the sandbox images you'll need for the SDKs:

```bash
# Python sandbox image
msb pull microsandbox/python

# Node.js sandbox image
msb pull microsandbox/node
```

This pulls and caches the images, allowing you to run `PythonSandbox` or `NodeSandbox` instances.

### 3. Generate API Key (Production)

If you started the server without `--dev` mode, generate an API key:

```bash
msb server keygen --expire 3mo
```

Set the generated key as an environment variable:

```bash
export MSB_API_KEY=msb_your_generated_key_here
```

<div align='center'>• • •</div>

## Server Management

### Check Server Status

```bash
msb server status
```

### View Server Logs

```bash
msb server log
```

### Stop the Server

```bash
msb server stop
```

### List Running Sandboxes

```bash
msb server list
```

<div align='center'>• • •</div>

## Configuration Options

### Server Start Options

The `msb server start` command supports several options:

```bash
msb server start [OPTIONS]
```

Common options:

- `--dev` - Skip API key requirement (development mode)
- `--detach` - Run in background
- `--port <PORT>` - Custom port (default: 5555)
- `--host <HOST>` - Bind address (default: 127.0.0.1)

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

<div align='center'>• • •</div>

## Security Considerations

### Development vs Production

| Mode                      | API Key      | Use Case                       |
| ------------------------- | ------------ | ------------------------------ |
| **Development** (`--dev`) | Not required | Local testing, development     |
| **Production**            | Required     | Shared servers, production use |

### API Key Management

- Generate keys with appropriate expiration times
- Rotate keys regularly in production
- Store keys securely (environment variables, secret managers)
- Never commit keys to version control

### Network Security

- By default, the server binds to `127.0.0.1` (localhost only)
- For remote access, bind to `0.0.0.0` but ensure proper firewall rules
- Consider using reverse proxies (nginx, Caddy) for HTTPS termination

<div align='center'>• • •</div>

## Cloud Hosting

For cloud deployment, consider these general approaches:

- **AWS EC2** - Elastic Compute Cloud instances
- **Google Cloud Compute Engine** - Virtual machine instances
- **DigitalOcean Droplets** - Simple cloud servers
- **Azure Virtual Machines** - Scalable compute resources

Ensure your cloud instance meets the platform requirements and has sufficient resources.

<div align='center'>• • •</div>

## Troubleshooting

### Common Issues

#### Server Won't Start

```bash
# Check if port is already in use
lsof -i :5555

# Check server logs
msb server log
```

#### Permission Errors (Linux)

```bash
# Ensure user is in required groups
groups $USER

# Should include: kvm, libvirt
```

#### Image Pull Failures

```bash
# Check network connectivity
curl -I https://registry.microsandbox.dev

# Try pulling with verbose logging
msb pull microsandbox/python --debug
```

### Getting Help

If you encounter issues:

1. Check server logs: `msb server log`
2. Verify system requirements
3. Search [GitHub issues](https://github.com/microsandbox/microsandbox/issues)
4. Ask in [Discord](https://discord.gg/T95Y3XnEAK)

<div align='center'>• • •</div>

## What's Next?

With your server running:

1. [**Install an SDK**](../sdks/index.md) - Start building with sandboxes
2. [**CLI Reference**](../cli/index.md) - Learn about CLI commands
3. [**Examples**](../examples/index.md) - See practical examples

---

!!!info "Production Deployment"
For production deployments, consider using process managers like systemd, Docker, or Kubernetes.
!!!
