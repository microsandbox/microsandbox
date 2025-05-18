#!/usr/bin/env python3
"""
Advanced example demonstrating the Python sandbox features.

This example shows:
1. Different ways to create and manage sandboxes
2. Resource configuration (memory, CPU)
3. Error handling
4. Multiple code execution patterns
5. Output handling

Before running this example:
    1. Install the package: pip install -e .
    2. Start the Microsandbox server (microsandbox-server)
    3. Run this script: python -m examples.python_sandbox

Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.
"""

import asyncio
import sys
from pathlib import Path

# Add the parent directory to Python path to allow importing the package
sys.path.insert(0, str(Path(__file__).parent.parent))

from microsandbox import PythonSandbox


async def example_context_manager():
    """Example using the async context manager pattern."""
    print("\n=== Context Manager Example ===")

    async with PythonSandbox.create(sandbox_name="sandbox-cm") as sandbox:
        # Run some computation
        code = """
import numpy as np
arr = np.random.rand(1000, 1000)
result = np.mean(arr)
print(f'Mean of random 1000x1000 array: {result:.4f}')
"""
        await sandbox.run(code)
        print("Output:", await sandbox.output())


async def example_explicit_lifecycle():
    """Example using explicit lifecycle management."""
    print("\n=== Explicit Lifecycle Example ===")

    # Create sandbox with custom configuration
    sandbox = PythonSandbox(
        server_url="http://127.0.0.1:5555", sandbox_name="sandbox-explicit"
    )

    # Create HTTP session
    import aiohttp

    sandbox._session = aiohttp.ClientSession()

    try:
        # Start with resource constraints
        await sandbox.start(
            memory=1024,  # 1GB RAM
            cpus=2.0,  # 2 CPU cores
        )

        # Run multiple code blocks
        await sandbox.run("x = 42")
        await sandbox.run("y = [i**2 for i in range(10)]")
        await sandbox.run("print(f'x = {x}')\nprint(f'y = {y}')")

        print("Output:", await sandbox.output())

        # Demonstrate error handling
        try:
            await sandbox.run("1/0")  # This will raise a ZeroDivisionError
        except RuntimeError as e:
            print("Caught error:", e)

    finally:
        # Cleanup
        await sandbox.stop()
        await sandbox._session.close()


async def example_scientific_computing():
    """Example demonstrating scientific computing capabilities."""
    print("\n=== Scientific Computing Example ===")

    async with PythonSandbox.create(sandbox_name="sandbox-sci") as sandbox:
        # Run a more complex scientific computation
        code = """
import numpy as np
from scipy import stats

# Generate sample data
data = np.random.normal(loc=0, scale=1, size=1000)

# Compute statistics
mean = np.mean(data)
std = np.std(data)
skew = stats.skew(data)
kurtosis = stats.kurtosis(data)

print(f'Sample Statistics:')
print(f'Mean: {mean:.4f}')
print(f'Std Dev: {std:.4f}')
print(f'Skewness: {skew:.4f}')
print(f'Kurtosis: {kurtosis:.4f}')

# Create a histogram
hist, bins = np.histogram(data, bins=30)
print(f'\\nHistogram Bins: {bins[0]:.2f} to {bins[-1]:.2f}')
print(f'Max Frequency: {max(hist)}')
"""
        await sandbox.run(code)
        print("Output:", await sandbox.output())


async def main():
    """Run all examples."""
    try:
        await example_context_manager()
        await example_explicit_lifecycle()
        await example_scientific_computing()
    except Exception as e:
        print(f"Error running examples: {e}")


if __name__ == "__main__":
    asyncio.run(main())
