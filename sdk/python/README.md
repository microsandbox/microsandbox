# Microsandbox Python SDK

A Python SDK for interacting with Microsandbox environments.

## Installation

```bash
# Install from PyPI
pip install microsandbox

# Or install from source
git clone https://github.com/microsandbox/microsandbox.git
cd microsandbox/sdk/python
pip install -e .
```

## Basic Usage

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    # Using the context manager (automatically starts and stops the sandbox)
    async with PythonSandbox.create() as sandbox:
        # Run code in the sandbox
        await sandbox.run("name = 'Python'")
        await sandbox.run("print(f'Hello {name}!')")

        # Get the output
        output = await sandbox.output()
        print(output)  # prints Hello Python!

# Run the async main function
asyncio.run(main())
```

## Advanced Usage

```python
import asyncio
import os
from microsandbox import PythonSandbox

async def main():
    # Configure API key (or set MSB_API_KEY environment variable)
    api_key = "msb_your_api_key_here"

    # Create a sandbox explicitly
    sandbox = PythonSandbox(
        server_url="http://127.0.0.1:5555",
        namespace="my-namespace",
        sandbox_name="my-python-sandbox",
        api_key=api_key
    )

    # Create a session
    import aiohttp
    sandbox._session = aiohttp.ClientSession()

    try:
        # Start the sandbox with a specific image and resource constraints
        await sandbox.start(
            image="appcypher/msb-python",
            memory=1024,
            cpus=2.0
        )

        # Run code
        await sandbox.run("import numpy as np")
        await sandbox.run("print(f'NumPy version: {np.__version__}')")

        # Get output
        output = await sandbox.output()
        print(output)
    finally:
        # Cleanup
        await sandbox.stop()
        await sandbox._session.close()

asyncio.run(main())
```

## API Structure

The SDK provides two main classes:

### BaseSandbox (Abstract Base Class)

Provides common functionality for all sandbox types:

- Sandbox lifecycle management (start/stop)
- API communication with Microsandbox server
- Output handling

### PythonSandbox

A concrete implementation of BaseSandbox specifically for Python code execution, with:

- Default Python image configuration
- Python-specific code execution

## Features

- Create isolated sandbox environments
- Execute code snippets remotely
- Retrieve execution output
- Manage sandbox lifecycle with context managers or explicit start/stop
- API key authentication
- Configurable resource limits (memory, CPU)

## Requirements

- Python 3.7+
- Running Microsandbox server (default: http://127.0.0.1:5555)
- API key (if authentication is enabled on the server)

## Environment Variables

- `MSB_API_KEY`: Optional API key for authentication with the Microsandbox server

## Examples

Check out the [examples directory](./examples) for sample scripts that demonstrate how to:

- Create and use sandboxes
- Run code in sandbox environments
- Handle execution output
- Use different API patterns (context manager vs explicit start/stop)

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
