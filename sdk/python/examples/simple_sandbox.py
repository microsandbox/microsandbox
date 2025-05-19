#!/usr/bin/env python3
"""
Simple example demonstrating the Python sandbox API.

This example shows the basic API pattern for using the Microsandbox Python SDK.

Before running this example:
    1. Install the package: pip install -e .
    2. Start the Microsandbox server (microsandbox-server)
    3. Run this script: python -m examples.simple_sandbox

Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.
"""

import asyncio
import sys
from pathlib import Path

# Add the parent directory to Python path to allow importing the package
sys.path.insert(0, str(Path(__file__).parent.parent))

from microsandbox import PythonSandbox


async def main():
    """Demonstrates the basic API pattern of the Python sandbox."""
    try:
        # Create a sandbox using the context manager
        async with PythonSandbox.create() as sb:
            # Run first statement that sets a variable
            execution = await sb.run("name = 'Python'")

            # Run second statement that uses the variable
            execution = await sb.run("print(f'Hello {name}!')")

            # Get output from the last execution
            output = await execution.output()
            print(output)  # prints "Hello Python!"

    except Exception as e:
        print(f"Error: {e}")


if __name__ == "__main__":
    asyncio.run(main())
