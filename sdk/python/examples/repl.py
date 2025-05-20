#!/usr/bin/env python3
"""
Advanced example demonstrating the Python sandbox features.

This example shows:
1. Different ways to create and manage sandboxes
2. Resource configuration (memory, CPU)
3. Error handling
4. Multiple code execution patterns
5. Output handling
6. Timeouts and handling long-running starts

Before running this example:
    1. Install the package: pip install -e .
    2. Start the Microsandbox server (microsandbox-server)
    3. Run this script: python -m examples.python_sandbox

Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.
"""

import asyncio

import aiohttp
from microsandbox import PythonSandbox


async def example_context_manager():
    """Example using the async context manager pattern."""
    print("\n=== Context Manager Example ===")

    async with PythonSandbox.create(sandbox_name="sandbox-cm") as sandbox:
        # Run some computation
        code = """
print("Hello, world!")
"""
        execution = await sandbox.run(code)
        output = await execution.output()
        print("Output:", output)


async def example_explicit_lifecycle():
    """Example using explicit lifecycle management."""
    print("\n=== Explicit Lifecycle Example ===")

    # Create sandbox with custom configuration
    sandbox = PythonSandbox(
        server_url="http://127.0.0.1:5555", sandbox_name="sandbox-explicit"
    )

    # Create HTTP session
    sandbox._session = aiohttp.ClientSession()

    try:
        # Start with resource constraints
        await sandbox.start(
            memory=1024,  # 1GB RAM
            cpus=2.0,  # 2 CPU cores
        )

        # Run multiple code blocks with variable assignments
        await sandbox.run("x = 42")
        await sandbox.run("y = [i**2 for i in range(10)]")
        execution3 = await sandbox.run("print(f'x = {x}')\nprint(f'y = {y}')")

        print("Output:", await execution3.output())

        # Demonstrate error handling
        try:
            error_execution = await sandbox.run(
                "1/0"
            )  # This will raise a ZeroDivisionError
            print("Error:", await error_execution.error())
        except RuntimeError as e:
            print("Caught error:", e)

    finally:
        # Cleanup
        await sandbox.stop()
        await sandbox._session.close()


async def example_execution_chaining():
    """Example demonstrating execution chaining with variables."""
    print("\n=== Execution Chaining Example ===")

    async with PythonSandbox.create(sandbox_name="sandbox-chain") as sandbox:
        # Execute a sequence of related code blocks
        await sandbox.run("name = 'Python'")
        await sandbox.run("import sys")
        await sandbox.run("version = sys.version")
        exec = await sandbox.run("print(f'Hello from {name} {version}!')")

        # Only get output from the final execution
        print("Output:", await exec.output())


async def main():
    """Run all examples."""
    try:
        await example_context_manager()
        await example_explicit_lifecycle()
        await example_execution_chaining()
    except Exception as e:
        print(f"Error running examples: {e}")


if __name__ == "__main__":
    asyncio.run(main())
