---
order: 60
icon: server
tags: [references]
---

# API Reference

Complete reference documentation for the microsandbox server HTTP API.

---

### Base URL

The microsandbox server runs on `http://127.0.0.1:5555` by default. All API endpoints are prefixed with `/api/v1`.

---

### Authentication

The API uses Bearer token authentication. Include your API key in the Authorization header:

```
Authorization: Bearer YOUR_API_KEY
```

Generate an API key using the CLI:

```bash
msb server keygen
```

---

### REST Endpoints

==- Health Check
Check if the server is running and healthy.

**Endpoint:** `GET /api/v1/health`

**Response:**
```json
{
  "message": "Service is healthy"
}
```

**Status Codes:**
- `200 OK` - Server is healthy
===

---

### JSON-RPC API

The main API uses JSON-RPC 2.0 over HTTP POST. All requests should be sent to `/api/v1/rpc`.

**Content-Type:** `application/json`

#### Request Format

```json
{
  "jsonrpc": "2.0",
  "method": "method_name",
  "params": { ... },
  "id": "unique_request_id"
}
```

#### Response Format

**Success:**
```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": "unique_request_id"
}
```

**Error:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Error description",
    "data": null
  },
  "id": "unique_request_id"
}
```

---

### Sandbox Management

==- `sandbox.start`
Start a new sandbox with specified configuration.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sandbox` | `string` | Yes | Name of the sandbox to start |
| `namespace` | `string` | Yes | Namespace for the sandbox |
| `config` | `object` | No | Sandbox configuration (see below) |

**Configuration Object:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | `string` | No | Docker image to use |
| `memory` | `integer` | No | Memory limit in MiB (default: 512) |
| `cpus` | `integer` | No | Number of CPUs (default: 1) |
| `volumes` | `array[string]` | No | Volume mounts (format: `host:container`) |
| `ports` | `array[string]` | No | Port mappings (format: `host:container`) |
| `envs` | `array[string]` | No | Environment variables (format: `KEY=VALUE`) |
| `depends_on` | `array[string]` | No | Dependencies on other sandboxes |
| `workdir` | `string` | No | Working directory |
| `shell` | `string` | No | Shell to use |
| `scripts` | `object` | No | Named scripts (key-value pairs) |
| `exec` | `string` | No | Command to execute on start |

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sandbox.start",
  "params": {
    "sandbox": "my-python-env",
    "namespace": "default",
    "config": {
      "image": "microsandbox/python",
      "memory": 1024,
      "cpus": 2,
      "envs": ["DEBUG=true"],
      "workdir": "/workspace"
    }
  },
  "id": "1"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "Sandbox my-python-env started successfully",
  "id": "1"
}
```

**Error Codes:**
- `-32602` - Invalid parameters
- `-32603` - Sandbox start failed
===

==- `sandbox.stop`
Stop a running sandbox and clean up its resources.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sandbox` | `string` | Yes | Name of the sandbox to stop |
| `namespace` | `string` | Yes | Namespace of the sandbox |

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sandbox.stop",
  "params": {
    "sandbox": "my-python-env",
    "namespace": "default"
  },
  "id": "2"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": "Sandbox my-python-env stopped successfully",
  "id": "2"
}
```

**Error Codes:**
- `-32602` - Invalid parameters
- `-32603` - Sandbox stop failed
===

==- `sandbox.metrics.get`
Get metrics and status for sandboxes including CPU usage, memory consumption, and running state.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `namespace` | `string` | Yes | Namespace to query (use `"*"` for all namespaces) |
| `sandbox` | `string` | No | Specific sandbox name (omit for all sandboxes in namespace) |

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sandbox.metrics.get",
  "params": {
    "namespace": "default",
    "sandbox": "my-python-env"
  },
  "id": "3"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "sandboxes": [
      {
        "namespace": "default",
        "name": "my-python-env",
        "running": true,
        "cpu_usage": 15.5,
        "memory_usage": 256,
        "disk_usage": 1048576
      }
    ]
  },
  "id": "3"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `namespace` | `string` | Namespace the sandbox belongs to |
| `name` | `string` | Name of the sandbox |
| `running` | `boolean` | Whether the sandbox is currently running |
| `cpu_usage` | `number` | CPU usage percentage (null if not available) |
| `memory_usage` | `number` | Memory usage in MiB (null if not available) |
| `disk_usage` | `number` | Disk usage in bytes (null if not available) |

**Error Codes:**
- `-32602` - Invalid parameters
- `-32603` - Failed to get metrics
===

---

### Code Execution

==- `sandbox.repl.run`
Execute code in a running sandbox. This method is forwarded to the sandbox's portal service.

**Prerequisites:** The target sandbox must be started first using `sandbox.start`.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sandbox` | `string` | Yes | Name of the sandbox (must be already started) |
| `namespace` | `string` | Yes | Namespace of the sandbox |
| `language` | `string` | Yes | Programming language (`"python"`, `"nodejs"`) |
| `code` | `string` | Yes | Code to execute |
| `timeout` | `integer` | No | Execution timeout in seconds |

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sandbox.repl.run",
  "params": {
    "sandbox": "my-python-env",
    "namespace": "default",
    "language": "python",
    "code": "print('Hello, World!')"
  },
  "id": "4"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "completed",
    "language": "python",
    "output": "Hello, World!\n",
    "error": "",
    "has_error": false
  },
  "id": "4"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `status` | `string` | Execution status |
| `language` | `string` | Language used for execution |
| `output` | `string` | Standard output from execution |
| `error` | `string` | Standard error from execution |
| `has_error` | `boolean` | Whether execution produced errors |

**Error Codes:**
- `-32602` - Invalid parameters
- `-32603` - Execution failed
===

==- `sandbox.command.run`
Execute a shell command in a running sandbox. This method is forwarded to the sandbox's portal service.

**Prerequisites:** The target sandbox must be started first using `sandbox.start`.

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `sandbox` | `string` | Yes | Name of the sandbox (must be already started) |
| `namespace` | `string` | Yes | Namespace of the sandbox |
| `command` | `string` | Yes | Command to execute |
| `args` | `array[string]` | No | Command arguments |
| `timeout` | `integer` | No | Execution timeout in seconds |

**Example Request:**
```json
{
  "jsonrpc": "2.0",
  "method": "sandbox.command.run",
  "params": {
    "sandbox": "my-python-env",
    "namespace": "default",
    "command": "ls",
    "args": ["-la", "/workspace"]
  },
  "id": "5"
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "result": {
    "command": "ls",
    "args": ["-la", "/workspace"],
    "exit_code": 0,
    "success": true,
    "output": "total 4\ndrwxr-xr-x 2 root root 4096 Jan 1 12:00 .\n",
    "error": ""
  },
  "id": "5"
}
```

**Response Fields:**

| Field | Type | Description |
|-------|------|-------------|
| `command` | `string` | The command that was executed |
| `args` | `array[string]` | Arguments used for the command |
| `exit_code` | `integer` | Command exit code |
| `success` | `boolean` | True if command was successful (exit code 0) |
| `output` | `string` | Standard output from command |
| `error` | `string` | Standard error from command |

**Error Codes:**
- `-32602` - Invalid parameters
- `-32603` - Command execution failed
===

---

### MCP (Model Context Protocol) Support

The microsandbox server also implements the Model Context Protocol, making it compatible with AI tools like Claude.

==- MCP Methods
The server supports the following MCP methods:

| Method | Description |
|--------|-------------|
| `initialize` | Initialize MCP connection and get server capabilities |
| `tools/list` | List available tools (sandbox operations) |
| `tools/call` | Execute a tool (maps to sandbox operations) |
| `prompts/list` | List available prompt templates |
| `prompts/get` | Get a specific prompt template |

**MCP Tools Available:**
- `sandbox_start` - Start a new sandbox
- `sandbox_stop` - Stop a running sandbox
- `sandbox_run_code` - Execute code in a sandbox
- `sandbox_run_command` - Execute commands in a sandbox
- `sandbox_get_metrics` - Get sandbox metrics

**MCP Prompts Available:**
- `create_python_sandbox` - Template for creating Python sandboxes
- `create_node_sandbox` - Template for creating Node.js sandboxes
===

---

### Error Handling

#### Standard JSON-RPC Error Codes

| Code | Message | Description |
|------|---------|-------------|
| `-32700` | Parse error | Invalid JSON was received |
| `-32600` | Invalid Request | The JSON sent is not a valid Request object |
| `-32601` | Method not found | The method does not exist / is not available |
| `-32602` | Invalid params | Invalid method parameter(s) |
| `-32603` | Internal error | Internal JSON-RPC error |

#### Custom Error Codes

| Code | Description |
|------|-------------|
| `-32000` | Server error |
| `-32001` | Validation error |
| `-32002` | Authentication error |
| `-32003` | Resource not found |

#### Common Error Scenarios

**Sandbox Not Found:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32603,
    "message": "Sandbox 'nonexistent' not found in configuration"
  },
  "id": "1"
}
```

**Invalid Namespace:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Namespace cannot be empty"
  },
  "id": "1"
}
```

**Authentication Required:**
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32002,
    "message": "Authentication required"
  },
  "id": "1"
}
```

---

### Rate Limiting

The API does not currently implement rate limiting, but it's recommended to:

- Limit concurrent sandbox starts to avoid resource exhaustion
- Use reasonable timeouts for long-running operations
- Monitor resource usage through the metrics endpoint

---

### Best Practices

!!!success Recommended Practices

1. **Always stop sandboxes** when done to prevent resource leaks
2. **Use meaningful names** for sandboxes and namespaces
3. **Set appropriate timeouts** for operations
4. **Monitor metrics** regularly to track resource usage
5. **Handle errors gracefully** with proper retry logic
6. **Use namespaces** to organize sandboxes by project or team

!!!

!!!warning Common Pitfalls

- Starting sandboxes without stopping them (resource leaks)
- Using invalid characters in sandbox/namespace names
- Not handling timeout errors properly
- Attempting operations on non-existent sandboxes
- Forgetting to include authentication headers

!!!

---

### Examples

#### Complete Workflow Example

**1. Start the server:**
```bash
msb server start --dev
```

**2. Generate API key:**
```bash
msb server keygen
```

**3. Use the API:**
```javascript
const apiKey = "your-api-key";
const baseUrl = "http://127.0.0.1:5555/api/v1/rpc";
```

**4. Start a sandbox:**
```javascript
const startResponse = await fetch(baseUrl, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "Authorization": `Bearer ${apiKey}`
  },
  body: JSON.stringify({
    jsonrpc: "2.0",
    method: "sandbox.start",
    params: {
      sandbox: "my-env",
      namespace: "default",
      config: {
        image: "microsandbox/python",
        memory: 512
      }
    },
    id: "1"
  })
});
```

**5. Execute code:**
```javascript
const runResponse = await fetch(baseUrl, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "Authorization": `Bearer ${apiKey}`
  },
  body: JSON.stringify({
    jsonrpc: "2.0",
    method: "sandbox.repl.run",
    params: {
      sandbox: "my-env",
      namespace: "default",
      language: "python",
      code: "print('Hello from API!')"
    },
    id: "2"
  })
});
```

**6. Stop the sandbox:**
```javascript
const stopResponse = await fetch(baseUrl, {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    "Authorization": `Bearer ${apiKey}`
  },
  body: JSON.stringify({
    jsonrpc: "2.0",
    method: "sandbox.stop",
    params: {
      sandbox: "my-env",
      namespace: "default"
    },
    "id": "3"
  })
});
```
