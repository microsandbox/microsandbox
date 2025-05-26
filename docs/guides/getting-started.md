---
order: 100
icon: rocket
tags: [guide]
---

# Getting Started

This guide will help you get up and running with secure code execution in minutes.

!!!info **Platform-specific requirements:**

- **macOS** — Requires Apple Silicon (M1/M2/M3/M4). Intel-based Macs are not currently supported due to virtualization requirements.
- **Linux** — KVM virtualization must be enabled. Most modern Linux distributions support this. You can check if KVM is available by running `lsmod | grep kvm`.
- **Windows** — [Coming soon!](https://github.com/microsandbox/microsandbox/issues/47)
  !!!

---

### Installation

#### Step 1: Install microsandbox

The easiest way to install microsandbox is using our installation script:

```bash
curl -sSL https://get.microsandbox.dev | sh
```

This will download and install the `msb` command-line tool on your system.

#### Step 2: Start the Server

Start the microsandbox server in development mode:

```bash
msb server start --dev
```

!!!info MCP Server
The microsandbox server is also an [MCP server](https://modelcontextprotocol.io), which means it works directly with Claude and other MCP-enabled AI tools out of the box!

[!ref See how to use microsandbox as an MCP server](/guides/mcp)
!!!

---

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

---

### Quick Examples

Here are some quick examples to get you started with common use cases:

#### Execute Commands in Sandbox

+++ Python

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="command-test") as sb:
        # Run shell commands
        result = await sb.command.run("ls", ["-la", "/"])
        print("Directory listing:")
        print(await result.output())

        # Create and execute a script
        await sb.run("""
with open("test.py", "w") as f:
    f.write("print('Hello from sandbox!')")
        """)

        result = await sb.command.run("python", ["test.py"])
        print("Script output:")
        print(await result.output())

asyncio.run(main())
```

+++ JavaScript

```javascript
import { PythonSandbox } from "microsandbox";

async function main() {
  const sb = await PythonSandbox.create({ name: "command-test" });

  try {
    // Run shell commands
    const result = await sb.command.run("ls", ["-la", "/"]);
    console.log("Directory listing:");
    console.log(await result.output());

    // Create and execute a script
    await sb.run(`
with open("test.py", "w") as f:
    f.write("print('Hello from sandbox!')")
    `);

    const scriptResult = await sb.command.run("python", ["test.py"]);
    console.log("Script output:");
    console.log(await scriptResult.output());
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

+++ Rust

```rust
use microsandbox::{BaseSandbox, PythonSandbox};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut sb = PythonSandbox::create("command-test").await?;
    sb.start(None).await?;

    let cmd = sb.command().await?;

    // Run shell commands
    let result = cmd.run("ls", Some(vec!["-la", "/"]), None).await?;
    println!("Directory listing:");
    println!("{}", result.output().await?);

    // Create and execute a script
    sb.run(r#"
with open("test.py", "w") as f:
    f.write("print('Hello from sandbox!')")
"#, None).await?;

    let script_result = cmd.run("python", Some(vec!["test.py"]), None).await?;
    println!("Script output:");
    println!("{}", script_result.output().await?);

    sb.stop().await?;
    Ok(())
}
```

+++

#### Create and Run Bash Scripts

+++ Python

```python
import asyncio
from textwrap import dedent
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="bash-test") as sb:
        # Create and run a bash script
        await sb.run(
            dedent("""
            with open("hello.sh", "w") as f:
                f.write("#!/bin/bash\\n")
                f.write("echo Hello World\\n")
                f.write("date\\n")
        """)
        )

        # Execute the script
        result = await sb.command.run("bash", ["hello.sh"])
        print("Script output:")
        print(await result.output())

asyncio.run(main())
```

+++ JavaScript

```javascript
import { PythonSandbox } from "microsandbox";

async function main() {
  const sb = await PythonSandbox.create({ name: "bash-test" });

  try {
    // Create and run a bash script
    await sb.run(`
with open("hello.sh", "w") as f:
    f.write("#!/bin/bash\\n")
    f.write("echo Hello World\\n")
    f.write("date\\n")
    `);

    // Execute the script
    const result = await sb.command.run("bash", ["hello.sh"]);
    console.log("Script output:");
    console.log(await result.output());
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

+++ Rust

```rust
use microsandbox::{BaseSandbox, PythonSandbox};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut sb = PythonSandbox::create("bash-test").await?;
    sb.start(None).await?;

    // Create and run a bash script
    sb.run(r#"
with open("hello.sh", "w") as f:
    f.write("#!/bin/bash\n")
    f.write("echo Hello World\n")
    f.write("date\n")
"#, None).await?;

    // Execute the script
    let cmd = sb.command().await?;
    let result = cmd.run("bash", Some(vec!["hello.sh"]), None).await?;
    println!("Script output:");
    println!("{}", result.output().await?);

    sb.stop().await?;
    Ok(())
}
```

+++

---

### Troubleshooting

#### First Run Takes Long

The first time you create a sandbox, microsandbox needs to download the base images. This is normal and subsequent runs will be much faster.

#### Server Won't Start

Check that no other services are using the default ports. You can specify custom ports with:

```bash
msb server start --dev --port 8080
```
