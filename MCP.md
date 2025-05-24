# <sub><img height="18" src="https://octicons-col.vercel.app/plug/A770EF">&nbsp;&nbsp;MODEL CONTEXT PROTOCOL</sub>

microsandbox server is also a **Model Context Protocol (MCP) server**, enabling seamless integration with AI tools and agents that support MCP.

#### What is MCP?

The Model Context Protocol (MCP) is an open standard that allows AI models to securely connect to external data sources and tools. It provides a standardized way for AI assistants to access and interact with various services through a unified interface.

With MCP, your AI can:

- Execute code in secure sandboxes
- Access real-time data and services
- Perform complex operations safely
- Maintain context across interactions

##

#### Connection Details

microsandbox server supports MCP connections via **Streamable HTTP transport only**.

- **Transport**: HTTP
- **URL**: `http://localhost:5555/mcp`
- **Method**: Streamable HTTP

##### Server Configuration

When running your microsandbox server, it automatically exposes MCP endpoints at:

```
http://localhost:5555/mcp
```

> [!NOTE]
> The MCP endpoint uses the same host and port as your main microsandbox server.

##

#### Getting Started

1. **Start your microsandbox server**:

   ```sh
   msb server start --dev
   ```

2. **Configure your MCP client** with the connection details above

3. **Begin using sandbox tools** through your AI assistant

   Try these example prompts:

   ```
   "Create a Python sandbox and run a simple hello world program"
   ```

   ```
   "Start a Node.js sandbox, install express, and create a basic web server"
   ```

   ```
   "Execute this shell command in a sandbox: curl -s https://api.github.com/users/octocat"
   ```

##

#### Available Tools

The microsandbox MCP server provides these tools:

- `sandbox_start` - Create and start new sandboxes
- `sandbox_stop` - Stop running sandboxes
- `sandbox_run_code` - Execute code in sandboxes
- `sandbox_run_command` - Run shell commands
- `sandbox_get_metrics` - Monitor sandbox status
