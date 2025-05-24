---
order: 80
icon: file-code
tags: [references]
---

# Rust SDK Reference

Complete reference documentation for the microsandbox Rust SDK.

---

### Installation

```bash
cargo add microsandbox
```

---

### Quick Start

```rust
use microsandbox::{PythonSandbox, SandboxOptions};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut sb = PythonSandbox::create("my-sandbox").await?;
    sb.start(None).await?;

    let exec = sb.run("print('Hello, World!')").await?;
    println!("{}", exec.output().await?);

    sb.stop().await?;
    Ok(())
}
```

---

### PythonSandbox

The main struct for creating and managing Python execution environments.

==- Constructor
```rust
PythonSandbox::create(name: &str) -> Result<Self, Box<dyn Error + Send + Sync>>
PythonSandbox::create_with_options(options: SandboxOptions) -> Result<Self, Box<dyn Error + Send + Sync>>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `&str` | Sandbox identifier |
| `options` | `SandboxOptions` | Configuration options for the sandbox |

```rust
// Simple creation with name
let sb = PythonSandbox::create("my-sandbox").await?;

// Creation with options
let options = SandboxOptions::builder()
    .name("my-sandbox")
    .namespace("default")
    .server_url("http://127.0.0.1:5555")
    .api_key("my-key")
    .build();
let sb = PythonSandbox::create_with_options(options).await?;
```
===

#### Instance Methods

==- `start()`
Starts the sandbox with optional resource constraints.

```rust
async fn start(
    &mut self,
    options: Option<StartOptions>
) -> Result<(), Box<dyn Error + Send + Sync>>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `options` | `StartOptions` | Optional configuration for sandbox startup |

```rust
// Start with defaults
sb.start(None).await?;

// Start with custom options
let options = StartOptions {
    image: Some("microsandbox/python".to_string()),
    memory: 1024,
    cpus: 2.0,
    timeout: 180.0,
};
sb.start(Some(options)).await?;
```
===

==- `stop()`
Stops and cleans up the sandbox.

```rust
async fn stop(&mut self) -> Result<(), Box<dyn Error + Send + Sync>>
```
===

==- `run()`
Executes Python code in the sandbox environment.

```rust
async fn run(
    &self,
    code: &str
) -> Result<Execution, Box<dyn Error + Send + Sync>>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `code` | `&str` | Python code to execute |

**Returns:** `Execution` object with results

+++ Simple Execution
```rust
let exec = sb.run("print('Hello, World!')").await?;
println!("{}", exec.output().await?);
```
+++ Multi-line Code
```rust
let code = r#"
import math
result = math.sqrt(16)
print(f"Square root of 16 is: {result}")
"#;
let exec = sb.run(code).await?;
```
+++
===

#### Methods

==- `command()`
Access to shell command execution interface.

```rust
async fn command(&self) -> Result<Command, Box<dyn Error + Send + Sync>>
```

```rust
let cmd = sb.command().await?;
let result = cmd.run("ls", Some(vec!["-la", "/"]), None).await?;
println!("{}", result.output().await?);
```
===

==- `metrics()`
Access to resource monitoring interface.

```rust
async fn metrics(&self) -> Result<Metrics, Box<dyn Error + Send + Sync>>
```

```rust
let metrics = sb.metrics().await?;
let cpu = metrics.cpu().await?;
let memory = metrics.memory().await?;
println!("CPU: {}%, Memory: {} MiB", cpu.unwrap_or(0.0), memory.unwrap_or(0));
```
===

---

### NodeSandbox

JavaScript/Node.js execution environment with identical API to PythonSandbox.

==- Constructor
```rust
NodeSandbox::create(name: &str) -> Result<Self, Box<dyn Error + Send + Sync>>
NodeSandbox::create_with_options(options: SandboxOptions) -> Result<Self, Box<dyn Error + Send + Sync>>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `&str` | Sandbox identifier |
| `options` | `SandboxOptions` | Configuration options for the sandbox |
===

All methods are identical to `PythonSandbox` but execute JavaScript code.

```rust
use microsandbox::NodeSandbox;

let mut sb = NodeSandbox::create("js-sandbox").await?;
sb.start(None).await?;

let exec = sb.run("console.log('Hello from Node.js!');").await?;
println!("{}", exec.output().await?);

sb.stop().await?;
```

---

### Execution

Represents the result of code execution from `sandbox.run()`.

#### Methods

==- `output()`
Returns standard output from execution.

```rust
async fn output(&self) -> Result<String, Box<dyn Error + Send + Sync>>
```
===

==- `error()`
Returns standard error from execution.

```rust
async fn error(&self) -> Result<String, Box<dyn Error + Send + Sync>>
```
===

==- `has_error()`
Checks if execution produced errors.

```rust
fn has_error(&self) -> bool
```
===

!!!warning Example: Handling Execution Results
```rust
let exec = sb.run("print('Hello'); import sys; sys.exit(1)").await?;
println!("Has error: {}", exec.has_error());
println!("Output: {}", exec.output().await?);
if exec.has_error() {
    println!("Error: {}", exec.error().await?);
}
```
!!!

---

### CommandExecution

Represents the result of command execution from `sandbox.command.run()`.

#### Methods

==- `output()`
Returns standard output from command execution.

```rust
async fn output(&self) -> Result<String, Box<dyn Error + Send + Sync>>
```
===

==- `error()`
Returns standard error from command execution.

```rust
async fn error(&self) -> Result<String, Box<dyn Error + Send + Sync>>
```
===

==- `exit_code()`
Command exit code.

```rust
fn exit_code(&self) -> i32
```
===

==- `success()`
True if command was successful (exit code 0).

```rust
fn success(&self) -> bool
```
===

---

### Command

Interface for executing shell commands within sandboxes.

==- `run()`
Executes shell commands with arguments.

```rust
async fn run(
    &self,
    command: &str,
    args: Option<Vec<&str>>,
    timeout: Option<u32>
) -> Result<CommandExecution, Box<dyn Error + Send + Sync>>
```

| Parameter | Type | Description |
|-----------|------|-------------|
| `command` | `&str` | Command to execute |
| `args` | `Option<Vec<&str>>` | Command arguments (optional) |
| `timeout` | `Option<u32>` | Execution timeout in seconds (optional) |

**Returns:** `CommandExecution` object with results

+++ Simple Command
```rust
let result = cmd.run("ls", None, None).await?;
```
+++ With Arguments
```rust
let result = cmd.run("ls", Some(vec!["-la", "/tmp"]), None).await?;
```
+++ With Timeout
```rust
let result = cmd.run("sleep", Some(vec!["10"]), Some(5)).await?;
```
+++ Error Handling
```rust
let result = cmd.run("ls", Some(vec!["/nonexistent"]), None).await?;
if result.success() {
    println!("Output: {}", result.output().await?);
} else {
    println!("Error: {}", result.error().await?);
    println!("Exit code: {}", result.exit_code());
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

```rust
async fn cpu(&self) -> Result<Option<f32>, Box<dyn Error + Send + Sync>>
```
===

==- `memory()`
Current memory usage in MiB.

```rust
async fn memory(&self) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>
```
===

==- `disk()`
Current disk usage in bytes.

```rust
async fn disk(&self) -> Result<Option<u64>, Box<dyn Error + Send + Sync>>
```
===

==- `is_running()`
Sandbox running status.

```rust
async fn is_running(&self) -> Result<bool, Box<dyn Error + Send + Sync>>
```
===

==- `all()`
All metrics as a JSON value.

```rust
async fn all(&self) -> Result<serde_json::Value, Box<dyn Error + Send + Sync>>
```

**Returns:** JSON object with keys: `name`, `namespace`, `running`, `cpu_usage`, `memory_usage`, `disk_usage`

```rust
// Individual metrics
let metrics = sb.metrics().await?;
let cpu = metrics.cpu().await?;
let memory = metrics.memory().await?;
let disk = metrics.disk().await?;
let running = metrics.is_running().await?;

println!("CPU: {}%", cpu.unwrap_or(0.0));
println!("Memory: {} MiB", memory.unwrap_or(0));
println!("Disk: {} bytes", disk.unwrap_or(0));
println!("Running: {}", running);

// All metrics at once
let all_metrics = metrics.all().await?;
println!("All metrics: {}", all_metrics);
```
===

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
The SDK provides a `SandboxError` enum for various conditions:

- **NotStarted** — Attempting operations on stopped sandboxes
- **RequestFailed** — Server unreachable or network issues
- **ServerError** — Server-side errors
- **Timeout** — Operations exceed time limits
- **HttpError** — Network-related errors
- **InvalidResponse** — Malformed server responses
!!!

#### Best Practices

```rust
use microsandbox::{PythonSandbox, SandboxError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut sb = PythonSandbox::create("error-test").await?;

    match sb.start(None).await {
        Ok(_) => {
            let exec = sb.run("potentially_failing_code()").await?;
            if exec.has_error() {
                println!("Execution error: {}", exec.error().await?);
            } else {
                println!("Success: {}", exec.output().await?);
            }
            sb.stop().await?;
        }
        Err(e) => {
            if let Some(sandbox_err) = e.downcast_ref::<SandboxError>() {
                println!("Sandbox error: {}", sandbox_err);
            } else {
                println!("Unknown error: {}", e);
            }
        }
    }
    Ok(())
}
```

---

### Best Practices

!!!success Recommended Practices

1. **Use Result types** — Handle all potential errors
2. **Set appropriate timeouts** — Prevent hanging operations
3. **Handle errors gracefully** — Match on error types
4. **Monitor resources** — Use metrics interface
5. **Use meaningful names** — Easier debugging and management
6. **Install packages once** — Reuse sandbox for multiple executions
7. **Always call stop()** — Prevent resource leaks

!!!

!!!warning Common Pitfalls

- Forgetting to call `stop()` after sandbox usage
- Not handling all potential error cases
- Setting timeouts too low for complex operations
- Creating too many concurrent sandboxes without resource limits

!!!
