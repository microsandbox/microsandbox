---
order: 75
icon: key
tags: [guide]
---

# API Keys

Learn how to generate, manage, and use API keys for authenticating with the microsandbox server.

---

### Overview

API keys provide secure authentication for accessing the microsandbox server. They are required for all API operations unless the server is running in development mode.

---

### Generating API Keys

#### Basic Key Generation

Generate a new API key with default settings:

```bash
msb server keygen
```

This creates an API key with:
- **No expiration** (permanent until manually revoked)
- **Global access** to all namespaces
- **Full permissions** for all sandbox operations

#### Key with Expiration

Generate an API key that expires after a specific duration:

```bash
msb server keygen --expire 3mo
```

**Supported Duration Formats:**
- `s` - seconds (e.g., `30s`)
- `m` - minutes (e.g., `15m`)
- `h` - hours (e.g., `24h`)
- `d` - days (e.g., `7d`)
- `w` - weeks (e.g., `2w`)
- `mo` - months (e.g., `3mo`)
- `y` - years (e.g., `1y`)

**Examples:**
```bash
# Expires in 1 hour
msb server keygen --expire 1h

# Expires in 7 days
msb server keygen --expire 7d

# Expires in 6 months
msb server keygen --expire 6mo
```

#### Namespace-Specific Keys

Generate an API key limited to a specific namespace:

```bash
msb server keygen --namespace team-alpha
```

This key will only have access to sandboxes running in the `team-alpha` namespace.

#### Combined Options

Generate a namespace-specific key with expiration:

```bash
msb server keygen --expire 1w --namespace project-web
```

---

### Using API Keys

#### Environment Variable (Recommended)

Set the API key as an environment variable:

```bash
export MSB_API_KEY="your-api-key-here"
```

Add this to a `.env` file for persistence:

```bash
echo 'MSB_API_KEY="your-api-key-here"' >> .env
```

#### SDK Configuration

**Python SDK:**
```python
from microsandbox import PythonSandbox

# Using environment variable (recommended)
async with PythonSandbox.create(name="my-sandbox") as sb:
    # API key automatically loaded from MSB_API_KEY
    pass

# Explicit API key
async with PythonSandbox.create(
    name="my-sandbox",
    api_key="your-api-key-here"
) as sb:
    pass
```

**TypeScript SDK:**
```typescript
import { NodeSandbox } from 'microsandbox';

// Using environment variable (recommended)
const sandbox = await NodeSandbox.create({
  name: 'my-sandbox'
  // API key automatically loaded from MSB_API_KEY
});

// Explicit API key
const sandbox = await NodeSandbox.create({
  name: 'my-sandbox',
  apiKey: 'your-api-key-here'
});
```

#### HTTP API Requests

Include the API key in the Authorization header:

```bash
curl -X POST http://127.0.0.1:5555/api/v1/rpc \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your-api-key-here" \
  -d '{
    "jsonrpc": "2.0",
    "method": "sandbox.start",
    "params": {
      "sandbox": "my-env",
      "namespace": "default"
    },
    "id": "1"
  }'
```

---

### Development Mode

For development and testing, you can skip API key requirements:

```bash
msb server start --dev
```

**Development Mode Features:**
- **No authentication** required
- **Faster setup** for local development
- **All operations** permitted without API keys

!!!warning Development Mode Security
Development mode should **never** be used in production environments as it disables all authentication and security measures.
!!!

---

### Key Management

#### Checking Server Status

Verify if the server requires authentication:

```bash
msb server status
```

#### Regenerating Keys

To generate a new API key (previous keys remain valid):

```bash
msb server keygen --expire 30d
```

#### Resetting Server Key

To invalidate all existing API keys and generate a new server key:

```bash
msb server start --reset-key
```

This will:
- **Invalidate all** existing API keys
- **Generate a new** server signing key
- **Require new** API key generation

---

### Security Best Practices

!!!success Recommended Practices

1. **Use expiring keys** - Set reasonable expiration times for enhanced security
2. **Namespace isolation** - Use namespace-specific keys to limit access scope
3. **Environment variables** - Store keys in environment variables, not in code
4. **Regular rotation** - Regenerate keys periodically, especially for long-running services
5. **Secure storage** - Never commit API keys to version control or logs
6. **Monitor usage** - Track API key usage and revoke unused or compromised keys
7. **Principle of least privilege** - Generate keys with minimal required permissions
8. **Secure transmission** - Always use HTTPS in production environments
!!!

!!!danger Production Security

- **Never use development mode** (`--dev`) in production environments
- **Always require authentication** for production deployments
- **Use TLS/SSL** for all API communications in production
- **Implement network security** (firewalls, VPNs) for additional protection
- **Regular security audits** of API key usage and access patterns

!!!
