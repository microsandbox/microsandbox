/**
 * Advanced example demonstrating the Python sandbox features.
 *
 * This example shows:
 * 1. Different ways to create and manage sandboxes
 * 2. Resource configuration (memory, CPU)
 * 3. Error handling
 * 4. Multiple code execution patterns
 * 5. Output handling
 * 6. Timeouts and handling long-running starts
 *
 * Before running this example:
 *     1. Install the package: npm install
 *     2. Start the Microsandbox server (microsandbox-server)
 *     3. Run this script: npx ts-node examples/python.ts
 *
 * Note: If authentication is enabled on the server, set MSB_API_KEY in your environment.
 */

import { PythonSandbox } from "../src";

async function exampleContextManager() {
  /**
   * Example demonstrating basic sandbox creation and cleanup.
   */
  console.log("\n=== Simple Sandbox Example ===");

  // Create and start a sandbox (equivalent to Python's async with)
  const sandbox = await PythonSandbox.create({ name: "sandbox-cm" });

  try {
    // Run some computation
    const code = `
print("Hello, world!")
`;
    const execution = await sandbox.run(code);
    const output = await execution.output();
    console.log("Output:", output);
  } finally {
    // Always stop the sandbox (equivalent to Python's context manager cleanup)
    await sandbox.stop();
  }
}

async function exampleExplicitLifecycle() {
  /**
   * Example using explicit lifecycle management.
   */
  console.log("\n=== Explicit Lifecycle Example ===");

  // Create sandbox with custom configuration
  const sandbox = new PythonSandbox({
    serverUrl: "http://127.0.0.1:5555",
    name: "sandbox-explicit",
  });

  try {
    // Start with resource constraints
    await sandbox.start(
      undefined, // use default image
      1024, // 1GB RAM
      2.0 // 2 CPU cores
    );

    // Run multiple code blocks with variable assignments
    await sandbox.run("x = 42");
    await sandbox.run("y = [i**2 for i in range(10)]");
    const execution3 = await sandbox.run(
      "print(f'x = {x}')\nprint(f'y = {y}')"
    );

    console.log("Output:", await execution3.output());

    // Demonstrate error handling
    try {
      const errorExecution = await sandbox.run("1/0"); // This will raise a ZeroDivisionError
      console.log("Error:", await errorExecution.error());
    } catch (e) {
      console.log("Caught error:", e instanceof Error ? e.message : e);
    }
  } finally {
    // Cleanup
    await sandbox.stop();
  }
}

async function exampleExecutionChaining() {
  /**
   * Example demonstrating execution chaining with variables.
   */
  console.log("\n=== Execution Chaining Example ===");

  const sandbox = await PythonSandbox.create({ name: "sandbox-chain" });

  try {
    // Execute a sequence of related code blocks
    await sandbox.run("name = 'Python'");
    await sandbox.run("import sys");
    await sandbox.run("version = sys.version");
    const exec = await sandbox.run("print(f'Hello from {name} {version}!')");

    // Only get output from the final execution
    console.log("Output:", await exec.output());
  } finally {
    await sandbox.stop();
  }
}

async function main() {
  /**
   * Run all examples.
   */
  try {
    await exampleContextManager();
    await exampleExplicitLifecycle();
    await exampleExecutionChaining();

    console.log("\nAll Python examples completed!");
  } catch (e) {
    console.error(
      `Error running examples: ${e instanceof Error ? e.message : e}`
    );
  }
}

main();
