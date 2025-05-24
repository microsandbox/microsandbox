---
icon: code
title: SDKs
description: Client libraries for integrating Microsandbox into your applications
order: 300
---

# SDKs

Microsandbox provides client libraries (SDKs) for multiple programming languages, making it easy to integrate secure sandboxes into your applications.

## Available SDKs

### Python SDK

The Python SDK provides async/await support and context managers for clean resource management.

**Features:**

- ✅ Async/await support
- ✅ Context manager integration
- ✅ Type hints and IDE support
- ✅ Error handling and timeouts
- ⚠️ Shell command execution (coming soon)

**Installation:**

```bash
pip install microsandbox
```

**Quick Example:**

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="example") as sandbox:
        execution = await sandbox.run("print('Hello from Python!')")
        print(await execution.output())

asyncio.run(main())
```

### JavaScript/TypeScript SDK

The JavaScript SDK works in both Node.js and browser environments with full TypeScript support.

**Features:**

- ✅ Promise-based API
- ✅ TypeScript definitions
- ✅ Node.js and browser support
- ✅ Error handling and timeouts
- ⚠️ Shell command execution (coming soon)

**Installation:**

```bash
npm install microsandbox
```

**Quick Example:**

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sandbox = await NodeSandbox.create({ name: "example" });
  try {
    const execution = await sandbox.run("console.log('Hello from Node!')");
    console.log(await execution.output());
  } finally {
    await sandbox.stop();
  }
}

main().catch(console.error);
```

### Rust SDK

The Rust SDK provides zero-cost abstractions and compile-time safety guarantees.

**Features:**

- ✅ Async/await support
- ✅ Type safety and error handling
- ✅ Zero-cost abstractions
- ✅ Memory safety guarantees
- ⚠️ Shell command execution (coming soon)

**Installation:**

```bash
cargo add microsandbox
```

**Quick Example:**

```rust
use microsandbox::{SandboxOptions, PythonSandbox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = PythonSandbox::create(
        SandboxOptions::builder().name("example").build()
    ).await?;

    let execution = sandbox.run("print('Hello from Rust!')").await?;
    println!("{}", execution.output().await?);

    sandbox.stop().await?;
    Ok(())
}
```

<div align='center'>• • •</div>

## Common Concepts

All SDKs share these core concepts:

### Sandbox Types

- **PythonSandbox** - Isolated Python environments
- **NodeSandbox** - Isolated Node.js environments
- **CustomSandbox** - User-defined container environments

### Lifecycle Management

1. **Create** - Initialize a new sandbox instance
2. **Start** - Boot the sandbox with specified resources
3. **Execute** - Run code or commands in the sandbox
4. **Stop** - Clean up and destroy the sandbox

### Resource Configuration

- **Memory** - RAM allocation (MB)
- **CPU** - CPU core allocation
- **Timeout** - Maximum execution time
- **Network** - Network access policies

### Error Handling

- **Execution Errors** - Code runtime errors
- **Timeout Errors** - Long-running execution limits
- **Resource Errors** - Insufficient resources
- **Network Errors** - Connection issues

<div align='center'>• • •</div>

## Implementation Status

| Feature               | Python | JavaScript | Rust |
| --------------------- | ------ | ---------- | ---- |
| Basic code execution  | ✅     | ✅         | ✅   |
| Resource limits       | ✅     | ✅         | ✅   |
| Error handling        | ✅     | ✅         | ✅   |
| Async/await           | ✅     | ✅         | ✅   |
| Context managers      | ✅     | ❌         | ❌   |
| Shell commands        | ⚠️     | ⚠️         | ⚠️   |
| File operations       | ⚠️     | ⚠️         | ⚠️   |
| Network configuration | ⚠️     | ⚠️         | ⚠️   |

**Legend:** ✅ Implemented | ⚠️ Coming Soon | ❌ Not Planned

<div align='center'>• • •</div>

## Quick Start Examples

### Basic Code Execution

+++ Python

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create() as sandbox:
        # Execute code and get result
        execution = await sandbox.run("2 + 2")
        print(await execution.output())  # "4"

asyncio.run(main())
```

+++ JavaScript

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sandbox = await NodeSandbox.create();
  try {
    const execution = await sandbox.run("console.log(2 + 2)");
    console.log(await execution.output()); // "4"
  } finally {
    await sandbox.stop();
  }
}
```

+++ Rust

```rust
use microsandbox::PythonSandbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = PythonSandbox::create("example").await?;

    let execution = sandbox.run("print(2 + 2)").await?;
    println!("{}", execution.output().await?); // "4"

    sandbox.stop().await?;
    Ok(())
}
```

+++

### Shell Command Execution

+++ Python

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create() as sandbox:
        # Execute shell command
        execution = await sandbox.command.run("echo", ["Hello World"])
        print(await execution.output())  # "Hello World"

asyncio.run(main())
```

+++ JavaScript

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sandbox = await NodeSandbox.create();
  try {
    const execution = await sandbox.command.run("echo", ["Hello World"]);
    console.log(await execution.output()); // "Hello World"
  } finally {
    await sandbox.stop();
  }
}
```

+++ Rust

```rust
use microsandbox::PythonSandbox;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sandbox = PythonSandbox::create("example").await?;

    let execution = sandbox.command().run("echo", &["Hello World"]).await?;
    println!("{}", execution.output().await?); // "Hello World"

    sandbox.stop().await?;
    Ok(())
}
```

+++

<div align='center'>• • •</div>

## Configuration

### Server Connection

All SDKs can be configured to connect to your Microsandbox server:

+++ Python

```python
from microsandbox import PythonSandbox

# Using environment variables
# MSB_SERVER_URL=http://localhost:5555
# MSB_API_KEY=your_api_key

sandbox = PythonSandbox(
    server_url="http://localhost:5555",
    api_key="your_api_key"
)
```

+++ JavaScript

```javascript
import { NodeSandbox } from "microsandbox";

const sandbox = await NodeSandbox.create({
  serverUrl: "http://localhost:5555",
  apiKey: "your_api_key",
});
```

+++ Rust

```rust
use microsandbox::{SandboxOptions, PythonSandbox};

let options = SandboxOptions::builder()
    .server_url("http://localhost:5555")
    .api_key("your_api_key")
    .build();

let sandbox = PythonSandbox::create_with_options(options).await?;
```

+++

### Resource Limits

Configure memory and CPU limits for your sandboxes:

+++ Python

```python
async with PythonSandbox.create() as sandbox:
    await sandbox.start(
        memory=1024,  # 1GB RAM
        cpus=2.0      # 2 CPU cores
    )
```

+++ JavaScript

```javascript
const sandbox = await NodeSandbox.create();
await sandbox.start({
  memory: 1024, // 1GB RAM
  cpus: 2.0, // 2 CPU cores
});
```

+++ Rust

```rust
let start_options = StartOptions {
    memory: 1024,  // 1GB RAM
    cpus: 2.0,     // 2 CPU cores
    ..Default::default()
};

sandbox.start(Some(start_options)).await?;
```

+++

<div align='center'>• • •</div>

## Other Language SDKs

We're actively working on SDKs for additional languages. Interested in contributing? Check out our [contribution guidelines](https://github.com/microsandbox/microsandbox/blob/main/CONTRIBUTING.md).

**Coming Soon:**

- Go SDK
- Java SDK
- C# SDK
- PHP SDK

## What's Next?

- [**Examples**](../examples/index.md) - See practical SDK usage examples
- [**CLI Reference**](../cli/index.md) - Learn about the CLI tools
- [**Self-Hosting**](../getting-started/self-hosting.md) - Set up your own server

---

!!!info "Need help?"
Join our [Discord community](https://discord.gg/T95Y3XnEAK) or check out the [GitHub repository](https://github.com/microsandbox/microsandbox).
!!!
