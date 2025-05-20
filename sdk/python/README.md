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

## Usage

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    # Using the context manager (automatically starts and stops the sandbox)
    async with PythonSandbox.create() as sandbox:
        # Run code in the sandbox
        await sandbox.run("name = 'Python'")
        execution = await sandbox.run("print(f'Hello {name}!')")

        # Get the output
        output = await execution.output()
        print(output)  # prints Hello Python!

# Run the async main function
asyncio.run(main())
```

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

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
