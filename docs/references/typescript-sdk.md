---
order: 90
icon: file-code
tags: [references]
---

# TypeScript SDK Reference

Complete reference documentation for the microsandbox TypeScript SDK.

---

### Installation

```bash
npm install microsandbox
```

---

### Quick Start

```typescript
import { PythonSandbox } from "microsandbox";

async function main() {
  const sb = await PythonSandbox.create({ name: "my-sandbox" });

  try {
    const exec = await sb.run("print('Hello, World!')");
    console.log(await exec.output());
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

---

### PythonSandbox

The main class for creating and managing Python execution environments.

==- Constructor
```typescript
constructor(options?: SandboxOptions)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options.serverUrl` | `string` | URL of the microsandbox server (defaults to MSB_SERVER_URL env var or "http://127.0.0.1:5555") |
| `options.namespace` | `string` | Namespace for the sandbox (default: "default") |
| `options.name` | `string` | Sandbox identifier (auto-generated if undefined) |
| `options.apiKey` | `string` | Authentication key (or set `MSB_API_KEY` env var) |
===

#### Class Methods

==- `create()`
Creates and automatically starts a new sandbox instance.

```typescript
static async create(options?: SandboxOptions): Promise<PythonSandbox>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options.serverUrl` | `string` | URL of the microsandbox server |
| `options.namespace` | `string` | Namespace for the sandbox |
| `options.name` | `string` | Name for the sandbox |
| `options.apiKey` | `string` | API key for authentication |
| `options.image` | `string` | Docker image to use |
| `options.memory` | `number` | Memory limit in MB |
| `options.cpus` | `number` | CPU cores |
| `options.timeout` | `number` | Startup timeout in seconds |

**Returns:** A started `PythonSandbox` instance

```typescript
const sb = await PythonSandbox.create({ name: "my-sandbox" });
try {
  const exec = await sb.run("print('Hello!')");
  console.log(await exec.output());
} finally {
  await sb.stop();
}
```
===

#### Instance Methods

==- `start()`
Starts the sandbox with optional resource constraints.

```typescript
async start(
  image?: string,
  memory?: number,
  cpus?: number,
  timeout?: number
): Promise<void>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `image` | `string` | Docker image to use (defaults to language-specific image) |
| `memory` | `number` | Memory limit in MB (default: 512) |
| `cpus` | `number` | CPU cores (will be rounded to nearest integer, default: 1.0) |
| `timeout` | `number` | Startup timeout in seconds (default: 180.0) |

```typescript
const sandbox = new PythonSandbox({ name: "resource-limited" });
await sandbox.start("microsandbox/python", 1024, 2.0);
```
===

==- `stop()`
Stops and cleans up the sandbox.

```typescript
async stop(): Promise<void>
```
===

==- `run()`
Executes Python code in the sandbox environment.

```typescript
async run(code: string, options?: { timeout?: number }): Promise<Execution>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `string` | Python code to execute |
| `options.timeout` | `number` | Execution timeout in seconds (optional) |

**Returns:** `Execution` object with results

+++ Simple Execution
```typescript
const exec = await sb.run("print('Hello, World!')");
console.log(await exec.output());
```
+++ Multi-line Code
```typescript
const code = `
import math
result = math.sqrt(16)
print(f"Square root of 16 is: {result}")
`;
const exec = await sb.run(code);
```
+++
===

#### Properties

==- `command`
Access to shell command execution interface.

```typescript
get command(): Command
```

```typescript
const result = await sb.command.run("ls", ["-la", "/"]);
console.log(await result.output());
```
===

==- `metrics`
Access to resource monitoring interface.

```typescript
get metrics(): Metrics
```

```typescript
const cpu = await sb.metrics.cpu();
const memory = await sb.metrics.memory();
console.log(`CPU: ${cpu}%, Memory: ${memory} MiB`);
```
===

==- `isStarted`
Returns sandbox running status.

```typescript
get isStarted(): boolean
```
===

==- `name`
Returns the sandbox identifier.

```typescript
get name(): string
```
===

==- `serverUrl`
Returns the server URL.

```typescript
get serverUrl(): string
```
===

==- `namespace`
Returns the namespace.

```typescript
get namespace(): string
```
===

==- `apiKey`
Returns the API key.

```typescript
get apiKey(): string | undefined
```
===

---

### NodeSandbox

JavaScript/Node.js execution environment with identical API to PythonSandbox.

==- Constructor
```typescript
constructor(options?: SandboxOptions)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options.serverUrl` | `string` | URL of the microsandbox server |
| `options.namespace` | `string` | Namespace for the sandbox |
| `options.name` | `string` | Sandbox identifier (auto-generated if undefined) |
| `options.apiKey` | `string` | Authentication key (or set `MSB_API_KEY` env var) |
===

All methods are identical to `PythonSandbox` but execute JavaScript code.

```typescript
import { NodeSandbox } from "microsandbox";

const sb = await NodeSandbox.create({ name: "js-sandbox" });
try {
  const exec = await sb.run("console.log('Hello from Node.js!');");
  console.log(await exec.output());
} finally {
  await sb.stop();
}
```

---

### Execution

Represents the result of code execution from `sandbox.run()`.

#### Methods

==- `output()`
Returns standard output from execution.

```typescript
async output(): Promise<string>
```
===

==- `error()`
Returns standard error from execution.

```typescript
async error(): Promise<string>
```
===

==- `hasError()`
Checks if execution produced errors.

```typescript
hasError(): boolean
```
===

#### Properties

==- `status`
Execution status.

```typescript
get status(): string
```
===

==- `language`
Language used for execution.

```typescript
get language(): string
```
===

!!!warning Example: Handling Execution Results
```typescript
const exec = await sb.run("print('Hello'); import sys; sys.exit(1)");
console.log(`Status: ${exec.status}`);
console.log(`Language: ${exec.language}`);
console.log(`Output: ${await exec.output()}`);
console.log(`Has error: ${exec.hasError()}`);
```
!!!

---

### CommandExecution

Represents the result of command execution from `sandbox.command.run()`.

#### Methods

==- `output()`
Returns standard output from command execution.

```typescript
async output(): Promise<string>
```
===

==- `error()`
Returns standard error from command execution.

```typescript
async error(): Promise<string>
```
===

#### Properties

==- `exitCode`
Command exit code.

```typescript
get exitCode(): number
```
===

==- `success`
True if command was successful (exit code 0).

```typescript
get success(): boolean
```
===

==- `command`
The command that was executed.

```typescript
get command(): string
```
===

==- `args`
Arguments used for the command.

```typescript
get args(): string[]
```
===

---

### Command

Interface for executing shell commands within sandboxes.

==- `run()`
Executes shell commands with arguments.

```typescript
async run(
  command: string,
  args?: string[],
  timeout?: number
): Promise<CommandExecution>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `command` | `string` | Command to execute |
| `args` | `string[]` | Command arguments (optional) |
| `timeout` | `number` | Execution timeout in seconds (optional) |

**Returns:** `CommandExecution` object with results

+++ Simple Command
```typescript
const result = await sb.command.run("ls");
```
+++ With Arguments
```typescript
const result = await sb.command.run("ls", ["-la", "/tmp"]);
```
+++ With Timeout
```typescript
const result = await sb.command.run("sleep", ["10"], 5);
```
+++ Error Handling
```typescript
const result = await sb.command.run("ls", ["/nonexistent"]);
if (result.success) {
  console.log("Output:", await result.output());
} else {
  console.log("Error:", await result.error());
  console.log("Exit code:", result.exitCode);
}
```
+++
===

---

### Metrics

Interface for monitoring sandbox resource usage and performance.

#### Methods

==- `cpu()`
Current CPU usage percentage.

```typescript
async cpu(): Promise<number | undefined>
```
===

==- `memory()`
Current memory usage in MiB.

```typescript
async memory(): Promise<number | undefined>
```
===

==- `disk()`
Current disk usage in bytes.

```typescript
async disk(): Promise<number | undefined>
```
===

==- `isRunning()`
Sandbox running status.

```typescript
async isRunning(): Promise<boolean>
```
===

==- `all()`
All metrics as a dictionary.

```typescript
async all(): Promise<any>
```

**Returns:** Object with keys: `name`, `namespace`, `running`, `cpu_usage`, `memory_usage`, `disk_usage`

```typescript
// Individual metrics
const cpu = await sb.metrics.cpu();
const memory = await sb.metrics.memory();
const disk = await sb.metrics.disk();
const running = await sb.metrics.isRunning();

console.log(`CPU: ${cpu !== undefined ? cpu + '%' : 'Not available'}`);
console.log(`Memory: ${memory || 'Not available'} MiB`);
console.log(`Disk: ${disk || 'Not available'} bytes`);
console.log(`Running: ${running}`);

// All metrics at once
const metrics = await sb.metrics.all();
console.log(`All metrics:`, metrics);
```
===

---

### Usage Patterns

#### Automatic Cleanup (Recommended)

Manual resource management with proper cleanup.

```typescript
const sb = await PythonSandbox.create({ name: "my-sandbox" });

try {
  const exec = await sb.run("print('Hello, World!')");
  console.log(await exec.output());
} finally {
  await sb.stop(); // Always cleanup
}
```

#### Manual Lifecycle Management

Manual control over sandbox lifecycle.

```typescript
const sandbox = new PythonSandbox({ name: "my-sandbox" });

try {
  await sandbox.start(undefined, 1024, 2.0);
  const exec = await sandbox.run("print('Hello, World!')");
  console.log(await exec.output());
} finally {
  await sandbox.stop();
}
```

#### State Persistence

Variables and imports persist between executions.

```typescript
const sb = await PythonSandbox.create({ name: "stateful" });

try {
  await sb.run("x = 42");
  await sb.run("y = x * 2");
  const exec = await sb.run("print(f'Result: {y}')");
  console.log(await exec.output()); // Output: Result: 84
} finally {
  await sb.stop();
}
```

#### Error Handling

Comprehensive error handling patterns.

```typescript
const sb = await PythonSandbox.create({ name: "error-handling" });

try {
  const exec = await sb.run("1/0"); // Division by zero
  if (exec.hasError()) {
    console.log("Error occurred:", await exec.error());
  }
} catch (error) {
  console.log("Runtime error:", error);
} finally {
  await sb.stop();
}
```

#### File Operations

Creating and manipulating files within sandboxes.

```typescript
const sb = await PythonSandbox.create({ name: "files" });

try {
  // Create a file using Python
  await sb.run(`
with open('/tmp/test.txt', 'w') as f:
    f.write('Hello from sandbox!')
  `);

  // Read the file using shell command
  const result = await sb.command.run("cat", ["/tmp/test.txt"]);
  console.log("File content:", await result.output());
} finally {
  await sb.stop();
}
```

---

### Configuration

#### Environment Variables

| Variable | Description |
|----------|-------------|
| `MSB_API_KEY` | API key for authentication |
| `MSB_SERVER_URL` | Default server URL (overrides default) |

#### Server Configuration

Default server URL: `http://127.0.0.1:5555`

---

### Error Handling

!!!danger Common Error Types
The SDK throws `Error` for various conditions:

- **Connection failures** — Server unreachable or network issues
- **Authentication errors** — Invalid or missing API key
- **Execution timeouts** — Code/commands exceed time limits
- **Invalid operations** — Attempting operations on stopped sandboxes
!!!

#### Best Practices

```typescript
try {
  const sb = await PythonSandbox.create({ name: "error-test" });

  try {
    const exec = await sb.run("potentially_failing_code()");

    if (exec.hasError()) {
      console.log("Execution error:", await exec.error());
    } else {
      console.log("Success:", await exec.output());
    }
  } finally {
    await sb.stop();
  }
} catch (error) {
  console.log(`SDK error: ${error}`);
}
```

---

### Best Practices

!!!success Recommended Practices

1. **Always use try-finally** — Ensures proper cleanup
2. **Set appropriate timeouts** — Prevent hanging operations
3. **Handle errors gracefully** — Use try-catch blocks
4. **Monitor resources** — Use metrics interface
5. **Use meaningful names** — Easier debugging and management
6. **Install packages once** — Reuse sandbox for multiple executions
7. **Always call stop()** — Prevent resource leaks

!!!

!!!warning Common Pitfalls

- Forgetting to call `stop()` in finally blocks
- Not handling execution errors properly
- Setting timeouts too low for complex operations
- Creating too many concurrent sandboxes without resource limits

!!!
