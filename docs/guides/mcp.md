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

### MCP Capabilities

microsandbox exposes the following MCP capabilities:

#### Tools Available
- **`sandbox_start`** - Create and start new sandboxes
- **`sandbox_stop`** - Stop running sandboxes
- **`sandbox_run_code`** - Execute code in sandboxes (Python, Node.js)
- **`sandbox_run_command`** - Run shell commands in sandboxes
- **`sandbox_get_metrics`** - Monitor sandbox resource usage

#### Connection Details

- **Endpoint:** `http://127.0.0.1:5555/mcp`
- **Protocol:** Streamable HTTP
- **Authentication:** Bearer token (if not in dev mode)

!!!info Transport Support
microsandbox server only supports the **Streamable HTTP** transport protocol. The deprecated HTTP+SSE transport is not supported. Prefer **Streamable HTTP** when connecting with MCP clients.
!!!

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
"Start a Node.js sandbox, create a simple Express server, and show me the code"
```

**Multi-step Processing:**
```
"Create a sandbox, download some data, process it, and create a visualization"
```

---

### Next Steps

Now that you understand MCP integration:

[!ref API Reference](/references/api)
