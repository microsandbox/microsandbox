---
order: 70
icon: plug
tags: [guide]
---

# Model Context Protocol (MCP)

Learn how to integrate microsandbox with AI tools using the Model Context Protocol for seamless code execution and sandbox management.

---

### Overview

The [Model Context Protocol (MCP)](https://modelcontextprotocol.io) is an open standard that enables AI applications to securely connect to external data sources and tools. microsandbox implements MCP as a built-in server, making it compatible with AI tools like Claude Desktop, Cursor, and other MCP-enabled applications.

---

### Connection Details

- **Endpoint:** `http://127.0.0.1:5555/mcp`
- **Protocol:** Streamable HTTP
- **Authentication:** Bearer token (if not in dev mode)

!!!info Transport Support
microsandbox server only supports the **Streamable HTTP** transport protocol.
!!!

---

### Tools

microsandbox exposes tools through the MCP interface for complete sandbox lifecycle management.

---

### Sandbox Management Tools

==- `sandbox_start`
Start a new sandbox with specified configuration. This creates an isolated environment for code execution.

| Parameter   | Type     | Required | Description                   |
| ----------- | -------- | -------- | ----------------------------- |
| `sandbox`   | `string` | ✓        | Name of the sandbox to start  |
| `namespace` | `string` | ✓        | Namespace for the sandbox     |
| `config`    | `object` |          | Sandbox configuration options |

#### Configuration Options

| Property  | Type            | Description                                                            |
| --------- | --------------- | ---------------------------------------------------------------------- |
| `image`   | `string`        | Docker image to use (e.g., `microsandbox/python`, `microsandbox/node`) |
| `memory`  | `integer`       | Memory limit in MiB                                                    |
| `cpus`    | `integer`       | Number of CPUs                                                         |
| `volumes` | `array[string]` | Volume mounts                                                          |
| `ports`   | `array[string]` | Port mappings                                                          |
| `envs`    | `array[string]` | Environment variables                                                  |

+++ Basic Python Sandbox

```json
{
  "sandbox": "my-python-env",
  "namespace": "default"
}
```

+++ Custom Configuration

```json
{
  "sandbox": "data-analysis",
  "namespace": "research",
  "config": {
    "image": "microsandbox/python",
    "memory": 1024,
    "cpus": 2,
    "envs": ["PYTHONPATH=/workspace"]
  }
}
```

+++ Node.js Environment

```json
{
  "sandbox": "node-env",
  "namespace": "development",
  "config": {
    "image": "microsandbox/node",
    "memory": 512
  }
}
```

+++

!!!warning Important
Always stop the sandbox when done to prevent it from running indefinitely and consuming resources.
!!!
===

==- `sandbox_stop`
Stop a running sandbox and clean up its resources.

| Parameter   | Type     | Required | Description                 |
| ----------- | -------- | -------- | --------------------------- |
| `sandbox`   | `string` | ✓        | Name of the sandbox to stop |
| `namespace` | `string` | ✓        | Namespace of the sandbox    |

```json
{
  "sandbox": "my-python-env",
  "namespace": "default"
}
```

!!!warning Critical
Always call this when you're finished with a sandbox to prevent resource leaks and indefinite running. Failing to stop sandboxes will cause them to consume system resources unnecessarily.
!!!
===

---

### Code Execution Tools

==- `sandbox_run_code`
Execute code in a running sandbox environment.

| Parameter   | Type     | Required | Description                                   |
| ----------- | -------- | -------- | --------------------------------------------- |
| `sandbox`   | `string` | ✓        | Name of the sandbox (must be already started) |
| `namespace` | `string` | ✓        | Namespace of the sandbox                      |
| `code`      | `string` | ✓        | Code to execute                               |
| `language`  | `string` | ✓        | Programming language (`python`, `nodejs`)     |

+++ Python Execution

```json
{
  "sandbox": "my-python-env",
  "namespace": "default",
  "code": "import math\nresult = math.sqrt(16)\nprint(f'Square root: {result}')",
  "language": "python"
}
```

+++ JavaScript Execution

```json
{
  "sandbox": "node-env",
  "namespace": "development",
  "code": "const fs = require('fs');\nconst data = { message: 'Hello from Node.js!' };\nconsole.log(JSON.stringify(data, null, 2));",
  "language": "nodejs"
}
```

+++

!!! Prerequisites
The target sandbox must be started first using `sandbox_start` - this will fail if the sandbox is not running. Code execution is synchronous and may take time depending on complexity.
!!!
===

==- `sandbox_run_command`
Execute shell commands in a running sandbox.

| Parameter   | Type            | Required | Description                                   |
| ----------- | --------------- | -------- | --------------------------------------------- |
| `sandbox`   | `string`        | ✓        | Name of the sandbox (must be already started) |
| `namespace` | `string`        | ✓        | Namespace of the sandbox                      |
| `command`   | `string`        | ✓        | Command to execute                            |
| `args`      | `array[string]` |          | Command arguments                             |

+++ Simple Command

```json
{
  "sandbox": "my-python-env",
  "namespace": "default",
  "command": "ls"
}
```

+++ Command with Arguments

```json
{
  "sandbox": "my-python-env",
  "namespace": "default",
  "command": "ls",
  "args": ["-la", "/workspace"]
}
```

+++ Package Installation

```json
{
  "sandbox": "data-analysis",
  "namespace": "research",
  "command": "pip",
  "args": ["install", "pandas", "numpy", "matplotlib"]
}
```

+++

!!!info Prerequisites
The target sandbox must be started first using `sandbox_start` - this will fail if the sandbox is not running. Command execution is synchronous and may take time depending on complexity.
!!!
===

---

### Monitoring Tools

==- `sandbox_get_metrics`
Get metrics and status for sandboxes including CPU usage, memory consumption, and running state.

| Parameter   | Type     | Required | Description                                       |
| ----------- | -------- | -------- | ------------------------------------------------- |
| `namespace` | `string` | ✓        | Namespace to query (use `*` for all namespaces)   |
| `sandbox`   | `string` |          | Optional specific sandbox name to get metrics for |

+++ Single Sandbox

```json
{
  "sandbox": "my-python-env",
  "namespace": "default"
}
```

+++ All Sandboxes in Namespace

```json
{
  "namespace": "development"
}
```

+++ All Sandboxes

```json
{
  "namespace": "*"
}
```

+++

**Returns:** JSON object with metrics including:

- `name` - Sandbox name
- `namespace` - Sandbox namespace
- `running` - Boolean running status
- `cpu_usage` - CPU usage percentage
- `memory_usage` - Memory usage in MiB
- `disk_usage` - Disk usage in bytes

!!! Usage
This tool can check the status of any sandbox regardless of whether it's running or not, making it useful for monitoring and cleanup operations.
!!!
===

---

### Setting Up microsandbox with an Agent

Let's use [Agno](https://docs.agno.com) to build an AI agent that can execute code in microsandbox.

#### Prerequisites

1. **Install Agno and dependencies**:

```bash
pip install agno openai
```

2. **Start microsandbox server**:

```bash
msb server start --dev
```

#### Integration Example

```python
from agno.agent import Agent
from agno.models.openai import OpenAIChat
from agno.tools.mcp import MCPTools

async def main():
    # Connect to microsandbox MCP server
    server_url = "http://127.0.0.1:5555/mcp"

    async with MCPTools(url=server_url, transport="streamable-http") as mcp_tools:
        # Create agent with microsandbox tools
        agent = Agent(
            model=OpenAIChat(id="gpt-4o"),
            tools=[mcp_tools],
            description="AI assistant with secure code execution capabilities"
        )

        # Use the agent with microsandbox integration
        await agent.aprint_response(
            "Create a Python sandbox and calculate the first 10 fibonacci numbers",
            stream=True
        )

# Run the example
import asyncio
asyncio.run(main())
```

#### Other MCP-Compatible Tools

microsandbox works with any MCP-compatible application:

- **Cursor** - AI-powered code editor
- **Custom MCP clients** - Build your own integrations

---

### Examples

#### Complete Workflow

1. **Start the server:**

```bash
msb server start --dev
```

2. **Configure Claude Desktop** with the MCP server

3. **Test the integration:**

```
Ask Claude: "Can you start a Python sandbox and run a simple calculation?"
```

4. **Claude will:**
   - Call `sandbox_start` to create a new Python environment
   - Call `sandbox_run_code` to execute your calculation
   - Return the results in a natural language response

#### Advanced Usage

**Data Analysis Workflow:**

```
"Create a Python sandbox, install pandas, and analyze this CSV data: [paste data]"
```

**Web Development:**

```
"Start a Node.js sandbox and create a simple HTML generator script"
```

**Multi-step Processing:**

```
"Create a sandbox, download some data, process it, and create a visualization"
```

---

### Next Steps

[!ref API Reference](/references/api)
