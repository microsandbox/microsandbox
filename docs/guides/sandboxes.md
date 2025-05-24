---
order: 90
icon: container
tags: [guide]
---

# Sandboxes

microsandbox provides different types of sandboxes optimized for specific programming languages and runtimes. Each sandbox type comes with pre-configured environments and language-specific optimizations.

---

### PythonSandbox

The PythonSandbox provides a complete Python environment with access to the Python interpreter, pip package manager, and the ability to execute both Python code and shell commands.

**Features:**

- Full Python 3.x environment
- Package installation with pip
- File system access
- Shell command execution
- Persistent state between executions

#### Usage Examples

+++ Python

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create(name="python-demo") as sb:
        # Execute Python code directly
        exec = await sb.run("print('Hello from Python!')")
        print(await exec.output())

        # Install and use packages
        await sb.run("pip install requests")
        exec = await sb.run("""
import requests
response = requests.get('https://httpbin.org/json')
print(response.status_code)
        """)
        print(await exec.output())

asyncio.run(main())
```

+++ JavaScript

```javascript
import { PythonSandbox } from "microsandbox";

async function main() {
  const sb = await PythonSandbox.create({ name: "python-demo" });

  try {
    // Execute Python code directly
    const exec = await sb.run("print('Hello from Python!')");
    console.log(await exec.output());

    // Install and use packages
    await sb.run("pip install requests");
    const packageExec = await sb.run(`
import requests
response = requests.get('https://httpbin.org/json')
print(response.status_code)
    `);
    console.log(await packageExec.output());
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
    let mut sb = PythonSandbox::create("python-demo").await?;
    sb.start(None).await?;

    // Execute Python code directly
    let exec = sb.run("print('Hello from Python!')", None).await?;
    println!("{}", exec.output().await?);

    // Install and use packages
    sb.run("pip install requests", None).await?;
    let package_exec = sb.run(r#"
import requests
response = requests.get('https://httpbin.org/json')
print(response.status_code)
"#, None).await?;
    println!("{}", package_exec.output().await?);

    sb.stop().await?;
    Ok(())
}
```

+++

---

### NodeSandbox

The NodeSandbox provides a complete Node.js environment with access to the Node.js runtime, npm package manager, and built-in Node.js modules.

**Features:**

- Full Node.js runtime environment
- Access to built-in Node.js modules (fs, os, path, etc.)
- Package installation with npm
- File system access
- Persistent state between executions

#### Usage Examples

+++ Python

```python
import asyncio
from microsandbox import NodeSandbox

async def main():
    async with NodeSandbox.create(name="node-demo") as sb:
        # Execute JavaScript code
        exec = await sb.run("console.log('Hello from Node.js!');")
        print("Output:", await exec.output())

        # Use Node.js built-in modules
        node_code = """
const fs = require('fs');
const os = require('os');

// Write and read a file
fs.writeFileSync('/tmp/test.txt', 'Hello from Node.js!');
const content = fs.readFileSync('/tmp/test.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Platform:', os.platform());
console.log('Node.js version:', process.version);
        """
        exec = await sb.run(node_code)
        print(await exec.output())

asyncio.run(main())
```

+++ JavaScript

```javascript
import { NodeSandbox } from "microsandbox";

async function main() {
  const sb = await NodeSandbox.create({ name: "node-demo" });

  try {
    // Execute JavaScript code
    const exec = await sb.run("console.log('Hello from Node.js!');");
    console.log("Output:", await exec.output());

    // Use Node.js built-in modules
    const nodeCode = `
const fs = require('fs');
const os = require('os');

// Write and read a file
fs.writeFileSync('/tmp/test.txt', 'Hello from Node.js!');
const content = fs.readFileSync('/tmp/test.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Platform:', os.platform());
console.log('Node.js version:', process.version);
    `;
    const nodeExec = await sb.run(nodeCode);
    console.log(await nodeExec.output());
  } finally {
    await sb.stop();
  }
}

main().catch(console.error);
```

+++ Rust

```rust
use microsandbox::{BaseSandbox, NodeSandbox};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut sb = NodeSandbox::create("node-demo").await?;
    sb.start(None).await?;

    // Execute JavaScript code
    let exec = sb.run("console.log('Hello from Node.js!');", None).await?;
    println!("Output: {}", exec.output().await?);

    // Use Node.js built-in modules
    let node_code = r#"
const fs = require('fs');
const os = require('os');

// Write and read a file
fs.writeFileSync('/tmp/test.txt', 'Hello from Node.js!');
const content = fs.readFileSync('/tmp/test.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Platform:', os.platform());
console.log('Node.js version:', process.version);
"#;
    let node_exec = sb.run(node_code, None).await?;
    println!("{}", node_exec.output().await?);

    sb.stop().await?;
    Ok(())
}
```

+++

---

### Choosing the Right Sandbox

#### Use PythonSandbox when:

- You need to execute Python code
- You want to use Python packages from PyPI
- You need data science libraries (pandas, numpy, etc.)
- You're building Python-based applications
- You need to run shell commands alongside Python code

#### Use NodeSandbox when:

- You need to execute JavaScript code
- You want to use npm packages
- You're building Node.js applications
- You need access to Node.js built-in modules
- You're working with web-related JavaScript code

---

### Common Patterns

#### State Persistence

Both sandbox types maintain state between executions within the same session:

```python
# Variables and imports persist between runs
await sb.run("x = 42")
await sb.run("y = x * 2")
exec = await sb.run("print(f'Result: {y}')")  # Outputs: Result: 84
```

#### Error Handling

Both sandboxes provide comprehensive error handling:

```python
exec = await sb.run("invalid_code_here")
if exec.has_error():
    print("Error occurred:", await exec.error())
else:
    print("Success:", await exec.output())
```
