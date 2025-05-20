#!/usr/bin/env python3
"""
Example demonstrating how to use NodeSandbox to execute JavaScript code.

This example shows:
1. Basic Node.js code execution
2. Different sandbox management patterns
3. JavaScript module usage
4. Error output handling

Before running this example:
    1. Install the package: pip install -e .
    2. Start the Microsandbox server (microsandbox-server)
    3. Run this script: python -m examples.node
"""

import asyncio

from microsandbox import NodeSandbox


async def basic_example():
    """Example showing basic JavaScript code execution with context manager."""
    print("\n=== Basic Node.js Example ===")

    # Create a sandbox using a context manager (automatically handles start/stop)
    async with NodeSandbox.create(sandbox_name="node-basic") as sandbox:
        # Run a simple JavaScript code snippet
        execution = await sandbox.run("console.log('Hello from Node.js!');")
        output = await execution.output()
        print("Output:", output)

        # Run JavaScript code that uses Node.js functionality
        version_code = """
const version = process.version;
const platform = process.platform;
console.log(`Node.js ${version} running on ${platform}`);
"""
        version_execution = await sandbox.run(version_code)
        print("Node.js info:", await version_execution.output())


async def error_handling_example():
    """Example showing how to handle JavaScript errors."""
    print("\n=== Error Handling Example ===")

    async with NodeSandbox.create(sandbox_name="node-error") as sandbox:
        # Run code with a caught error
        caught_error_code = """
try {
    // This will cause a ReferenceError
    console.log(undefinedVariable);
} catch (error) {
    console.error('Caught error:', error.message);
}
"""
        caught_execution = await sandbox.run(caught_error_code)
        print("Standard output:", await caught_execution.output())
        print("Error output:", await caught_execution.error())
        print("Has error:", caught_execution.has_error())


async def module_example():
    """Example showing Node.js module usage."""
    print("\n=== Module Usage Example ===")

    async with NodeSandbox.create(sandbox_name="node-module") as sandbox:
        # Using built-in Node.js modules
        fs_code = """
const fs = require('fs');
const os = require('os');

// Write a file
fs.writeFileSync('/tmp/hello.txt', 'Hello from Node.js!');
console.log('File written successfully');

// Read the file back
const content = fs.readFileSync('/tmp/hello.txt', 'utf8');
console.log('File content:', content);

// Get system info
console.log('Hostname:', os.hostname());
console.log('Platform:', os.platform());
console.log('Architecture:', os.arch());
"""
        fs_execution = await sandbox.run(fs_code)
        print(await fs_execution.output())


async def execution_chaining_example():
    """Example demonstrating execution chaining with variables."""
    print("\n=== Execution Chaining Example ===")

    async with NodeSandbox.create(sandbox_name="node-chain") as sandbox:
        # Execute a sequence of related code blocks that maintain state
        await sandbox.run("const name = 'Node.js';")
        await sandbox.run("const version = process.version;")
        await sandbox.run("const numbers = [1, 2, 3, 4, 5];")

        # Use variables from previous executions
        final_execution = await sandbox.run("""
console.log(`Hello from ${name} ${version}!`);
const sum = numbers.reduce((a, b) => a + b, 0);
console.log(`Sum of numbers: ${sum}`);
""")

        print(await final_execution.output())


async def main():
    """Run all examples."""
    print("Node.js Sandbox Examples")
    print("=======================")

    try:
        await basic_example()
        await error_handling_example()
        await module_example()
        await execution_chaining_example()

        print("\nAll Node.js examples completed!")
    except Exception as e:
        print(f"Error running examples: {e}")


if __name__ == "__main__":
    asyncio.run(main())
