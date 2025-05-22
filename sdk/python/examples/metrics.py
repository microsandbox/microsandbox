#!/usr/bin/env python3
"""
Example demonstrating how to retrieve sandbox metrics.
"""

import asyncio
import time

import aiohttp
from microsandbox import PythonSandbox


async def basic_metrics_example():
    """Example showing how to get individual metrics for a sandbox."""
    print("\n=== Basic Metrics Example ===")

    # Create a sandbox using a context manager
    async with PythonSandbox.create(name="metrics-example") as sandbox:
        # Run a command to generate some load
        print("Running commands to generate some sandbox activity...")
        await sandbox.command.run("ls", ["-la", "/"])
        await sandbox.command.run(
            "dd", ["if=/dev/zero", "of=/tmp/testfile", "bs=1M", "count=10"]
        )

        # Sleep a moment to allow metrics to update
        time.sleep(1)

        # Get individual metrics
        print("\nGetting individual metrics for this sandbox:")

        try:
            # Get CPU usage
            cpu = await sandbox.metrics.cpu()
            # CPU metrics may be 0.0 when idle or None if unavailable
            if cpu is None:
                print("CPU Usage: Not available")
            else:
                print(f"CPU Usage: {cpu}%")

            # Get memory usage
            memory = await sandbox.metrics.memory()
            print(f"Memory Usage: {memory or 'Not available'} MiB")

            # Get disk usage
            disk = await sandbox.metrics.disk()
            print(f"Disk Usage: {disk or 'Not available'} bytes")

            # Check if running
            running = await sandbox.metrics.is_running()
            print(f"Is Running: {running}")
        except RuntimeError as e:
            print(f"Error getting metrics: {e}")


async def all_metrics_example():
    """Example showing how to get all metrics at once."""
    print("\n=== All Metrics Example ===")

    # Create a sandbox
    async with PythonSandbox.create(name="all-metrics-example") as sandbox:
        try:
            # Run some commands to generate activity
            print("Running commands to generate some sandbox activity...")
            await sandbox.command.run("cat", ["/etc/os-release"])
            # Using a simpler command that won't time out or cause errors
            await sandbox.command.run("ls", ["-la", "/usr"])

            # Sleep a moment to allow metrics to update
            time.sleep(1)

            # Get all metrics at once
            print("\nGetting all metrics as a dictionary:")
            all_metrics = await sandbox.metrics.all()

            # Print formatted metrics
            print(
                f"Sandbox: {all_metrics.get('name')} (namespace: {all_metrics.get('namespace')})"
            )
            print(f"  Running: {all_metrics.get('running')}")

            # Handle CPU metrics which may be 0.0 or None
            cpu = all_metrics.get("cpu_usage")
            if cpu is None:
                print("  CPU Usage: Not available")
            else:
                print(f"  CPU Usage: {cpu}%")

            print(
                f"  Memory Usage: {all_metrics.get('memory_usage') or 'Not available'} MiB"
            )
            print(
                f"  Disk Usage: {all_metrics.get('disk_usage') or 'Not available'} bytes"
            )
        except Exception as e:
            print(f"Error in all_metrics_example: {e}")


async def continuous_monitoring_example():
    """Example showing how to continuously monitor sandbox metrics."""
    print("\n=== Continuous Monitoring Example ===")

    # Create a sandbox
    async with PythonSandbox.create(name="monitoring-example") as sandbox:
        try:
            print("Starting continuous monitoring (5 seconds)...")

            # Generate load with a simple and safe command
            _ = await sandbox.command.run(
                "sh",
                [
                    "-c",
                    "for i in $(seq 1 5); do ls -la / > /dev/null; sleep 0.2; done &",
                ],
            )

            # Monitor for 5 seconds
            start_time = time.time()
            while time.time() - start_time < 5:
                try:
                    # Get metrics
                    cpu = await sandbox.metrics.cpu()
                    memory = await sandbox.metrics.memory()

                    # Format CPU usage (could be 0.0 or None)
                    cpu_str = f"{cpu}%" if cpu is not None else "Not available"

                    # Print current values
                    print(
                        f"[{time.time() - start_time:.1f}s] CPU: {cpu_str}, Memory: {memory or 'Not available'} MiB"
                    )
                except Exception as e:
                    print(f"Error getting metrics: {e}")

                # Wait before next check
                await asyncio.sleep(1)

            print("Monitoring complete.")
        except Exception as e:
            print(f"Error in continuous_monitoring_example: {e}")


async def cpu_load_test_example():
    """Example generating CPU load to test CPU metrics."""
    print("\n=== CPU Load Test Example ===")

    # Create a sandbox
    async with PythonSandbox.create(name="cpu-load-test") as sandbox:
        try:
            # Run a CPU-intensive Python script
            print("Running CPU-intensive task...")

            # First create a Python script that will use CPU
            cpu_script = """
import time
start = time.time()
duration = 10  # seconds

# CPU-intensive calculation
while time.time() - start < duration:
    # Calculate prime numbers - CPU intensive
    for i in range(1, 100000):
        is_prime = True
        for j in range(2, int(i ** 0.5) + 1):
            if i % j == 0:
                is_prime = False
                break

    # Print progress every second
    elapsed = time.time() - start
    if int(elapsed) == elapsed:
        print(f"Running for {int(elapsed)} seconds...")

print("CPU load test complete")
"""
            # Write the script to a file
            await sandbox.command.run(
                "bash", ["-c", f"cat > /tmp/cpu_test.py << 'EOF'\n{cpu_script}\nEOF"]
            )

            # Run the script in the background
            print("Starting CPU test (running for 10 seconds)...")
            await sandbox.command.run("python", ["/tmp/cpu_test.py", "&"])

            # Monitor CPU usage while the script runs
            print("\nMonitoring CPU usage...")
            for i in range(5):
                # Wait a moment
                await asyncio.sleep(2)

                # Get metrics
                cpu = await sandbox.metrics.cpu()
                memory = await sandbox.metrics.memory()

                # Format CPU usage (could be 0.0 or None)
                cpu_str = f"{cpu}%" if cpu is not None else "Not available"

                # Print current values
                print(
                    f"[{i * 2} seconds] CPU: {cpu_str}, Memory: {memory or 'Not available'} MiB"
                )

            print("CPU load test complete.")
        except Exception as e:
            print(f"Error in cpu_load_test_example: {e}")


async def error_handling_example():
    """Example showing error handling with metrics."""
    print("\n=== Error Handling Example ===")

    # Create a sandbox without starting it immediately
    sandbox = PythonSandbox(name="error-example")

    try:
        # Try to get metrics before starting the sandbox
        print("Trying to get metrics before starting the sandbox...")
        cpu = await sandbox.metrics.cpu()
        print(f"CPU: {cpu}%")  # This shouldn't be reached
    except RuntimeError as e:
        print(f"Expected error: {e}")

    try:
        # Now properly start the sandbox
        print("\nStarting the sandbox properly...")
        sandbox._session = aiohttp.ClientSession()
        await sandbox.start()

        # Get metrics after starting
        cpu = await sandbox.metrics.cpu()
        # Format CPU usage (could be 0.0 or None)
        cpu_str = f"{cpu}%" if cpu is not None else "Not available"
        print(f"CPU usage after starting: {cpu_str}")
    except Exception as e:
        print(f"Error: {e}")
    finally:
        # Clean up
        if sandbox._is_started:
            await sandbox.stop()
        if sandbox._session and not sandbox._session.closed:
            await sandbox._session.close()


async def main():
    """Main function to run all examples."""
    print("Sandbox Metrics Examples")
    print("=======================")

    try:
        await basic_metrics_example()
        await all_metrics_example()
        await continuous_monitoring_example()
        await cpu_load_test_example()
        await error_handling_example()
    except Exception as e:
        print(f"Error in main: {e}")

    print("\nAll examples completed!")


if __name__ == "__main__":
    asyncio.run(main())
