---
order: 100
icon: file-code
tags: [references]
---

# Python SDK Reference

Complete reference documentation for the microsandbox Python SDK.

---

### Installation

```bash
pip install microsandbox
```

---

### Quick Start

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="my-sandbox") as sb:
        exec = await sb.run("print('Hello, World!')")
        print(await exec.output())

asyncio.run(main())
```

---

### PythonSandbox

The main class for creating and managing Python execution environments.

==- Constructor
```python
PythonSandbox(
    server_url: str = None,
    namespace: str = "default",
    name: str = None,
    api_key: str = None
)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `server_url` | `str` | URL of the microsandbox server (defaults to MSB_SERVER_URL env var or "http://127.0.0.1:5555") |
| `namespace` | `str` | Namespace for the sandbox (default: "default") |
| `name` | `str` | Sandbox identifier (auto-generated if None) |
| `api_key` | `str` | Authentication key (or set `MSB_API_KEY` env var) |
===

#### Class Methods

==- `create()`
Creates and automatically starts a new sandbox instance as an async context manager.

```python
@classmethod
@asynccontextmanager
async def create(
    cls,
    server_url: str = None,
    namespace: str = "default",
    name: str = None,
    api_key: str = None
)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `server_url` | `str` | URL of the microsandbox server |
| `namespace` | `str` | Namespace for the sandbox |
| `name` | `str` | Name for the sandbox |
| `api_key` | `str` | API key for authentication |

**Returns:** An async context manager that yields a started `PythonSandbox` instance

```python
async with PythonSandbox.create(name="my-sandbox") as sb:
    # Sandbox automatically started and stopped
    exec = await sb.run("print('Hello!')")
    print(await exec.output())
```
===

#### Instance Methods

==- `start()`
Starts the sandbox with optional resource constraints.

```python
async def start(
    self,
    image: str = None,
    memory: int = 512,
    cpus: float = 1.0,
    timeout: float = 180.0
) -> None
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `image` | `str` | Docker image to use (defaults to language-specific image) |
| `memory` | `int` | Memory limit in MB (default: 512) |
| `cpus` | `float` | CPU cores (will be rounded to nearest integer, default: 1.0) |
| `timeout` | `float` | Startup timeout in seconds (default: 180.0) |

```python
sandbox = PythonSandbox(name="resource-limited")
sandbox._session = aiohttp.ClientSession()
await sandbox.start(memory=1024, cpus=2.0)
```
===

==- `stop()`
Stops and cleans up the sandbox.

```python
async def stop(self) -> None
```
===

==- `run()`
Executes Python code in the sandbox environment.

```python
async def run(self, code: str) -> "Execution"
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `str` | Python code to execute |

**Returns:** `Execution` object with results

+++ Simple Execution
```python
exec = await sb.run("print('Hello, World!')")
print(await exec.output())
```
+++ Multi-line Code
```python
code = """
import math
result = math.sqrt(16)
print(f"Square root of 16 is: {result}")
"""
exec = await sb.run(code)
```
+++
===

#### Properties

==- `command`
Access to shell command execution interface.

```python
@property
def command(self) -> "Command"
```

```python
result = await sb.command.run("ls", ["-la", "/"])
print(await result.output())
```
===

==- `metrics`
Access to resource monitoring interface.

```python
@property
def metrics(self) -> "Metrics"
```

```python
cpu = await sb.metrics.cpu()
memory = await sb.metrics.memory()
print(f"CPU: {cpu}%, Memory: {memory} MiB")
```
===

==- `is_started`
Returns sandbox running status.

```python
@property
def is_started(self) -> bool
```
===

==- `name`
Returns the sandbox identifier.

```python
@property
def name(self) -> str
```
===

---

### NodeSandbox

JavaScript/Node.js execution environment with identical API to PythonSandbox.

==- Constructor
```python
NodeSandbox(
    server_url: str = None,
    namespace: str = "default",
    name: str = None,
    api_key: str = None
)
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `server_url` | `str` | URL of the microsandbox server |
| `namespace` | `str` | Namespace for the sandbox |
| `name` | `str` | Sandbox identifier (auto-generated if None) |
| `api_key` | `str` | Authentication key (or set `MSB_API_KEY` env var) |
===

All methods are identical to `PythonSandbox` but execute JavaScript code.

```python
from microsandbox import NodeSandbox

async with NodeSandbox.create(name="js-sandbox") as sb:
    exec = await sb.run("console.log('Hello from Node.js!');")
    print(await exec.output())
```

---

### Execution

Represents the result of code execution from `sandbox.run()`.

#### Methods

==- `output()`
Returns standard output from execution.

```python
async def output(self) -> str
```
===

==- `error()`
Returns standard error from execution.

```python
async def error(self) -> str
```
===

==- `has_error()`
Checks if execution produced errors.

```python
def has_error(self) -> bool
```
===

#### Properties

==- `status`
Execution status.

```python
@property
def status(self) -> str
```
===

==- `language`
Language used for execution.

```python
@property
def language(self) -> str
```
===

!!!warning Example: Handling Execution Results
```python
exec = await sb.run("print('Hello'); import sys; sys.exit(1)")
print(f"Status: {exec.status}")
print(f"Language: {exec.language}")
print(f"Output: {await exec.output()}")
print(f"Has error: {exec.has_error()}")
```
!!!

---

### CommandExecution

Represents the result of command execution from `sandbox.command.run()`.

#### Methods

==- `output()`
Returns standard output from command execution.

```python
async def output(self) -> str
```
===

==- `error()`
Returns standard error from command execution.

```python
async def error(self) -> str
```
===

#### Properties

==- `exit_code`
Command exit code.

```python
@property
def exit_code(self) -> int
```
===

==- `success`
True if command was successful (exit code 0).

```python
@property
def success(self) -> bool
```
===

==- `command`
The command that was executed.

```python
@property
def command(self) -> str
```
===

==- `args`
Arguments used for the command.

```python
@property
def args(self) -> List[str]
```
===

---

### Command

Interface for executing shell commands within sandboxes.

==- `run()`
Executes shell commands with arguments.

```python
async def run(
    self,
    command: str,
    args: List[str] = None,
    timeout: int = None
) -> "CommandExecution"
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `command` | `str` | Command to execute |
| `args` | `List[str]` | Command arguments (optional) |
| `timeout` | `int` | Execution timeout in seconds (optional) |

**Returns:** `CommandExecution` object with results

+++ Simple Command
```python
result = await sb.command.run("ls")
```
+++ With Arguments
```python
result = await sb.command.run("ls", ["-la", "/tmp"])
```
+++ With Timeout
```python
result = await sb.command.run("sleep", ["10"], timeout=5)
```
+++ Error Handling
```python
result = await sb.command.run("ls", ["/nonexistent"])
if result.success:
    print("Output:", await result.output())
else:
    print("Error:", await result.error())
    print("Exit code:", result.exit_code)
```
+++
===

---

### Metrics

Interface for monitoring sandbox resource usage and performance.

#### Methods

==- `cpu()`
Current CPU usage percentage.

```python
async def cpu(self) -> Optional[float]
```
===

==- `memory()`
Current memory usage in MiB.

```python
async def memory(self) -> Optional[int]
```
===

==- `disk()`
Current disk usage in bytes.

```python
async def disk(self) -> Optional[int]
```
===

==- `is_running()`
Sandbox running status.

```python
async def is_running(self) -> bool
```
===

==- `all()`
All metrics as a dictionary.

```python
async def all(self) -> Dict[str, Any]
```

**Returns:** Dictionary with keys: `name`, `namespace`, `running`, `cpu_usage`, `memory_usage`, `disk_usage`

```python
# Individual metrics
cpu = await sb.metrics.cpu()
memory = await sb.metrics.memory()
disk = await sb.metrics.disk()
running = await sb.metrics.is_running()

print(f"CPU: {cpu}%" if cpu is not None else "CPU: Not available")
print(f"Memory: {memory} MiB" if memory else "Memory: Not available")
print(f"Disk: {disk} bytes" if disk else "Disk: Not available")
print(f"Running: {running}")

# All metrics at once
metrics = await sb.metrics.all()
print(f"All metrics: {metrics}")
```
===

---

### Usage Patterns

#### Context Manager (Recommended)

Automatic resource management with guaranteed cleanup.

```python
async with PythonSandbox.create(name="my-sandbox") as sb:
    exec = await sb.run("print('Hello, World!')")
    print(await exec.output())
# Sandbox automatically stopped and cleaned up
```

#### Manual Lifecycle Management

Manual control over sandbox lifecycle.

```python
import aiohttp

sandbox = PythonSandbox(name="my-sandbox")
sandbox._session = aiohttp.ClientSession()

try:
    await sandbox.start(memory=1024, cpus=2.0)
    exec = await sandbox.run("print('Hello, World!')")
    print(await exec.output())
finally:
    await sandbox.stop()
    await sandbox._session.close()
```

#### State Persistence

Variables and imports persist between executions.

```python
async with PythonSandbox.create(name="stateful") as sb:
    await sb.run("x = 42")
    await sb.run("y = x * 2")
    exec = await sb.run("print(f'Result: {y}')")
    print(await exec.output())  # Output: Result: 84
```

#### Error Handling

Comprehensive error handling patterns.

```python
async with PythonSandbox.create(name="error-handling") as sb:
    try:
        exec = await sb.run("1/0")  # Division by zero
        if exec.has_error():
            print("Error occurred:", await exec.error())
    except RuntimeError as e:
        print("Runtime error:", e)
```

#### File Operations

Creating and manipulating files within sandboxes.

```python
async with PythonSandbox.create(name="files") as sb:
    # Create a file using Python
    await sb.run("""
with open('/tmp/test.txt', 'w') as f:
    f.write('Hello from sandbox!')
    """)

    # Read the file using shell command
    result = await sb.command.run("cat", ["/tmp/test.txt"])
    print("File content:", await result.output())
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
The SDK raises `RuntimeError` for various conditions:

- **Connection failures** — Server unreachable or network issues
- **Authentication errors** — Invalid or missing API key
- **Execution timeouts** — Code/commands exceed time limits
- **Invalid operations** — Attempting operations on stopped sandboxes
!!!

#### Best Practices

```python
try:
    async with PythonSandbox.create(name="error-test") as sb:
        exec = await sb.run("potentially_failing_code()")

        if exec.has_error():
            print("Execution error:", await exec.error())
        else:
            print("Success:", await exec.output())

except RuntimeError as e:
    print(f"SDK error: {e}")
except Exception as e:
    print(f"Unexpected error: {e}")
```

---

### Best Practices

!!!success Recommended Practices

1. **Use context managers** — Ensures automatic cleanup
2. **Set appropriate timeouts** — Prevent hanging operations
3. **Handle errors gracefully** — Use try-catch blocks
4. **Monitor resources** — Use metrics interface
5. **Use meaningful names** — Easier debugging and management
6. **Install packages once** — Reuse sandbox for multiple executions
7. **Close HTTP sessions** — When using manual lifecycle management

!!!

!!!warning Common Pitfalls

- Forgetting to call `stop()` with manual lifecycle management
- Not handling execution errors properly
- Setting timeouts too low for complex operations
- Creating too many concurrent sandboxes without resource limits

!!!
