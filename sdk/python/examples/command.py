#!/usr/bin/env python3
"""
Example demonstrating how to use sandbox.command.run to execute shell commands.
"""

import asyncio

import aiohttp
from microsandbox import PythonSandbox


async def basic_example():
    """Example showing basic command execution with context manager."""
    print("\n=== Basic Command Example ===")

    # Create a sandbox using a context manager (automatically handles start/stop)
    async with PythonSandbox.create(sandbox_name="command-example") as sandbox:
        # Run a simple command
        ls_execution = await sandbox.command.run("ls", ["-la", "/"])
        print("$ ls -la /")
        print(f"Exit code: {ls_execution.exit_code}")
        print("Output:")
        print(await ls_execution.output())

        # Execute a command with string arguments
        echo_execution = await sandbox.command.run(
            "echo", ["Hello from", "sandbox command!"]
        )
        print("\n$ echo Hello from sandbox command!")
        print(f"Output: {await echo_execution.output()}")

        # Get system information
        uname_execution = await sandbox.command.run("uname", ["-a"])
        print("\n$ uname -a")
        print(f"Output: {await uname_execution.output()}")


async def error_handling_example():
    """Example showing how to handle command errors."""
    print("\n=== Error Handling Example ===")

    async with PythonSandbox.create(sandbox_name="error-example") as sandbox:
        # Run a command that generates an error
        error_execution = await sandbox.command.run("ls", ["/nonexistent"])

        print("$ ls /nonexistent")
        print(f"Exit code: {error_execution.exit_code}")
        print(f"Success: {error_execution.success}")
        print("Error output:")
        print(await error_execution.error())

        # Deliberately cause a command not found error
        try:
            _nonexistent_cmd = await sandbox.command.run("nonexistentcommand", [])
            # This should not execute if the command fails
            print("Command succeeded unexpectedly")
        except RuntimeError as e:
            print(f"\nCaught exception for nonexistent command: {e}")


async def timeout_example():
    """Example showing how to use command timeouts."""
    print("\n=== Timeout Example ===")

    async with PythonSandbox.create(sandbox_name="timeout-example") as sandbox:
        print("Running command with timeout...")
        try:
            # Run a command that takes longer than the specified timeout
            await sandbox.command.run("sleep", ["10"], timeout=2)
            print("Command completed (unexpected!)")
        except RuntimeError as e:
            print(f"Command timed out as expected: {e}")

        # Show that the sandbox is still usable after a timeout
        echo_execution = await sandbox.command.run("echo", ["Still working!"])
        print(f"\nSandbox still works: {await echo_execution.output()}")


async def advanced_example():
    """Example showing more advanced command usage."""
    print("\n=== Advanced Example ===")

    async with PythonSandbox.create(sandbox_name="advanced-example") as sandbox:
        # Write a file
        write_cmd = await sandbox.command.run(
            "bash", ["-c", "echo 'Hello, file content!' > /tmp/test.txt"]
        )
        print(f"Created file, exit code: {write_cmd.exit_code}")

        # Read the file back
        read_cmd = await sandbox.command.run("cat", ["/tmp/test.txt"])
        print(f"File content: {await read_cmd.output()}")

        # Run a more complex pipeline
        pipeline_cmd = await sandbox.command.run(
            "bash",
            [
                "-c",
                "mkdir -p /tmp/test_dir && "
                "echo 'Line 1' > /tmp/test_dir/data.txt && "
                "echo 'Line 2' >> /tmp/test_dir/data.txt && "
                "cat /tmp/test_dir/data.txt | grep 'Line' | wc -l",
            ],
        )
        print(f"\nPipeline output (should be 2): {await pipeline_cmd.output()}")

        # Create and run a Python script
        create_script = await sandbox.command.run(
            "bash",
            [
                "-c",
                """cat > /tmp/test.py << 'EOF'
import sys
print("Python script executed!")
print(f"Arguments: {sys.argv[1:]}")
EOF""",
            ],
        )

        if create_script.success:
            # Run the script with arguments
            script_cmd = await sandbox.command.run(
                "python", ["/tmp/test.py", "arg1", "arg2", "arg3"]
            )
            print("\nPython script output:")
            print(await script_cmd.output())


async def explicit_lifecycle_example():
    """Example showing explicit lifecycle management."""
    print("\n=== Explicit Lifecycle Example ===")

    # Create sandbox without context manager
    sandbox = PythonSandbox(sandbox_name="explicit-lifecycle")
    sandbox._session = aiohttp.ClientSession()

    try:
        # Manually start the sandbox
        print("Starting sandbox...")
        await sandbox.start()

        # Execute commands
        hostname_cmd = await sandbox.command.run("hostname")
        print(f"Hostname: {await hostname_cmd.output()}")

        date_cmd = await sandbox.command.run("date")
        print(f"Date: {await date_cmd.output()}")

    finally:
        # Manually stop the sandbox and close session
        print("Stopping sandbox...")
        await sandbox.stop()
        await sandbox._session.close()


async def main():
    """Main function to run all examples."""
    print("Command Execution Examples")
    print("=========================")

    await basic_example()
    await error_handling_example()
    await timeout_example()
    await advanced_example()
    await explicit_lifecycle_example()

    print("\nAll examples completed!")


if __name__ == "__main__":
    asyncio.run(main())
