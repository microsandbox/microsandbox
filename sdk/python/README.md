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

### Running Code

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

### Executing Shell Commands

```python
import asyncio
from microsandbox import PythonSandbox

async def main():
    async with PythonSandbox.create() as sandbox:
        # Execute a command with arguments
        execution = await sandbox.command.run("echo", ["Hello", "World"])

        # Get the command output
        print(await execution.output())  # prints "Hello World"

        # Check the exit code and success status
        print(f"Exit code: {execution.exit_code}")
        print(f"Success: {execution.success}")

        # Handle command errors
        error_cmd = await sandbox.command.run("ls", ["/nonexistent"])
        print(await error_cmd.error())  # prints error message

        # Specify a timeout (in seconds)
        try:
            await sandbox.command.run("sleep", ["10"], timeout=2)
        except RuntimeError as e:
            print(f"Command timed out: {e}")

asyncio.run(main())
```

## Requirements

- Python 3.8+
- Running Microsandbox server (default: http://127.0.0.1:5555)
- API key (if authentication is enabled on the server)

## Environment Variables

- `MSB_API_KEY`: Optional API key for authentication with the Microsandbox server
- `MSB_SERVER_URL`: URL for the Microsandbox server (default: http://127.0.0.1:5555)

## Examples

Check out the [examples directory](./examples) for sample scripts that demonstrate how to:

- Create and use sandboxes
- Run code in sandbox environments
- Execute shell commands in the sandbox
- Handle execution output and error handling

## License

[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0)
