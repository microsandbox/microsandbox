# Microsandbox TypeScript SDK Examples

This directory contains examples demonstrating how to use the Microsandbox TypeScript SDK.

## Prerequisites

1. Install dependencies:

   ```
   npm install
   ```

2. Start the Microsandbox server:

   ```
   microsandbox-server
   ```

3. Run the examples as described below

## Running Examples

The examples can be run using the following npm scripts:

```bash
# Run Node.js sandbox example
npm run example:node

# Run Python sandbox example
npm run example:python

# Run shell command execution example
npm run example:command

# Run metrics monitoring example
npm run example:metrics
```

## Examples

### Node.js Example (`node.ts`)

Demonstrates how to use `NodeSandbox` to execute JavaScript code, including:

- Basic JavaScript execution
- Error handling
- Node.js module usage
- Execution chaining with variable state

### Python Example (`python.ts`)

Demonstrates the Python sandbox features, including:

- Different sandbox creation and management patterns
- Resource configuration (memory, CPU)
- Error handling with Python exceptions
- Execution chaining with variable state

### Command Execution Example (`command.ts`)

Shows how to execute shell commands within a sandbox, including:

- Basic command execution
- Error handling
- Command timeouts
- Advanced usage (file I/O, complex pipelines)
- Explicit lifecycle management

### Metrics Example (`metrics.ts`)

Demonstrates how to retrieve and monitor sandbox metrics, including:

- Individual metrics retrieval (CPU, memory, disk)
- All metrics at once
- Continuous monitoring
- CPU load generation and measurement
- Error handling with metrics
