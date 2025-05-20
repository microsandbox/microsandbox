#!/usr/bin/env python3
"""
Example demonstrating how to use sandbox.metrics.get to retrieve sandbox metrics.
"""

import asyncio
import time

from microsandbox import PythonSandbox


async def basic_metrics_example():
    """Example showing basic metrics retrieval."""
    print("\n=== Basic Metrics Example ===")

    # Create a sandbox using a context manager (automatically handles start/stop)
    async with PythonSandbox.create(sandbox_name="metrics-example") as sandbox:
        # Get initial sandbox metrics
        print("Getting initial sandbox metrics...")
        metrics = await sandbox.metrics.get()

        print(f"Sandbox running: {metrics.running}")
        print(f"CPU usage: {metrics.cpu_usage:.2f}%")
        print(f"Memory usage: {metrics.memory_usage_mb:.2f} MB")
        print(f"Disk usage: {metrics.disk_usage_mb:.2f} MB")

        # Run some code to generate some activity
        print("\nRunning a CPU-intensive task...")
        await sandbox.run("""
import time
import math

# Perform some CPU-intensive calculations
start = time.time()
for i in range(1000000):
    math.sqrt(i)
end = time.time()
print(f"Calculation took {end - start:.2f} seconds")
        """)

        # Wait a moment for metrics to update
        await asyncio.sleep(1)

        # Get updated metrics after running the code
        print("\nGetting updated metrics after CPU task...")
        metrics = await sandbox.metrics.get()

        print(f"Sandbox running: {metrics.running}")
        print(f"CPU usage: {metrics.cpu_usage:.2f}%")
        print(f"Memory usage: {metrics.memory_usage_mb:.2f} MB")
        print(f"Disk usage: {metrics.disk_usage_mb:.2f} MB")


async def monitoring_example():
    """Example showing how to monitor sandbox metrics over time."""
    print("\n=== Monitoring Metrics Example ===")

    async with PythonSandbox.create(sandbox_name="monitoring-example") as sandbox:
        # Monitor metrics while running various operations
        print("Starting monitoring...")

        # Get baseline metrics
        baseline = await sandbox.metrics.get()
        print(
            f"Baseline - Memory: {baseline.memory_usage_mb:.2f} MB, CPU: {baseline.cpu_usage:.2f}%"
        )

        # Run a memory-intensive operation
        print("\nRunning memory-intensive operation...")
        await sandbox.run("""
# Allocate a large list in memory
large_list = ["x" * 1000 for _ in range(1000000)]
print(f"List created with {len(large_list)} elements")
        """)

        # Check metrics after memory operation
        await asyncio.sleep(1)
        mem_metrics = await sandbox.metrics.get()
        print(
            f"After memory operation - Memory: {mem_metrics.memory_usage_mb:.2f} MB, CPU: {mem_metrics.cpu_usage:.2f}%"
        )

        # Run a disk operation
        print("\nRunning disk operation...")
        await sandbox.command.run(
            "dd", ["if=/dev/zero", "of=/tmp/testfile", "bs=1M", "count=100"]
        )
        await sandbox.command.run("sync")

        # Check metrics after disk operation
        await asyncio.sleep(1)
        disk_metrics = await sandbox.metrics.get()
        print(
            f"After disk operation - Disk: {disk_metrics.disk_usage_mb:.2f} MB, Memory: {disk_metrics.memory_usage_mb:.2f} MB"
        )

        # Clean up
        await sandbox.command.run("rm", ["-f", "/tmp/testfile"])


async def advanced_metrics_example():
    """Example showing more advanced metrics usage."""
    print("\n=== Advanced Metrics Example ===")

    async with PythonSandbox.create(sandbox_name="advanced-example") as sandbox:
        # Get all raw metrics data
        metrics = await sandbox.metrics.get()

        print("Raw metrics data:")
        print(f"  {metrics.raw_data}")

        # Perform continuous monitoring for a short period
        print("\nContinuous monitoring for 10 seconds:")

        start_time = time.time()
        while time.time() - start_time < 10:
            # Run a short task to create some activity
            await sandbox.run("import random; [random.random() for _ in range(100000)]")

            # Get current metrics
            metrics = await sandbox.metrics.get()
            print(
                f"Time: {time.time() - start_time:.1f}s, CPU: {metrics.cpu_usage:.2f}%, "
                f"Memory: {metrics.memory_usage_mb:.2f} MB"
            )

            # Short pause between measurements
            await asyncio.sleep(1)


async def main():
    """Main function to run all examples."""
    print("Sandbox Metrics Examples")
    print("=======================")

    await basic_metrics_example()
    await monitoring_example()
    await advanced_metrics_example()

    print("\nAll examples completed!")


if __name__ == "__main__":
    asyncio.run(main())
