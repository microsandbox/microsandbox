/**
 * Example demonstrating how to use sandbox.command.run to execute shell commands.
 *
 * Before running this example:
 *     1. Install the package: npm install
 *     2. Start the Microsandbox server (microsandbox-server)
 *     3. Run this script: npx ts-node examples/command.ts
 */

import { PythonSandbox } from "../src";

async function basicExample() {
  /**
   * Example showing basic command execution with context manager.
   */
  console.log("\n=== Basic Command Example ===");

  // Create a sandbox using a try/finally pattern (equivalent to Python's context manager)
  const sandbox = await PythonSandbox.create({ name: "command-example" });

  try {
    // Run a simple command
    const lsExecution = await sandbox.command.run("ls", ["-la", "/"]);
    console.log("$ ls -la /");
    console.log(`Exit code: ${lsExecution.exitCode}`);
    console.log("Output:");
    console.log(await lsExecution.output());

    // Execute a command with string arguments
    const echoExecution = await sandbox.command.run("echo", [
      "Hello from",
      "sandbox command!",
    ]);
    console.log("\n$ echo Hello from sandbox command!");
    console.log(`Output: ${await echoExecution.output()}`);

    // Get system information
    const unameExecution = await sandbox.command.run("uname", ["-a"]);
    console.log("\n$ uname -a");
    console.log(`Output: ${await unameExecution.output()}`);
  } finally {
    await sandbox.stop();
  }
}

async function errorHandlingExample() {
  /**
   * Example showing how to handle command errors.
   */
  console.log("\n=== Error Handling Example ===");

  const sandbox = await PythonSandbox.create({ name: "error-example" });

  try {
    // Run a command that generates an error
    const errorExecution = await sandbox.command.run("ls", ["/nonexistent"]);

    console.log("$ ls /nonexistent");
    console.log(`Exit code: ${errorExecution.exitCode}`);
    console.log(`Success: ${errorExecution.success}`);
    console.log("Error output:");
    console.log(await errorExecution.error());

    // Deliberately cause a command not found error
    try {
      await sandbox.command.run("nonexistentcommand", []);
      // This should not execute if the command fails
      console.log("Command succeeded unexpectedly");
    } catch (e) {
      console.log(
        `\nCaught exception for nonexistent command: ${
          e instanceof Error ? e.message : e
        }`
      );
    }
  } finally {
    await sandbox.stop();
  }
}

async function timeoutExample() {
  /**
   * Example showing how to use command timeouts.
   */
  console.log("\n=== Timeout Example ===");

  const sandbox = await PythonSandbox.create({ name: "timeout-example" });

  try {
    console.log("Running command with timeout...");
    try {
      // Run a command that takes longer than the specified timeout
      await sandbox.command.run("sleep", ["10"], 2);
      console.log("Command completed (unexpected!)");
    } catch (e) {
      console.log(
        `Command timed out as expected: ${e instanceof Error ? e.message : e}`
      );
    }

    // Show that the sandbox is still usable after a timeout
    const echoExecution = await sandbox.command.run("echo", ["Still working!"]);
    console.log(`\nSandbox still works: ${await echoExecution.output()}`);
  } finally {
    await sandbox.stop();
  }
}

async function advancedExample() {
  /**
   * Example showing more advanced command usage.
   */
  console.log("\n=== Advanced Example ===");

  const sandbox = await PythonSandbox.create({ name: "advanced-example" });

  try {
    // Write a file
    const writeCmd = await sandbox.command.run("bash", [
      "-c",
      "echo 'Hello, file content!' > /tmp/test.txt",
    ]);
    console.log(`Created file, exit code: ${writeCmd.exitCode}`);

    // Read the file back
    const readCmd = await sandbox.command.run("cat", ["/tmp/test.txt"]);
    console.log(`File content: ${await readCmd.output()}`);

    // Run a more complex pipeline
    const pipelineCmd = await sandbox.command.run("bash", [
      "-c",
      "mkdir -p /tmp/test_dir && " +
        "echo 'Line 1' > /tmp/test_dir/data.txt && " +
        "echo 'Line 2' >> /tmp/test_dir/data.txt && " +
        "cat /tmp/test_dir/data.txt | grep 'Line' | wc -l",
    ]);
    console.log(
      `\nPipeline output (should be 2): ${await pipelineCmd.output()}`
    );

    // Create and run a Python script
    const createScript = await sandbox.command.run("bash", [
      "-c",
      `cat > /tmp/test.py << 'EOF'
import sys
print("Python script executed!")
print(f"Arguments: {sys.argv[1:]}")
EOF`,
    ]);

    if (createScript.success) {
      // Run the script with arguments
      const scriptCmd = await sandbox.command.run("python", [
        "/tmp/test.py",
        "arg1",
        "arg2",
        "arg3",
      ]);
      console.log("\nPython script output:");
      console.log(await scriptCmd.output());
    }
  } finally {
    await sandbox.stop();
  }
}

async function explicitLifecycleExample() {
  /**
   * Example showing explicit lifecycle management.
   */
  console.log("\n=== Explicit Lifecycle Example ===");

  // Create sandbox without context manager
  const sandbox = new PythonSandbox({ name: "explicit-lifecycle" });

  try {
    // Manually start the sandbox
    console.log("Starting sandbox...");
    await sandbox.start();

    // Execute commands
    const hostnameCmd = await sandbox.command.run("hostname");
    console.log(`Hostname: ${await hostnameCmd.output()}`);

    const dateCmd = await sandbox.command.run("date");
    console.log(`Date: ${await dateCmd.output()}`);
  } finally {
    // Manually stop the sandbox
    console.log("Stopping sandbox...");
    await sandbox.stop();
  }
}

async function main() {
  /**
   * Main function to run all examples.
   */
  console.log("Command Execution Examples");
  console.log("=========================");

  try {
    await basicExample();
    await errorHandlingExample();
    await timeoutExample();
    await advancedExample();
    await explicitLifecycleExample();

    console.log("\nAll command examples completed!");
  } catch (e) {
    console.error(
      `Error running examples: ${e instanceof Error ? e.message : e}`
    );
  }
}

main();
