---
order: 100
icon: rocket
tags: [guide]
---

# Getting Started

This guide will help you get up and running with secure code execution in minutes.

!!!info Prerequisites
Before starting, make sure you have:

- A supported operating system (macOS or Linux)
- Internet connection for downloading images
- Basic familiarity with command line
  !!!

## Installation

#### Step 1: Install microsandbox

The easiest way to install microsandbox is using our installation script:

```bash
curl -sSL https://get.microsandbox.dev | sh
```

This will download and install the `msb` command-line tool on your system.

!!!info Platform Support

- :icon-check-circle: **macOS** - Fully supported
- :icon-check-circle: **Linux** - Fully supported
- :icon-clock: **Windows** - Work in progress
  !!!

#### Step 2: Start the Server

Start the microsandbox server in development mode:

```bash
msb server start --dev
```

This will:

- Start the microsandbox server locally
- Download necessary VM images (this may take a few minutes on first run)
- Enable the MCP server for AI tool integration

!!!tip MCP Integration
The microsandbox server is also an [MCP server](https://modelcontextprotocol.io), which means it works directly with Cursor, Claude, and other MCP-enabled AI tools out of the box!
!!!

### Your First Sandbox

microsandbox provides SDKs for multiple programming languages. Choose your preferred language below:

+++ Python
Install the Python SDK:

```bash
pip install microsandbox
```

Create your first sandbox:

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="my-first-sandbox") as sb:
        # Execute some Python code
        exec = await sb.run("name = 'World'")
        exec = await sb.run("print(f'Hello {name}!')")

        # Get the output
        output = await exec.output()
        print(output)  # prints: Hello World!

asyncio.run(main())
```

+++ JavaScript
Install the JavaScript SDK:

```bash
npm install microsandbox
```

Create your first sandbox:

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sb = await NodeSandbox.create({ name: "my-first-sandbox" });

  try {
    // Execute some JavaScript code
    let exec = await sb.run("var name = 'World'");
    exec = await sb.run("console.log(`Hello ${name}!`)");

    // Get the output
    const output = await exec.output();
    console.log(output); // prints: Hello World!
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

+++ Rust
Add microsandbox to your `Cargo.toml`:

```bash
cargo add microsandbox
```

Create your first sandbox:

```rust
use microsandbox::{SandboxOptions, PythonSandbox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = PythonSandbox::create(
        SandboxOptions::builder()
            .name("my-first-sandbox")
            .build()
    ).await?;

    // Execute some Python code
    let exec = sb.run(r#"name = "World""#).await?;
    let exec = sb.run(r#"print(f"Hello {name}!")"#).await?;

    // Get the output
    let output = exec.output().await?;
    println!("{}", output); // prints: Hello World!

    sb.stop().await?;
    Ok(())
}
```

+++

!!!success Congratulations!
You've successfully created and executed code in your first microsandbox! The code ran in a completely isolated microVM, protecting your system while providing full execution capabilities.
!!!

### Quick Examples

Here are some quick examples to get you started with common use cases:

#### Execute a Simple Script

```python
async with PythonSandbox.create(name="script-test") as sb:
    script = """
    import math
    result = math.sqrt(16)
    print(f"Square root of 16 is: {result}")
    """
    exec = await sb.run(script)
    print(await exec.output())
```

#### Install and Use Packages

```python
async with PythonSandbox.create(name="package-test") as sb:
    # Install a package
    await sb.run("pip install requests")

    # Use the package
    code = """
    import requests
    response = requests.get('https://httpbin.org/json')
    print(response.json())
    """
    exec = await sb.run(code)
    print(await exec.output())
```

#### File Operations

```python
async with PythonSandbox.create(name="file-test") as sb:
    # Create a file
    await sb.run("with open('test.txt', 'w') as f: f.write('Hello from sandbox!')")

    # Read the file
    exec = await sb.run("with open('test.txt', 'r') as f: print(f.read())")
    print(await exec.output())
```

### Troubleshooting

#### First Run Takes Long

The first time you create a sandbox, microsandbox needs to download the base images. This is normal and subsequent runs will be much faster.

#### Permission Issues

If you encounter permission issues, make sure your user has access to virtualization features on your system.

#### Server Won't Start

Check that no other services are using the default ports. You can specify custom ports with:

```bash
msb server start --dev --port 8080
```

#### Import Errors

If you get import errors with the SDK, make sure you've installed it correctly:

```bash
# For Python
pip install microsandbox

# For JavaScript
npm install microsandbox

# For Rust
cargo add microsandbox
```

### Next Steps

Now that you have microsandbox running, explore these topics:

=== :icon-project: **Projects**
Learn about project-based development with Sandboxfiles

[!ref Project Management](projects.md) - Working with Sandboxfiles and project workflows

=== :icon-organization: **Architecture**
Understand how microsandbox works under the hood

[!ref Architecture Guide](architecture.md) - System architecture and security model

=== :icon-package: **SDK Deep Dive**
Master the SDKs for your preferred language

[!ref SDK Documentation](/sdk/) - Detailed SDK documentation and advanced usage

=== :icon-gear: **Configuration**
Customize microsandbox for your needs

# [!ref Configuration Guide](configuration.md) - Server and sandbox configuration options

!!!question Need Help?
Join our community for support:

- [Discord](https://discord.gg/T95Y3XnEAK)
- [GitHub Issues](https://github.com/microsandbox/microsandbox/issues)
- [Reddit](https://www.reddit.com/r/microsandbox)
  !!!
