/**
 * Example demonstrating how to use NodeSandbox to execute JavaScript code.
 *
 * This example shows:
 * 1. Basic Node.js code execution
 * 2. Different sandbox management patterns
 * 3. JavaScript module usage
 * 4. Error output handling
 *
 * Before running this example:
 *     1. Install the package: npm install
 *     2. Start the Microsandbox server (microsandbox-server)
 *     3. Run this script: npx ts-node examples/node.ts
 */

import { NodeSandbox } from "../src";

async function basicExample() {
  /**
   * Example showing basic JavaScript code execution with context manager.
   */
  console.log("\n=== Basic Node.js Example ===");

  // Create a sandbox (with try/finally to simulate Python's context manager)
  const sandbox = await NodeSandbox.create({ name: "node-basic" });

  try {
    // Run a simple JavaScript code snippet
    const execution = await sandbox.run("console.log('Hello from Node.js!');");
    const output = await execution.output();
    console.log("Output:", output);

    // Run JavaScript code that uses Node.js functionality
    const versionCode = `
const version = process.version;
const platform = process.platform;
console.log(\`Node.js \${version} running on \${platform}\`);
`;
    const versionExecution = await sandbox.run(versionCode);
    console.log("Node.js info:", await versionExecution.output());
  } finally {
    await sandbox.stop();
  }
}

async function errorHandlingExample() {
  /**
   * Example showing how to handle JavaScript errors.
   */
  console.log("\n=== Error Handling Example ===");

  const sandbox = await NodeSandbox.create({ name: "node-error" });

  try {
    // Run code with a caught error
    const caughtErrorCode = `
try {
    // This will cause a ReferenceError
    console.log(undefinedVariable);
} catch (error) {
    console.error('Caught error:', error.message);
}
`;
    const caughtExecution = await sandbox.run(caughtErrorCode);
    console.log("Standard output:", await caughtExecution.output());
    console.log("Error output:", await caughtExecution.error());
    console.log("Has error:", caughtExecution.hasError());
  } finally {
    await sandbox.stop();
  }
}

async function moduleExample() {
  /**
   * Example showing Node.js module usage.
   */
  console.log("\n=== Module Usage Example ===");

  const sandbox = await NodeSandbox.create({ name: "node-module" });

  try {
    // Using built-in Node.js modules
    const fsCode = `
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
`;
    const fsExecution = await sandbox.run(fsCode);
    console.log(await fsExecution.output());
  } finally {
    await sandbox.stop();
  }
}

async function executionChainingExample() {
  /**
   * Example demonstrating execution chaining with variables.
   */
  console.log("\n=== Execution Chaining Example ===");

  const sandbox = await NodeSandbox.create({ name: "node-chain" });

  try {
    // Execute a sequence of related code blocks that maintain state
    await sandbox.run("const name = 'Node.js';");
    await sandbox.run("const version = process.version;");
    await sandbox.run("const numbers = [1, 2, 3, 4, 5];");

    // Use variables from previous executions
    const finalExecution = await sandbox.run(`
console.log(\`Hello from \${name} \${version}!\`);
const sum = numbers.reduce((a, b) => a + b, 0);
console.log(\`Sum of numbers: \${sum}\`);
`);

    console.log(await finalExecution.output());
  } finally {
    await sandbox.stop();
  }
}

async function main() {
  /**
   * Run all examples.
   */
  console.log("Node.js Sandbox Examples");
  console.log("=======================");

  try {
    await basicExample();
    await errorHandlingExample();
    await moduleExample();
    await executionChainingExample();

    console.log("\nAll Node.js examples completed!");
  } catch (e) {
    console.error(
      `Error running examples: ${e instanceof Error ? e.message : e}`
    );
  }
}

main();
