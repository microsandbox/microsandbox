---
icon: rocket
title: Getting Started
description: Get up and running with Microsandbox quickly
order: 100
---

# Getting Started

Welcome to Microsandbox! This guide will help you get up and running quickly with secure sandboxes for your AI agents.

## Overview

Microsandbox consists of two main components:

1. **Microsandbox Server** - The backend that manages and orchestrates sandboxes
2. **Microsandbox SDK** - Client libraries for integrating with your applications

## Quick Setup

### 1. Install the CLI

First, install the `msb` command-line tool which helps you manage sandboxes locally.

```bash
curl -sSL https://get.microsandbox.dev | sh
```

### 2. Start the Server

Start your own sandbox server to host secure environments.

```bash
msb server start --dev
```

### 3. Pull Images

Download the sandbox images you'll need.

```bash
msb pull microsandbox/python
msb pull microsandbox/node
```

### 4. Install SDK

Choose your preferred programming language and install the SDK.

+++ Python

```bash
pip install microsandbox
```

+++ JavaScript

```bash
npm install microsandbox
```

+++ Rust

```bash
cargo add microsandbox
```

+++

### 5. Run Your First Sandbox

+++ Python

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="test") as sb:
        exec = await sb.run("print('Hello Microsandbox!')")
        print(await exec.output())

asyncio.run(main())
```

+++ JavaScript

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sb = await NodeSandbox.create({ name: "test" });
  try {
    const exec = await sb.run("console.log('Hello Microsandbox!')");
    console.log(await exec.output());
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

+++ Rust

```rust
use microsandbox::{SandboxOptions, PythonSandbox};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut sb = PythonSandbox::create(
        SandboxOptions::builder().name("test").build()
    ).await?;

    let exec = sb.run(r#"print("Hello Microsandbox!")"#).await?;
    println!("{}", exec.output().await?);

    sb.stop().await?;
    Ok(())
}
```

+++

<div align='center'>• • •</div>

## Platform Requirements

| Platform    | Status         | Requirements                                                             |
| ----------- | -------------- | ------------------------------------------------------------------------ |
| **macOS**   | ✅ Supported   | Apple Silicon (M1/M2/M3/M4)                                              |
| **Linux**   | ✅ Supported   | KVM virtualization enabled                                               |
| **Windows** | 🚧 Coming Soon | [Track progress](https://github.com/microsandbox/microsandbox/issues/47) |

## What's Next?

- [**Installation**](installation.md) - Detailed installation instructions
- [**Self-Hosting**](self-hosting.md) - Set up your own server
- [**SDKs**](../sdks/index.md) - Explore the available SDKs
- [**Examples**](../examples/index.md) - See practical examples

## Need Help?

- Join our [Discord community](https://discord.gg/T95Y3XnEAK)
- Check the [GitHub repository](https://github.com/microsandbox/microsandbox)
- Browse [examples](../examples/index.md) for inspiration
